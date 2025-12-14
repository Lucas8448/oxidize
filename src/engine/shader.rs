use gl::types::*;
use std::ptr;

pub struct ShaderProgram { pub id: u32 }

impl ShaderProgram {
    pub unsafe fn from_source(vertex_src: &str, fragment_src: &str) -> Result<Self, String> {
        let vs = compile_shader(vertex_src, gl::VERTEX_SHADER)?;
        let fs = compile_shader(fragment_src, gl::FRAGMENT_SHADER)?;
        let program = gl::CreateProgram();
        gl::AttachShader(program, vs);
        gl::AttachShader(program, fs);
        gl::LinkProgram(program);
        gl::DeleteShader(vs);
        gl::DeleteShader(fs);
        let mut success = 0;
        gl::GetProgramiv(program, gl::LINK_STATUS, &mut success);
        if success == 0 {
            let mut len = 0;
            gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
            let mut buf = vec![0u8; len as usize];
            gl::GetProgramInfoLog(program, len, ptr::null_mut(), buf.as_mut_ptr() as *mut _);
            buf.truncate(buf.iter().position(|&c| c == 0).unwrap_or(buf.len()));
            return Err(String::from_utf8_lossy(&buf).to_string());
        }
        Ok(Self { id: program })
    }
    pub unsafe fn use_program(&self) { gl::UseProgram(self.id); }
    pub unsafe fn set_mat4(&self, name: &str, mat: &glam::Mat4) {
        let cname = std::ffi::CString::new(name).unwrap();
        let loc = gl::GetUniformLocation(self.id, cname.as_ptr());
        gl::UniformMatrix4fv(loc, 1, gl::FALSE, mat.to_cols_array().as_ptr());
    }
    pub unsafe fn set_vec3(&self, name: &str, v: &glam::Vec3) {
        let cname = std::ffi::CString::new(name).unwrap();
        let loc = gl::GetUniformLocation(self.id, cname.as_ptr());
        gl::Uniform3f(loc, v.x, v.y, v.z);
    }
    pub unsafe fn set_int(&self, name: &str, value: i32) {
        let cname = std::ffi::CString::new(name).unwrap();
        let loc = gl::GetUniformLocation(self.id, cname.as_ptr());
        gl::Uniform1i(loc, value);
    }
}

impl Drop for ShaderProgram {
    fn drop(&mut self) { unsafe { gl::DeleteProgram(self.id); } }
}

unsafe fn compile_shader(src: &str, ty: GLenum) -> Result<u32, String> {
    let shader = gl::CreateShader(ty);
    let c_str = std::ffi::CString::new(src).unwrap();
    gl::ShaderSource(shader, 1, &c_str.as_ptr(), ptr::null());
    gl::CompileShader(shader);
    let mut success = 0;
    gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);
    if success == 0 {
        let mut len = 0;
        gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
        let mut buf = vec![0u8; len as usize];
        gl::GetShaderInfoLog(shader, len, ptr::null_mut(), buf.as_mut_ptr() as *mut _);
        buf.truncate(buf.iter().position(|&c| c == 0).unwrap_or(buf.len()));
        return Err(String::from_utf8_lossy(&buf).to_string());
    }
    Ok(shader)
}
