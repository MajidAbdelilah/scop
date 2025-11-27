use std::{env, process::exit};

use glium::{
    Program, ProgramCreationError, Surface, Texture2d, glutin::surface::{SwapInterval, WindowSurface}, implement_uniform_block, uniform, winit::{self, application::ApplicationHandler, keyboard::PhysicalKey, window::Window}
};
// mod structs;
mod obj_parcer;
use obj_parcer::vec3_normalize;
// use obj_parcer::Vertex;

struct Mat4f {
    data: [[f32; 4]; 4],
}

#[derive(Copy, Clone)]
struct ColorArr
{
    color_position: [[f32; 4]; 5],
}implement_uniform_block!(ColorArr, color_position);

pub struct App {
    display: glium::Display<WindowSurface>,
    window: Window,
    t: f64,
    obj: obj_parcer::Obj,
    program: Result<Program, ProgramCreationError>,
    tex: Texture2d,
    w_h: (u32, u32),
    shading_lerp_val: f32,
    fram_time: f64,
    set_texture: bool,
    rotation_axis: [f32; 3],
    rotation_direction: f32,
    lerp_time: f32,
    key: winit::keyboard::PhysicalKey,
    isrepeat_key: bool,
    is_key_pressed: bool,
    trigger_rot_anim: (bool, winit::keyboard::PhysicalKey),
    trigger_rot_reverse: (bool, f32, f32, f32),
    color_positions: glium::uniforms::UniformBuffer<ColorArr>,
    color: ColorArr,
    is_lizzard: bool,
    time_mul_delta: f64,
}

fn dot(a: &[f32; 3], b: &[f32; 3]) -> f32 {
    return a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
}

// Compute the cross product of two [f32; 3] vectors.
fn cross(a: &[f32; 3], b: &[f32; 3]) -> [f32; 3] {
    return [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ];
}

fn look_at(&position: &[f32; 3], &lookat: &[f32; 3], &up: &[f32; 3]) -> Mat4f {
    let mut look_at = lookat.clone();
    look_at[0] -= position[0];
    look_at[1] -= position[1];
    look_at[2] -= position[2];

    let f: &[f32; 3] = &vec3_normalize(look_at); // Camera's direction vector
    let s: &[f32; 3] = &vec3_normalize(cross(f, &up)); // Camera's right vector
    let u: &[f32; 3] = &cross(s, f); // Camera's corrected up vector

    let mut result: Mat4f = Mat4f {
        data: [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ],
    };
    result.data[0][0] = s[0];
    result.data[0][1] = u[0];
    result.data[0][2] = -f[0];
    result.data[0][3] = 0.0;

    result.data[1][0] = s[1];
    result.data[1][1] = u[1];
    result.data[1][2] = -f[1];
    result.data[1][3] = 0.0;

    result.data[2][0] = s[2];
    result.data[2][1] = u[2];
    result.data[2][2] = -f[2];
    result.data[2][3] = 0.0;

    result.data[3][0] = -dot(s, &position);
    result.data[3][1] = -dot(u, &position);
    result.data[3][2] = dot(f, &position);
    result.data[3][3] = 1.0;

    return result;
}

