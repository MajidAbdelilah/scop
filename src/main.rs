
use glium::{Surface, glutin::surface::WindowSurface, winit::{self, application::ApplicationHandler, window::Window}};

mod structs;
use structs::Vertex;

pub struct App{
    display: glium::Display<WindowSurface>,
    _window: Window,
}

impl ApplicationHandler for App{
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
                glium::winit::event::WindowEvent::RedrawRequested => {
                    let mut target = self.display.draw();
                    target.clear_color(1.0, 1.0, 1.0, 1.0);
                    let shape = vec![
                        Vertex {position: [0.0, 0.5]},
                        Vertex {position: [-0.5, -0.5]},
                        Vertex {position: [0.5, -0.5]},
                    ];
                    let vertex_buffer = glium::VertexBuffer::new(&self.display, &shape).unwrap();
                    let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);
                    let vertex_shader_src = r#"
                        #version 140
                        in vec2 position;

                        void main() {
                            gl_Position = vec4(position, 0.0, 1.0);
                        }
                    "#;

                    let fragment_shader_src = r#"
                        #version 140

                        out vec4 color;

                        void main() {
                            color = vec4(1.0, 0.0, 0.0, 1.0);
                        }
                    "#;

                    let program = glium::Program::from_source(&self.display, vertex_shader_src, fragment_shader_src, None);
                    match program {
                        Ok(program) => {
                            let drawing = target.draw(&vertex_buffer, &indices, &program, &glium::uniforms::EmptyUniforms, &Default::default());
                            match drawing {
                                Ok(_) => {
                                    target.finish().unwrap();
                                }
                                Err(e) => {
                                    println!("Failed to draw: {}", e);
                                    return ;
                                }
                            }
                        }
                        Err(e) => {
                            println!("Failed to compile shader program: {}", e);
                            return ;
                        }
                    }
                }
                _ => (),
        }
    }
}
fn main() {
    
    // 1. The **winit::EventLoop** for handling events.
    let event_loop = winit::event_loop::EventLoop::builder().build().unwrap();
    // 2. Create a glutin context and glium Display
    let (window, display) = glium::backend::glutin::SimpleWindowBuilder::new().build(&event_loop);
    
    let mut app = App{
        display,
        _window: window,
    };

    let _run = event_loop.run_app(&mut app);
    match _run {
        Ok(_) => (),
        Err(e) => println!("Error during event loop: {}", e),
    }
    
}
