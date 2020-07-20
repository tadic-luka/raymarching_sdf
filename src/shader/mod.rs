use gl::types::*;
use std::ptr;
use std::ffi::CString;

const GLSL_VERSION: &'static [u8] = b"#version 310 es\nprecision highp float;\n\0";

const VERTEX_SHADER_SOURCE: &'static str = include_str!("../../assets/shaders/main.vert");
const FRAGMENT_SHADER_SOURCE: &'static str = include_str!("../../assets/shaders/main.frag");

#[macro_export]
macro_rules! gen_uniforms {
    (
        $struct:ident,
        $(
            $name:ident, $fn_name:ident => $val:tt
        ),*
    ) => {

        #[derive(Clone, Copy, Debug)]
        struct Locs {
            $(
                $name: GLint,
            )*
        }

        impl Locs {
            fn get(prog: GLuint) -> Self {
                Locs {
                $(
                    $name:  unsafe { 
                        gl::GetUniformLocation(prog, concat!(stringify!($name), "\0").as_bytes().as_ptr() as _)
                    },
                )*
                }
            }
        }

        impl $struct {
            $(
                pub fn $fn_name(&self, $name: $val) {
                    let _tmp_prog = self.use_program();
                    send_uniform_to_gl!(self.locs.$name, $val, $name);
                }
            )*
        }

    }
}

macro_rules! send_uniform_to_gl {
    ($loc:expr, [f32; 3], $value:ident) => {
        let [x, y, z] = $value;
        unsafe {
            gl::Uniform3f($loc, x, y, z);
        }
    };
    ($loc:expr, [f32; 2], $value:ident) => {
        let [x, y] = $value;
        unsafe {
            gl::Uniform2f($loc, x, y);
        }
    };
    ($loc:expr, f32, $value:ident) => {
        unsafe {
            gl::Uniform1f($loc, $value);
        }
    };
    ($loc:expr, bool, $value:ident) => {
        let val = $value as _ ;
        unsafe {
            gl::Uniform1i($loc, val);
        }
    };
    ($loc:expr, i32, $value:ident) => {
        let val = $value as _ ;
        unsafe {
            gl::Uniform1i($loc, val);
        }
    }
}

gen_uniforms!(
    Program,
    resolution, set_resolution => [f32; 2],
    obj, set_obj => i32,
    time, set_time => f32,
    shininess, set_shininess => f32,
    camera_eye, set_camera_eye => [f32; 3]
);


pub struct TmpProgram {
    last_program: GLuint
}

impl Drop for TmpProgram {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe {
            gl::UseProgram(self.last_program);
        }
    }
}

pub struct Program {
    program: GLuint,
    locs: Locs,
}

impl Drop for Program {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.program);
        }
    }
}

impl Program {
    pub fn new() -> Result<Self, String> {
        compile_shader::<VertShader>(VERTEX_SHADER_SOURCE)
            .and_then(|vshader| compile_shader::<FragShader>(FRAGMENT_SHADER_SOURCE).map(|fshader| (vshader, fshader)))
            .and_then(|(vshader, fshader)| create_shader_program(&[vshader, fshader]))
            .map(|program| {
                let prog = Program {
                    locs: Locs::get(program),
                    program,
                };
                prog
            })
    }
    


    #[inline]
    pub fn use_program(&self) -> TmpProgram {
        let mut last_program: GLint = 0;
        unsafe {
            gl::GetIntegerv(gl::CURRENT_PROGRAM, &mut last_program);
            gl::UseProgram(self.program);
        }
        TmpProgram {
            last_program: last_program  as GLuint,
        }
    }
}

struct VertShader{}
struct FragShader{}

trait Shader {
    fn shader_type() -> GLenum;
}
impl Shader for VertShader {
    fn shader_type() -> GLenum {
        gl::VERTEX_SHADER
    }
}
impl Shader for FragShader {
    fn shader_type() -> GLenum {
        gl::FRAGMENT_SHADER
    }
}

#[derive(Copy, Clone)]
enum GlObj {
    Shader,
    Program,
}

impl GlObj {
    fn get_info_log(&self, obj:  GLuint, buf_size: GLsizei, length: *mut GLsizei, source: *mut GLchar) {
        match self {
            GlObj::Shader => unsafe { gl::GetShaderInfoLog(obj, buf_size, length, source) },
            GlObj::Program => unsafe { gl::GetProgramInfoLog(obj, buf_size, length, source) },
        }
    }
}

fn get_gl_error(value: GLuint, variant: GlObj) -> Option<String>
where
{
    unsafe {
        let mut success = gl::FALSE as GLint;
        match variant {
            GlObj::Shader => gl::GetShaderiv(value, gl::COMPILE_STATUS, &mut success),
            GlObj::Program => gl::GetProgramiv(value, gl::LINK_STATUS, &mut success),
        }

        if success == gl::FALSE as GLint {
            let  mut max_len: GLint = 0;
            match variant {
                GlObj::Shader => gl::GetShaderiv(value, gl::INFO_LOG_LENGTH, &mut max_len),
                GlObj::Program => gl::GetProgramiv(value, gl::INFO_LOG_LENGTH, &mut max_len),
            }
            let mut info_log = vec![0; max_len as usize];
            variant.get_info_log(
                value,
                max_len as _,
                ptr::null_mut(),
                info_log.as_mut_ptr() as *mut GLchar
            );
            info_log.pop();
            return Some(format!(
                "ERROR::SHADER::VERTEX::COMPILATION_FAILED\n{}",
                std::str::from_utf8(&info_log).unwrap()
            ));
        }
        None
    }
}

fn compile_shader<S: Shader>(shader_source: &str) -> Result<GLuint, String> {
    unsafe {
        let shader_source = CString::new(shader_source).unwrap();
        let shader = gl::CreateShader(S::shader_type());
        let sources = [
            GLSL_VERSION.as_ptr() as *const GLchar,
            shader_source.as_ptr() as *const GLchar,
        ];
        let sources_len = [
            GLSL_VERSION.len() as GLint -1,
            shader_source.as_bytes().len() as GLint -1 
        ];
        gl::ShaderSource(shader, 2, sources.as_ptr(), sources_len.as_ptr());
        gl::CompileShader(shader);
        match get_gl_error(shader, GlObj::Shader) {
            Some(err) => Err(err),
            None => Ok(shader)
        }
    }
}

fn create_shader_program(shaders: &[GLuint]) -> Result<GLuint, String> {
    unsafe {
        let prog = gl::CreateProgram();
        for shader in shaders {
            gl::AttachShader(prog, *shader);
        }
        gl::LinkProgram(prog);
        match get_gl_error(prog, GlObj::Program) {
            Some(err) => Err(err),
            None => Ok(prog),
        }
    }
}