// Replace the old perspective(...) with a simpler, OpenGL-friendly one.
// Now takes fov_y (radians) and aspect ratio, returns column-major mat4 (data[column][row]).
fn perspective(fov_y: f32, aspect: f32, near: f32, far: f32) -> Mat4f {
    // f = 1 / tan(fov_y/2)
    let f = 1.0 / (fov_y * 0.5).tan();

    let mut m = Mat4f {
        data: [[0.0; 4]; 4],
    };

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

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    let lerp = a + (b - a) * t;
    return lerp;
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
            glium::winit::event::WindowEvent::CloseRequested => {
                event_loop.exit();
                exit(0)
            }
            glium::winit::event::WindowEvent::Resized(new_size) => {
                self.display.resize(new_size.into());
                self.w_h = new_size.into();
                self.window.request_redraw();
            }

            glium::winit::event::WindowEvent::KeyboardInput {
                device_id: _,
                event,
                is_synthetic: _,
            } => {
                if event.physical_key == PhysicalKey::Code(winit::keyboard::KeyCode::KeyQ)
                    && event.state.is_pressed()
                    && event.repeat == false
                {
                    self.set_texture = !self.set_texture;
                }
                if event.physical_key == PhysicalKey::Code(winit::keyboard::KeyCode::KeyT)
                && event.state.is_pressed()
                && event.repeat == false
                {
                    self.is_lizzard = !self.is_lizzard;
                    if !self.is_lizzard
                    {
                        self.color = ColorArr { color_position: [[0.1, 0.1, 0.1, 0.1], [0.2, 0.2, 0.2, 0.2], [0.3, 0.3, 0.3, 0.0], [0.4, 0.4, 0.4, 0.0], [0.5, 0.5, 0.5, 0.0]] };
                        self.color_positions.write(&self.color);
                    }
                }
                
                
                self.key = event.physical_key;
                self.isrepeat_key = event.repeat;
                self.is_key_pressed = event.state.is_pressed();
            }

            glium::winit::event::WindowEvent::RedrawRequested => {
                let start_time = std::time::Instant::now();

                let mut target = self.display.draw();
                target.clear_color_and_depth((1.0, 1.0, 1.0, 1.0), 1.0);
                self.time_mul_delta += 1.0 * self.fram_time;
                
                if self.key == PhysicalKey::Code(winit::keyboard::KeyCode::KeyY)
                    && self.is_key_pressed
                    && self.isrepeat_key == false
                {
                    self.trigger_rot_reverse = (true, if self.rotation_direction <= 0.0  {1.0} else {-1.0}, self.rotation_direction, 0.0);
                }

                if self.trigger_rot_reverse.0 
                {
                    self.rotation_direction = lerp(self.rotation_direction, self.trigger_rot_reverse.1 , self.trigger_rot_reverse.3);
                     if (self.trigger_rot_reverse.1 - self.rotation_direction).abs() < 0.01
                     {
                        self.rotation_direction = self.rotation_direction.round();
                        self.trigger_rot_reverse.0 = false;
                        self.trigger_rot_reverse.2 = 0.0;
                     } 
                     println!("self.trigger_rot_reverse.1 = {} , self.rotation_direction = {}", self.trigger_rot_reverse.1, self.rotation_direction);
                    self.trigger_rot_reverse.3 += (0.001 * self.fram_time) as f32;
                }
                
                if self.is_lizzard
                {
                    for i in 0..5
                    {
                        self.color.color_position[i][0] = (self.color.color_position[(i+6) % 5][i % 3] + self.time_mul_delta as f32 * 3.5).cos().abs();
                        self.color.color_position[i][1] = (self.color.color_position[(i+3) % 5][i % 3] + self.time_mul_delta as f32 * 2.5).cos().abs();
                        self.color.color_position[i][2] = (self.color.color_position[(i+2) % 5][i % 3] + self.time_mul_delta as f32 * 1.5).cos().abs();
                        self.color.color_position[i][3] = 0.0;
                    }


                    self.color_positions.write(&self.color);
                }
                
                self.t +=  2.0 * self.fram_time;
                if self.t >= (std::f64::consts::PI * 2.0) && self.trigger_rot_anim.0 == false && self.trigger_rot_reverse.0 == false
                {
                    self.t -= std::f64::consts::PI * 2.0;
                }

                if self.key == PhysicalKey::Code(winit::keyboard::KeyCode::KeyW)
                    && self.is_key_pressed
                    && self.isrepeat_key == false
                {
                    self.lerp_time = 0.0;
                    self.trigger_rot_anim.0 = true;
                    self.trigger_rot_anim.1 = self.key;
                    
                }
                if self.key == PhysicalKey::Code(winit::keyboard::KeyCode::KeyE)
                    && self.is_key_pressed
                    && self.isrepeat_key == false
                {
                    self.trigger_rot_anim.0 = true;
                    self.trigger_rot_anim.1 = self.key;
                    
                    self.lerp_time = 0.0;
                }
                if self.key == PhysicalKey::Code(winit::keyboard::KeyCode::KeyR)
                    && self.is_key_pressed
                    && self.isrepeat_key == false
                {
                    self.trigger_rot_anim.0 = true;
                    self.trigger_rot_anim.1 = self.key;
                    
                    self.lerp_time = 0.0;
                }

                if self.trigger_rot_anim.1 == PhysicalKey::Code(winit::keyboard::KeyCode::KeyW)
                    && self.trigger_rot_anim.0
                {
                    self.rotation_axis;
                    if self.rotation_axis[0] < 0.999 {
                        self.rotation_axis[0] = lerp(self.rotation_axis[0], 1.0, self.lerp_time);
                    }
                    if self.rotation_axis[1] > 0.001 {
                        self.rotation_axis[1] = lerp(self.rotation_axis[1], 0.0, self.lerp_time);
                    }
                    if self.rotation_axis[2] > 0.001 {
                        self.rotation_axis[2] = lerp(self.rotation_axis[2], 0.0, self.lerp_time);
                    }
                    if self.rotation_axis[0] > 0.999
                        && self.rotation_axis[1] < 0.001
                        && self.rotation_axis[2] < 0.001
                    {
                        for v in &mut self.rotation_axis {
                            *v = v.round();
                        }
                        self.trigger_rot_anim.0 = false;
                        self.lerp_time = 0.0;
                    } else {
                        self.lerp_time += (0.001 * self.fram_time) as f32;
                    }
                }

                if self.trigger_rot_anim.1 == PhysicalKey::Code(winit::keyboard::KeyCode::KeyE)
                    && self.trigger_rot_anim.0
                {
                    self.rotation_axis;
                    if self.rotation_axis[0] > 0.001 {
                        self.rotation_axis[0] = lerp(self.rotation_axis[0], 0.0, self.lerp_time);
                    }
                    if self.rotation_axis[1] < 0.999 {
                        self.rotation_axis[1] = lerp(self.rotation_axis[1], 1.0, self.lerp_time);
                    }
                    if self.rotation_axis[2] > 0.001 {
                        self.rotation_axis[2] = lerp(self.rotation_axis[2], 0.0, self.lerp_time);
                    }
                    if self.rotation_axis[0] < 0.001
                        && self.rotation_axis[1] > 0.999
                        && self.rotation_axis[2] < 0.001
                    {
                        for v in &mut self.rotation_axis {
                            *v = v.round();
                        }
                        self.trigger_rot_anim.0 = false;
                        self.lerp_time = 0.0;
                    } else {
                        self.lerp_time += (0.001 * self.fram_time) as f32;
                    }
                }
                if self.trigger_rot_anim.1 == PhysicalKey::Code(winit::keyboard::KeyCode::KeyR)
                    && self.trigger_rot_anim.0
                {
                    self.rotation_axis;
                    if self.rotation_axis[0] > 0.001 {
                        self.rotation_axis[0] = lerp(self.rotation_axis[0], 0.0, self.lerp_time);
                    }
                    if self.rotation_axis[1] > 0.001 {
                        self.rotation_axis[1] = lerp(self.rotation_axis[1], 0.0, self.lerp_time);
                    }
                    if self.rotation_axis[2] < 0.999 {
                        self.rotation_axis[2] = lerp(self.rotation_axis[2], 1.0, self.lerp_time);
                    }
                    if self.rotation_axis[0] < 0.001
                        && self.rotation_axis[1] < 0.001
                        && self.rotation_axis[2] > 0.999
                    {
                        for v in &mut self.rotation_axis {
                            *v = v.round();
                        }
                        self.trigger_rot_anim.0 = false;
                        self.lerp_time = 0.0;
                    } else {
                        self.lerp_time += (0.001 * self.fram_time) as f32;
                    }
                }

                

                if self.set_texture && self.shading_lerp_val < 1.0 {
                    self.shading_lerp_val += (2.0 * self.fram_time) as f32;
                }
                if !self.set_texture && self.shading_lerp_val > 0.0 {
                    self.shading_lerp_val -= (2.0 * self.fram_time) as f32;
                }

                self.shading_lerp_val = self.shading_lerp_val.clamp(0.0, 1.0);
                // let x_off = self.t.sin() * 0.5;
                // println!("x_off: {}, t: {}", x_off, self.t);

                // let simple_triangle = [
                //     obj_parcer::Vertex { position: [ -0.5, -0.5, 0.0 ] },
                //     obj_parcer::Vertex { position: [  0.0,  0.5, 0.0 ] },
                //     obj_parcer::Vertex { position: [  0.5, -0.25, 0.0 ] },
                // ];

                let vertex_buffer =
                    glium::VertexBuffer::new(&self.display, &self.obj.vertices).unwrap();
                let indices = glium::index::IndexBuffer::new(
                    &self.display,
                    glium::index::PrimitiveType::TrianglesList,
                    &self.obj.indices,
                )
                .unwrap();

                // Translation: convert to column-major
                let translation_mat: Mat4f = Mat4f {
                    data: [
                        // column 0
                        [1.0, 0.0, 0.0, 0.0],
                        // column 1
                        [0.0, 1.0, 0.0, 0.0],
                        // column 2
                        [0.0, 0.0, 1.0, 0.0],
                        // column 3 (translation goes in rows 0..2 of column 3, w=1 at row3)
                        [0.0, 0.0, 2.0, 1.0],
                    ],
                };

                // Rotation matrices around X, Y, Z axes (use helpers)
                let rot_x = rotation_x(self.rotation_axis[0] * self.rotation_direction * self.t as f32);
                let rot_y = rotation_y(self.rotation_axis[1] * self.rotation_direction * self.t as f32);
                let rot_z = rotation_z(self.rotation_axis[2] * self.rotation_direction * self.t as f32);
                // Compose rotations: R = Rz * Ry * Rx
                let rotation_mat = mat_mul(&rot_z, &mat_mul(&rot_y, &rot_x));

                // Scale (identity here) in column-major
                let scale_mat: Mat4f = Mat4f {
                    data: [
                        [1.0, 0.0, 0.0, 0.0],
                        [0.0, 1.0, 0.0, 0.0],
                        [0.0, 0.0, 1.0, 0.0],
                        [0.0, 0.0, 0.0, 1.0],
                    ],
                };

                let aspect = (self.w_h.0 as f32) / (self.w_h.1 as f32);
                let fov: f32 = std::f32::consts::FRAC_PI_2; // 45 degrees
                let near: f32 = 0.1;
                let far: f32 = 100.0;
                // Use new perspective API (fov_y, aspect, near, far)
                let projection_mat: Mat4f = perspective(fov, aspect, near, far);

                // View matrix: translate camera along Z by -3 (column-major)
                let view_mat: Mat4f =
                    look_at(&[0.0, 0.0, -3.0], &[0.0, 0.0, 5.0], &[0.0, 1.0, 0.0]);
                let sampler = self
                    .tex
                    .sampled()
                    .wrap_function(glium::uniforms::SamplerWrapFunction::Repeat)
                    .magnify_filter(glium::uniforms::MagnifySamplerFilter::Linear);

                match &self.program {
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
                               view: view_mat.data,
                               tex: sampler,
                               bb_min: self.obj.bb[0],
                               bb_max: self.obj.bb[1],
                               shading_lerp_val: self.shading_lerp_val,
                               positions: &self.color_positions,
                            },
                            &glium::DrawParameters {
                                depth: glium::Depth {
                                    test: glium::DepthTest::IfLess,
                                    write: true,
                                    ..Default::default()
                                },

                                ..Default::default()
                            },
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
                let end_time = std::time::Instant::now();
                self.fram_time = end_time.duration_since(start_time).as_secs_f64();
            }
            _ => {
                self.window.request_redraw();
            }
        }
    }
}

