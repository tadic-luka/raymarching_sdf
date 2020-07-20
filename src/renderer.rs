use crate::shader::Program;
use gl::types::*;
use std::{mem, ptr};
use std::ffi::c_void;
use std::time::Instant;
use crate::State;
use glutin::{
    dpi::PhysicalSize, 
    WindowedContext, 
    PossiblyCurrent
};

struct Quad {
    vao: GLuint,
    vbo: GLuint,
}

pub struct Renderer {
    quad: Quad,
    prog: Program,
    render_start: Instant,
}

impl Renderer {
    pub fn new(ctx: &WindowedContext<PossiblyCurrent>) -> Self {
        gl::load_with(|name| ctx.get_proc_address(name) as *const _);
        let prog = Program::new().unwrap();
        let _tmp_prog = prog.use_program();
        let quad = Quad::new();
        let start = std::time::Instant::now();
        Self {
            prog,
            quad,
            render_start: start,
        }
    }

    pub fn update_state(&self, state: &State) {
        self.prog.set_camera_eye(state.camera_eye);
        self.prog.set_shininess(state.shininess);
        self.prog.set_obj(state.obj as _);
        self.prog.set_time(self.render_start.elapsed().as_secs_f32());
    }

    pub fn render(&self) {
        let _tmp_prog = self.prog.use_program();
        unsafe {
            gl::ClearColor(0.4, 0.4, 0.4, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
        self.quad.render();
    }
    
    pub fn set_resolution(&self, size: PhysicalSize<u32>) {
        unsafe {
            gl::Viewport(0, 0, size.width as _, size.height as _);
        };
        self.prog.set_resolution([size.width as f32, size.height as f32]);
    }

}



impl Quad {
    fn new() -> Self {
        let (mut vbo, mut vao) = (0, 0);

        let vertices = [
            -1.0f32, -1.0, 0.0,
            -1.0, 1.0, 0.0,
            1.0, -1.0, 0.0,

            1.0, -1.0, 0.0,
            -1.0, 1.0, 0.0,
            1.0, 1.0, 0.0,
        ];
        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vbo);
            // bind the Vertex Array Object first, then bind and set vertex buffer(s), and then configure vertex attributes(s).
            gl::BindVertexArray(vao);
            {
                gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
                {
                    gl::BufferData(
                        gl::ARRAY_BUFFER,
                        (vertices.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
                        &vertices[0] as *const f32 as *const c_void,
                        gl::STATIC_DRAW,
                    );
                    gl::VertexAttribPointer(
                        0,
                        3,
                        gl::FLOAT,
                        gl::FALSE,
                        3 * mem::size_of::<GLfloat>() as GLsizei,
                        ptr::null(),
                    );
                    gl::EnableVertexAttribArray(0);
                }
                // note that this is allowed, the call to gl::VertexAttribPointer registered VBO as the vertex attribute's bound vertex buffer object so afterwards we can safely unbind
                gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            }
            gl::BindVertexArray(0);

        }
        Self {
            vbo, 
            vao,
        }
    }

    fn render(&self) {
        unsafe {
            gl::BindVertexArray(self.vao);
            gl::DrawArrays(gl::TRIANGLES, 0, 6);
        }

    }
}


impl Drop for Quad {
  fn drop(&mut self) {
    unsafe {
      gl::DeleteBuffers(1, &self.vbo);
      gl::DeleteBuffers(1, &self.vao);
    }
  }
}
