extern crate glfw;
use glfw::{Action, Context, Key, MouseButton, WindowEvent};
use crate::engine::camera::{Camera, CameraMove, CameraUniform};
// use glam::Mat4;
use std::mem;
use std::ptr;
use std::time::Instant;

// OpenGL function loader
use gl::types::*;

static VERT_SRC: &str = r"#version 330 core
layout (location = 0) in vec3 aPos;
layout (location = 1) in vec3 aColor;
uniform mat4 uViewProj;
out vec3 vColor;
void main() {
	vColor = aColor;
	gl_Position = uViewProj * vec4(aPos, 1.0);
}";

static FRAG_SRC: &str = r"#version 330 core
in vec3 vColor;
out vec4 FragColor;
void main() { FragColor = vec4(vColor, 1.0); }";

pub fn run() {
	let mut glfw = glfw::init(|err, desc| {
		eprintln!("GLFW Error {:?}: {}", err, desc);
	}).expect("Failed to initialize GLFW");
	glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
	glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));
	#[cfg(target_os = "macos")] {
		glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));
	}

	let (mut window, events) = glfw
		.create_window(800, 600, "Oxidize", glfw::WindowMode::Windowed)
		.expect("Failed to create GLFW window.");
	window.set_key_polling(true);
	window.set_cursor_pos_polling(true);
	window.set_mouse_button_polling(true);
	window.set_framebuffer_size_polling(true);
	window.make_current();
	window.set_cursor_mode(glfw::CursorMode::Disabled);

	gl::load_with(|s| window.get_proc_address(s) as *const _);

	unsafe {
		gl::Enable(gl::DEPTH_TEST);
		gl::Viewport(0, 0, 800, 600);
	}

	// Cube data
	#[rustfmt::skip]
	let vertices: [f32; 6*6*6] = [
		// positions        // colors
		// Front face (two triangles)
		-0.5,-0.5, 0.5, 1.0,0.0,0.0,
		 0.5,-0.5, 0.5, 0.0,1.0,0.0,
		 0.5, 0.5, 0.5, 0.0,0.0,1.0,
		-0.5,-0.5, 0.5, 1.0,0.0,0.0,
		 0.5, 0.5, 0.5, 0.0,0.0,1.0,
		-0.5, 0.5, 0.5, 1.0,1.0,0.0,
		// Back face
		-0.5,-0.5,-0.5, 1.0,0.0,0.0,
		 0.5, 0.5,-0.5, 0.0,0.0,1.0,
		 0.5,-0.5,-0.5, 0.0,1.0,0.0,
		-0.5,-0.5,-0.5, 1.0,0.0,0.0,
		-0.5, 0.5,-0.5, 1.0,1.0,0.0,
		 0.5, 0.5,-0.5, 0.0,0.0,1.0,
		// Left
		-0.5,-0.5,-0.5, 1.0,0.0,0.0,
		-0.5,-0.5, 0.5, 0.0,1.0,0.0,
		-0.5, 0.5, 0.5, 0.0,0.0,1.0,
		-0.5,-0.5,-0.5, 1.0,0.0,0.0,
		-0.5, 0.5, 0.5, 0.0,0.0,1.0,
		-0.5, 0.5,-0.5, 1.0,1.0,0.0,
		// Right
		 0.5,-0.5,-0.5, 1.0,0.0,0.0,
		 0.5, 0.5, 0.5, 0.0,0.0,1.0,
		 0.5,-0.5, 0.5, 0.0,1.0,0.0,
		 0.5,-0.5,-0.5, 1.0,0.0,0.0,
		 0.5, 0.5,-0.5, 1.0,1.0,0.0,
		 0.5, 0.5, 0.5, 0.0,0.0,1.0,
		// Top
		-0.5, 0.5, 0.5, 1.0,0.0,0.0,
		 0.5, 0.5, 0.5, 0.0,1.0,0.0,
		 0.5, 0.5,-0.5, 0.0,0.0,1.0,
		-0.5, 0.5, 0.5, 1.0,0.0,0.0,
		 0.5, 0.5,-0.5, 0.0,0.0,1.0,
		-0.5, 0.5,-0.5, 1.0,1.0,0.0,
		// Bottom
		-0.5,-0.5, 0.5, 1.0,0.0,0.0,
		 0.5,-0.5,-0.5, 0.0,0.0,1.0,
		 0.5,-0.5, 0.5, 0.0,1.0,0.0,
		-0.5,-0.5, 0.5, 1.0,0.0,0.0,
		-0.5,-0.5,-0.5, 1.0,1.0,0.0,
		 0.5,-0.5,-0.5, 0.0,0.0,1.0,
	];
    let (mut vbo, mut vao) = (0, 0);
    unsafe {
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
        let stride = (6 * mem::size_of::<f32>()) as GLsizei;
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, stride, ptr::null());
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(1, 3, gl::FLOAT, gl::FALSE, stride, (3 * mem::size_of::<f32>()) as *const _);
        gl::EnableVertexAttribArray(1);
    }

    let shader_program = unsafe { create_program(VERT_SRC, FRAG_SRC) };
    let mut first_mouse = true;
    let mut last_x = 400.0f64;
    let mut last_y = 300.0f64;
    let mut camera = Camera::new(800.0 / 600.0);
    let mut last_frame = Instant::now();

    while !window.should_close() {
        let now = Instant::now();
        let delta = (now - last_frame).as_secs_f32();
        last_frame = now;

        process_input(&mut window, &mut camera, delta);

        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            handle_event(&mut window, event, &mut first_mouse, &mut last_x, &mut last_y, &mut camera);
        }

        unsafe {
            gl::ClearColor(0.1, 0.1, 0.15, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            gl::UseProgram(shader_program);
            let cam_uni = CameraUniform::new(&camera);
            let loc = gl::GetUniformLocation(shader_program, b"uViewProj\0".as_ptr() as *const _);
            gl::UniformMatrix4fv(loc, 1, gl::FALSE, cam_uni.view_proj.to_cols_array().as_ptr());
            gl::BindVertexArray(vao);
            gl::DrawArrays(gl::TRIANGLES, 0, (vertices.len() / 6) as i32);
        }

        window.swap_buffers();
    }
}