// Column-major matrix multiplication: returns A * B.
// Matrices use data[column][row].
fn mat_mul(a: &Mat4f, b: &Mat4f) -> Mat4f {
    let mut m = Mat4f {
        data: [[0.0; 4]; 4],
    };
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
fn rotation_x(theta: f32) -> Mat4f {
    let (s, c) = (theta.sin(), theta.cos());
    Mat4f {
        data: [
            // column 0
            [1.0, 0.0, 0.0, 0.0],
            // column 1
            [0.0, c, s, 0.0],
            // column 2
            [0.0, -s, c, 0.0],
            // column 3
            [0.0, 0.0, 0.0, 1.0],
        ],
    }
}

fn rotation_y(theta: f32) -> Mat4f {
    let (s, c) = (theta.sin(), theta.cos());
    Mat4f {
        data: [
            // column 0
            [c, 0.0, -s, 0.0],
            // column 1
            [0.0, 1.0, 0.0, 0.0],
            // column 2
            [s, 0.0, c, 0.0],
            // column 3
            [0.0, 0.0, 0.0, 1.0],
        ],
    }
}

fn rotation_z(theta: f32) -> Mat4f {
    let (s, c) = (theta.sin(), theta.cos());
    Mat4f {
        data: [
            // column 0
            [c, s, 0.0, 0.0],
            // column 1
            [-s, c, 0.0, 0.0],
            // column 2
            [0.0, 0.0, 1.0, 0.0],
            // column 3
            [0.0, 0.0, 0.0, 1.0],
        ],
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 4 {
        println!("fuck you, give me correct parameters. asshole!");
        println!("usage: ./scop obj_path texture_path [box || sphere]");
        return;
    }
    let obj_path = &args[1];
    let texture_path = &args[2];
    let uv_algorithm = &args[3];

    if uv_algorithm != "box" && uv_algorithm != "sphere" {
        println!("you small dick, chouse eather box or sphere");
        return;
    }

    // 1. The **winit::EventLoop** for handling events.
    let event_loop = winit::event_loop::EventLoop::builder().build().unwrap();
    // 2. Create a glutin context and glium Display
    let (window, display) = glium::backend::glutin::SimpleWindowBuilder::new()
        .with_inner_size(1920, 1080)
        .with_title("scop")
        .with_vsync(false)
        .build(&event_loop);

    let w_h = display.get_framebuffer_dimensions();

    match display.set_swap_interval(SwapInterval::DontWait) {
        Ok(_) => {}
        Err(e) => {
            println!("err setting swap interval: {:?}", e);
        }
    }

    let obj = obj_parcer::Obj::read_file(&obj_path, &uv_algorithm, &texture_path);
    match obj {
        Ok(o) => {
            println!("Successfully parsed OBJ file:");
            for vertex in &o.vertices {
                println!(
                    "Vertex position: {:?}, UV: {:?}",
                    vertex.position, vertex.uv
                );
            }
            println!("Faces: {:?}", o.faces);
            println!("Indices: {:?}", o.indices);
            println!("File Name: {}", o.file_name);
            // println!("texture: {:?}", o.texture);
            println!("bb: {:?}", o.bb);
            let image = glium::texture::RawImage2d::from_raw_rgb_reversed(
                o.texture.as_slice(),
                (o.texture_width, o.texture_height),
            );

            let texture = glium::texture::Texture2d::new(&display, image).unwrap();

            let vertex_shader_src = r#"
                    #version 140
                    in vec3 position;
                    in vec2 uv;
                    out vec2 v_uv;
                    uniform mat4 rotation;
                    uniform mat4 translation;
                    uniform mat4 scale;
                    uniform mat4 projection;
                    uniform mat4 view;
                    uniform vec3 bb_min;
                    uniform vec3 bb_max;
                    layout(std140) uniform positions{
                        vec4 color_position[5];
                    };
                    flat out vec3 vertex_color;
                    
                    void main() {
                        // Note: matrices are provided column-major so no transpose is necessary.
                        // Apply model = translation * rotation * scale, then view and projection.

                        vertex_color = color_position[(gl_VertexID / 3) % 5].xyz;
                        v_uv = uv;
                        vec3 center = (bb_min + bb_max) / 2;
                        gl_Position = projection * view * translation * rotation * scale * vec4((position - center), 1.0);
                    }
                "#;

            let fragment_shader_src = r#"
                    #version 140
                    in vec2 v_uv;
                    uniform float shading_lerp_val;
                    out vec4 color;
                    flat in vec3 vertex_color;
                    uniform sampler2D tex;

                    void main() {
                        color = mix(vec4(vertex_color, 1.0), texture(tex, v_uv), shading_lerp_val);
                    }
                "#;

            let program =
                glium::Program::from_source(&display, vertex_shader_src, fragment_shader_src, None);
            let colors_uniform = glium::uniforms::UniformBuffer::new(&display,
                    ColorArr { color_position: [[0.1, 0.1, 0.1, 0.1], [0.2, 0.2, 0.2, 0.2], [0.3, 0.3, 0.3, 0.0], [0.4, 0.4, 0.4, 0.0], [0.5, 0.5, 0.5, 0.0]] }).unwrap();
            let mut app = App {
                display,
                window,
                t: 0.0,
                obj: o,
                program,
                tex: texture,
                w_h,
                shading_lerp_val: 0.0,
                set_texture: false,
                fram_time: 1.0 / 60.0,
                rotation_axis: [0.0, 1.0, 0.0],
                rotation_direction: 1.0,
                lerp_time: 0.0,
                key: PhysicalKey::Code(winit::keyboard::KeyCode::F1),
                isrepeat_key: false,
                is_key_pressed: false,
                trigger_rot_anim: (false, PhysicalKey::Code(winit::keyboard::KeyCode::F1)),
                trigger_rot_reverse: (false, 1.0, 1.0, 0.0),
                color_positions: colors_uniform,
                is_lizzard: false,
                color: ColorArr { color_position: [[0.1, 0.1, 0.1, 0.1], [0.2, 0.2, 0.2, 0.2], [0.3, 0.3, 0.3, 0.0], [0.4, 0.4, 0.4, 0.0], [0.5, 0.5, 0.5, 0.0]] },
                time_mul_delta: 0.0,
            };

            let _run = event_loop.run_app(&mut app);
            match _run {
                Ok(_) => (),
                Err(e) => println!("Error during event loop: {}", e),
            }
        }
        Err(e) => {
            println!("Error parsing OBJ file: {}", e);
            exit(1);
        }
    }
}
