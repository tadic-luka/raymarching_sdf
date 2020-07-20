use imgui_winit_support::{HiDpiMode, WinitPlatform};
use crate::imgui_opengl_renderer::Renderer as ImguiRenderer;
use super::{State, Obj};
use glutin::{
    window::Window, 
    WindowedContext, 
    PossiblyCurrent
};

pub struct Ui {
    pub imgui: imgui::Context,
    pub platform: WinitPlatform,
    pub renderer: ImguiRenderer,
}

impl Ui {
    pub fn new(ctx: &WindowedContext<PossiblyCurrent>) -> Self {
        let mut imgui = imgui::Context::create();
        imgui.set_ini_filename(None);
        let renderer = ImguiRenderer::new(&mut imgui, |s| ctx.get_proc_address(s) as _);
        let mut platform = WinitPlatform::init(&mut imgui);
        platform.attach_window(imgui.io_mut(), ctx.window(), HiDpiMode::Default);
        Self {
            imgui,
            renderer,
            platform,
        }
    }

    pub fn render(&mut self, state: &mut State, window: &Window) {
        let ui = self.imgui.frame();
        imgui::Window::new(imgui::im_str!("Render settings"))
            .size([300.0, 100.0], imgui::Condition::FirstUseEver)
            .position([0.0, 0.0], imgui::Condition::FirstUseEver)
            .build(&ui, || {
                ui.drag_float3(imgui::im_str!("camera pos"), &mut state.camera_eye)
                    .speed(0.01)
                    .build();
                ui.drag_float(imgui::im_str!("shininess"), &mut state.shininess)
                    .min(0.0)
                    .speed(0.01)
                    .build();
                {
                    ui.text(imgui::im_str!("Select object!"));
                    ui.separator();
                    ui.radio_button(
                        imgui::im_str!("Sphere"),
                        &mut state.obj,
                        Obj::Sphere,
                    );
                    ui.radio_button(
                        imgui::im_str!("Cube"),
                        &mut state.obj,
                        Obj::Cube,
                    );
                    ui.radio_button(
                        imgui::im_str!("Torus"),
                        &mut state.obj,
                        Obj::Torus,
                    );
                }
            });
        self.platform.prepare_render(&ui, window);
        self.renderer.render(ui);
    }

}
