use gl::types::*;
use std::mem;
use std::ptr;

pub struct Mesh { pub vao: u32, vbo: u32, count: i32 }

impl Mesh {
    pub fn from_vertices(vertices: &[f32]) -> Self {
        assert!(vertices.len() % 9 == 0, "vertex slice must be multiple of 9 (pos3+normal3+color3)");
        unsafe {
            let (mut vbo, mut vao) = (0, 0);
            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vbo);
            gl::BindVertexArray(vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (vertices.len() * mem::size_of::<f32>()) as isize,
                vertices.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );
            let stride = (9 * mem::size_of::<f32>()) as GLsizei;
            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, stride, ptr::null());
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(1, 3, gl::FLOAT, gl::FALSE, stride, (3 * mem::size_of::<f32>()) as *const _);
            gl::EnableVertexAttribArray(1);
            gl::VertexAttribPointer(2, 3, gl::FLOAT, gl::FALSE, stride, (6 * mem::size_of::<f32>()) as *const _);
            gl::EnableVertexAttribArray(2);
            Mesh { vao, vbo, count: (vertices.len() / 9) as i32 }
        }
    }

    pub unsafe fn draw(&self) {
        gl::BindVertexArray(self.vao);
        gl::DrawArrays(gl::TRIANGLES, 0, self.count);
    }
}

impl Drop for Mesh {
    fn drop(&mut self) { unsafe { gl::DeleteVertexArrays(1, &self.vao); gl::DeleteBuffers(1, &self.vbo); } }
}
