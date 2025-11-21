use std::process::exit;

use glium::{
    Surface, glutin::surface::WindowSurface, uniform, winit::{self, application::ApplicationHandler, window::Window}
};
// mod structs;
mod obj_parcer;
// use obj_parcer::Vertex;

struct mat4f {
    data: [[f32; 4]; 4],
}



pub struct App {
    display: glium::Display<WindowSurface>,
    window: Window,
    t: f32,
    obj: obj_parcer::Obj,
    obj_type: String,
}

// Replace the old perspective(...) with a simpler, OpenGL-friendly one.
// Now takes fov_y (radians) and aspect ratio, returns column-major mat4 (data[column][row]).
fn perspective(fov_y: f32, aspect: f32, near: f32, far: f32) -> mat4f {
	// f = 1 / tan(fov_y/2)
	let f = 1.0f32 / (fov_y * 0.5).tan();

	let mut m = mat4f { data: [[0.0; 4]; 4] };

	// Column-major layout: data[column][row]
	// Column 0
	m.data[0][0] = f / aspect;
	m.data[0][1] = 0.0;
	m.data[0][2] = 0.0;
	m.data[0][3] = 0.0;

	// Column 1
	m.data[1][0] = 0.0;
	m.data[1][1] = f;
	m.data[1][2] = 0.0;
	m.data[1][3] = 0.0;

	// Column 2
	// (far + near) / (near - far)  and -1 in row 3
	m.data[2][0] = 0.0;
	m.data[2][1] = 0.0;
	m.data[2][2] = (far + near) / (near - far);
	m.data[2][3] = -1.0;

	// Column 3
	// (2 * far * near) / (near - far)
	m.data[3][0] = 0.0;
	m.data[3][1] = 0.0;
	m.data[3][2] = (2.0 * far * near) / (near - far);
	m.data[3][3] = 0.0;

	m
}




impl ApplicationHandler for App {
    fn resumed(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        // self.window = Some(event_loop.create_window(Window::default_attributes())).unwrap().unwrap();
    }


    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            glium::winit::event::WindowEvent::CloseRequested => event_loop.exit(),
            glium::winit::event::WindowEvent::Resized(new_size) => {
                self.display.resize(new_size.into());
            }
            glium::winit::event::WindowEvent::RedrawRequested => {
                let mut target = self.display.draw();
                target.clear_color(1.0, 1.0, 1.0, 1.0);
                self.t += 0.02;

                // let x_off = self.t.sin() * 0.5;
                // println!("x_off: {}, t: {}", x_off, self.t);

                // let simple_triangle = [
                //     obj_parcer::Vertex { position: [ -0.5, -0.5, 0.0 ] },
                //     obj_parcer::Vertex { position: [  0.0,  0.5, 0.0 ] },
                //     obj_parcer::Vertex { position: [  0.5, -0.25, 0.0 ] },
                // ];

                let vertex_buffer = glium::VertexBuffer::new(&self.display, &self.obj.vertices).unwrap();
                let indices = glium::index::IndexBuffer::new(&self.display, glium::index::PrimitiveType::TrianglesList, &self.obj.indices).unwrap();
                
                // Translation: convert to column-major
                let translation_mat: mat4f = mat4f { data: [
                    // column 0
                    [1.0, 0.0, 0.0, 0.0],
                    // column 1
                    [0.0, 1.0, 0.0, 0.0],
                    // column 2
                    [0.0, 0.0, 1.0, 0.0],
                    // column 3 (translation goes in rows 0..2 of column 3, w=1 at row3)
                    [0.0, 0.0, -5.0, 1.0],
                ] };

                // Rotation matrices around X, Y, Z axes (use helpers)
                let rot_x = rotation_x(0.0);
                let rot_y = rotation_y(self.t);
                let rot_z = rotation_z(0.0);
                // Compose rotations: R = Rz * Ry * Rx
                let rotation_mat = mat_mul(&rot_z, &mat_mul(&rot_y, &rot_x));
                
                // Scale (identity here) in column-major
                let scale_mat: mat4f = mat4f { data: [
                    [1.0, 0.0, 0.0, 0.0],
                    [0.0, 1.0, 0.0, 0.0],
                    [0.0, 0.0, 1.0, 0.0],
                    [0.0, 0.0, 0.0, 1.0],
                ] };

                let (w, h) = self.display.get_framebuffer_dimensions();
                let aspect = (w as f32) / (h as f32);
                let fov_y: f32 = std::f32::consts::FRAC_PI_4; // 45 degrees
                let near: f32 = 0.1;
                let far: f32 = 100.0;
                // Use new perspective API (fov_y, aspect, near, far)
                let projection_mat: mat4f = perspective(fov_y, aspect, near, far);

                // View matrix: translate camera along Z by -3 (column-major)
                let view_mat: mat4f = mat4f { data: [
                    [1.0, 0.0, 0.0, 0.0],
                    [0.0, 1.0, 0.0, 0.0],
                    [0.0, 0.0, 1.0, 0.0],
                    [0.0, 0.0, -3.0, 1.0],
                ] };


                let vertex_shader_src = r#"
                        #version 140
                        in vec3 position;
                        uniform mat4 rotation;
                        uniform mat4 translation;
                        uniform mat4 scale;
                        uniform mat4 projection;
                        uniform mat4 view;
                        void main() {
                            // Note: matrices are provided column-major so no transpose is necessary.
                            // Apply model = translation * rotation * scale, then view and projection.
                            gl_Position = projection * view * translation * rotation * scale * vec4(position, 1.0);
                        }
                    "#;

                let fragment_shader_src = r#"
                        #version 140

                        out vec4 color;

                        void main() {
                            color = vec4(1.0, 0.0, 0.0, 1.0);
                        }
                    "#;

                let program = glium::Program::from_source(
                    &self.display,
                    vertex_shader_src,
                    fragment_shader_src,
                    None,
                );
                match program {
                    Ok(program) => {
                        let drawing = target.draw(
                            &vertex_buffer,
                            &indices,
                            &program,
                            &uniform! { 
                                rotation: rotation_mat.data,
                                translation: translation_mat.data,
                                scale: scale_mat.data,
                                projection: projection_mat.data,
                                view: view_mat.data },
                            &Default::default(),
                        );
                        match drawing {
                            Ok(_) => {
                                target.finish().unwrap();
                                self.window.request_redraw();
                            }
                            Err(e) => {
                                println!("Failed to draw: {}", e);
                                self.window.request_redraw();
                                return;
                            }
                        }
                    }
                    Err(e) => {
                        println!("Failed to compile shader program: {}", e);
                        target.finish().unwrap();
                        exit(1);
                    }
                }
            }
            _ => (),
        }
    }
}

