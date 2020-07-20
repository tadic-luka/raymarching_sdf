mod shader;
mod imgui_opengl_renderer;
mod renderer;
mod ui;

use glutin::event::{Event, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::WindowBuilder;
use glutin::{Api, ContextBuilder, GlRequest};
use renderer::Renderer;
use ui::Ui;

#[derive(Copy, Clone, PartialEq)]
#[repr(i32)]
pub enum Obj {
    Cube = 0,
    Torus = 1,
    Sphere = 2,
}

pub struct State {
    pub camera_eye: [f32; 3],
    pub shininess: f32,
    pub obj: Obj,
}

fn main() {
    let el = EventLoop::new();
    let wb = WindowBuilder::new()
        .with_title("A fantastic window!")
        .with_inner_size(glutin::dpi::LogicalSize::new(1366f64, 768f64));

    let windowed_context = ContextBuilder::new()
        .with_gl(GlRequest::Specific(Api::OpenGlEs, (3, 1)))
        .build_windowed(wb, &el).unwrap();

    let windowed_context = unsafe { windowed_context.make_current().unwrap() };

    println!(
        "Pixel format of the window's GL context: {:?}",
        windowed_context.get_pixel_format()
    );

    let mut state = State {
        camera_eye: [0.0, 0.0, 15.0],
        shininess: 10.0,
        obj: Obj::Sphere,
    };
    let renderer = Renderer::new(&windowed_context);
    let mut ui = Ui::new(&windowed_context);
    let mut curr_time = std::time::Instant::now(); 
    let mut last_frame = std::time::Instant::now();
    let max_dur_per_frame = std::time::Duration::from_micros(1_000_000 / 60); 
    el.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::WaitUntil(std::time::Instant::now() + max_dur_per_frame);
        ui.platform.handle_event(ui.imgui.io_mut(), windowed_context.window(), &event);

        match event {
            Event::LoopDestroyed => return,
            Event::NewEvents(_) => {
                last_frame = ui.imgui.io_mut().update_delta_time(last_frame);
            },
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(physical_size) => {
                    windowed_context.resize(physical_size);
                    renderer.set_resolution(physical_size);
                    windowed_context.window().request_redraw();
                }
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit
                }
                _ => (),
            },
            Event::MainEventsCleared => {
                windowed_context.window().request_redraw();
            },
            Event::RedrawRequested(_) => {
                renderer.update_state(&state);
                renderer.render();
                ui.render(&mut state, windowed_context.window());
                windowed_context.swap_buffers().unwrap();
                 // Limit the fps if needed.
                let elapsed = curr_time.elapsed();
                if elapsed < max_dur_per_frame {
                    std::thread::sleep(max_dur_per_frame - elapsed);
                }
                curr_time = std::time::Instant::now();
            }
            _ => (),
        }
    });
}