fn process_input(window: &mut glfw::Window, camera: &mut Camera, delta: f32) {
	if window.get_key(Key::W) == Action::Press { camera.process_keyboard(CameraMove::Forward, delta); }
	if window.get_key(Key::S) == Action::Press { camera.process_keyboard(CameraMove::Backward, delta); }
	if window.get_key(Key::A) == Action::Press { camera.process_keyboard(CameraMove::Left, delta); }
	if window.get_key(Key::D) == Action::Press { camera.process_keyboard(CameraMove::Right, delta); }
	if window.get_key(Key::Space) == Action::Press { camera.process_keyboard(CameraMove::Up, delta); }
	if window.get_key(Key::LeftShift) == Action::Press { camera.process_keyboard(CameraMove::Down, delta); }
	if window.get_key(Key::Escape) == Action::Press { window.set_should_close(true); }
}

fn handle_event(window: &mut glfw::Window, event: WindowEvent, first_mouse: &mut bool, last_x: &mut f64, last_y: &mut f64, camera: &mut Camera) {
	match event {
		WindowEvent::CursorPos(x, y) => {
			if *first_mouse { *last_x = x; *last_y = y; *first_mouse = false; }
			let dx = (x - *last_x) as f32;
			let dy = (y - *last_y) as f32;
			*last_x = x; *last_y = y;
			camera.process_mouse(dx, dy);
		}
		WindowEvent::FramebufferSize(w, h) => unsafe { 
			gl::Viewport(0, 0, w, h); 
			if h > 0 { camera.aspect = w as f32 / h as f32; }
		},
		WindowEvent::MouseButton(MouseButton::Button1, Action::Press, _) => { window.set_cursor_mode(glfw::CursorMode::Disabled); },
		_ => {}
	}
}

unsafe fn compile_shader(src: &str, ty: GLenum) -> u32 {
	let shader = gl::CreateShader(ty);
	let c_str = std::ffi::CString::new(src).unwrap();
	gl::ShaderSource(shader, 1, &c_str.as_ptr(), ptr::null());
	gl::CompileShader(shader);
	let mut success = 0;
	gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);
	if success == 0 {
		let mut len = 0; gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
		let mut buf = Vec::with_capacity(len as usize);
		buf.set_len((len as usize).saturating_sub(1));
		gl::GetShaderInfoLog(shader, len, ptr::null_mut(), buf.as_mut_ptr() as *mut _);
		panic!("Shader compile error: {}", String::from_utf8_lossy(&buf));
	}
	shader
}

unsafe fn create_program(vs_src: &str, fs_src: &str) -> u32 {
	let vs = compile_shader(vs_src, gl::VERTEX_SHADER);
	let fs = compile_shader(fs_src, gl::FRAGMENT_SHADER);
	let program = gl::CreateProgram();
	gl::AttachShader(program, vs);
	gl::AttachShader(program, fs);
	gl::LinkProgram(program);
	gl::DeleteShader(vs);
	gl::DeleteShader(fs);
	let mut success = 0;
	gl::GetProgramiv(program, gl::LINK_STATUS, &mut success);
	if success == 0 {
		let mut len = 0; gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
		let mut buf = Vec::with_capacity(len as usize);
		buf.set_len((len as usize).saturating_sub(1));
		gl::GetProgramInfoLog(program, len, ptr::null_mut(), buf.as_mut_ptr() as *mut _);
		panic!("Program link error: {}", String::from_utf8_lossy(&buf));
	}
	program
}