// Column-major matrix multiplication: returns A * B.
// Matrices use data[column][row].
fn mat_mul(a: &mat4f, b: &mat4f) -> mat4f {
    let mut m = mat4f { data: [[0.0; 4]; 4] };
    for col in 0..4 {
        for row in 0..4 {
            let mut sum = 0.0;
            for k in 0..4 {
                // A_{row,k} is a.data[k][row]; B_{k,col} is b.data[col][k]
                sum += a.data[k][row] * b.data[col][k];
            }
            m.data[col][row] = sum;
        }
    }
    m
}

// --- new rotation helper functions (column-major) ---
fn rotation_x(theta: f32) -> mat4f {
    let (s, c) = (theta.sin(), theta.cos());
    mat4f { data: [
        // column 0
        [1.0, 0.0, 0.0, 0.0],
        // column 1
        [0.0, c, s, 0.0],
        // column 2
        [0.0, -s, c, 0.0],
        // column 3
        [0.0, 0.0, 0.0, 1.0],
    ] }
}

fn rotation_y(theta: f32) -> mat4f {
    let (s, c) = (theta.sin(), theta.cos());
    mat4f { data: [
        // column 0
        [c, 0.0, -s, 0.0],
        // column 1
        [0.0, 1.0, 0.0, 0.0],
        // column 2
        [s, 0.0, c, 0.0],
        // column 3
        [0.0, 0.0, 0.0, 1.0],
    ] }
}

fn rotation_z(theta: f32) -> mat4f {
    let (s, c) = (theta.sin(), theta.cos());
    mat4f { data: [
        // column 0
        [c, s, 0.0, 0.0],
        // column 1
        [-s, c, 0.0, 0.0],
        // column 2
        [0.0, 0.0, 1.0, 0.0],
        // column 3
        [0.0, 0.0, 0.0, 1.0],
    ] }
}

fn main() {
    // 1. The **winit::EventLoop** for handling events.
    let event_loop = winit::event_loop::EventLoop::builder().build().unwrap();
    // 2. Create a glutin context and glium Display
    let (window, display) = glium::backend::glutin::SimpleWindowBuilder::new().build(&event_loop);
 

    let obj = obj_parcer::Obj::read_file("../scope_res/resources/42.obj", "42");
    let mut app: App;
 
    match obj {
        Ok(o) => {
            println!("Successfully parsed OBJ file:");
            for vertex in &o.vertices {
                println!("Vertex position: {:?}, UV: {:?}", vertex.position, vertex.uv);                
            }
            println!("Faces: {:?}", o.faces);
            println!("Indices: {:?}", o.indices);
            println!("File Name: {}", o.file_name);
            app = App {
                display,
                window,
                t: 0.0,
                obj: o,
                obj_type: "42".to_string(),
            };
        }
        Err(e) => {
            println!("Error parsing OBJ file: {}", e);
            exit(1);
        }
        
    }
    let _run = event_loop.run_app(&mut app);
    match _run {
        Ok(_) => (),
        Err(e) => println!("Error during event loop: {}", e),
    }
}
