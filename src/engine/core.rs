use glfw::{Context, Key, WindowEvent, GlfwReceiver, PWindow};
use std::time::Instant;

use crate::engine::camera::Camera;
use crate::engine::input::InputState;
use crate::engine::game::Game;
use crate::engine::constants::CLEAR_COLOR;

pub struct Engine {
    pub glfw: glfw::Glfw,
    pub window: PWindow,
    pub events: GlfwReceiver<(f64, WindowEvent)>,
    pub should_close: bool,
    pub camera: Camera,
    pub time: f32,
    pub frame: u64,
    pub input: InputState,
}

impl Engine {
    pub fn new(width: u32, height: u32, title: &str) -> Self {
        let mut glfw = glfw::init(|err, desc| {
            eprintln!("GLFW Error {:?}: {}", err, desc);
        }).expect("Failed to initialize GLFW");
        glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
        glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));
        #[cfg(target_os = "macos")] {
            glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));
        }
        let (mut window, events) = glfw
            .create_window(width, height, title, glfw::WindowMode::Windowed)
            .expect("Failed to create GLFW window.");
        window.set_key_polling(true);
        window.set_cursor_pos_polling(true);
        window.set_mouse_button_polling(true);
        window.set_framebuffer_size_polling(true);
        window.make_current();
        window.set_cursor_mode(glfw::CursorMode::Disabled);

        gl::load_with(|s| window.get_proc_address(s) as *const _);
        let (fb_w, fb_h) = window.get_framebuffer_size();
        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            gl::Enable(gl::CULL_FACE);
            gl::CullFace(gl::BACK);
            gl::FrontFace(gl::CCW);
            gl::Viewport(0, 0, fb_w, fb_h);
        }

        Engine {
            glfw,
            window,
            events,
            should_close: false,
            camera: Camera::new(fb_w as f32 / fb_h as f32),
            time: 0.0,
            frame: 0,
            input: InputState::default(),
        }
    }

    pub fn run<G: Game>(&mut self, game: &mut G) {
        game.on_start(self);
        let mut last_frame = Instant::now();
        let mut first_mouse = true;
        let mut last_x = 0.0f64;
        let mut last_y = 0.0f64;

    while !self.window.should_close() && !self.should_close {
            let now = Instant::now();
            let dt = (now - last_frame).as_secs_f32();
            last_frame = now;
            self.time += dt;
            self.frame += 1;

            self.update_input_begin();
            self.process_input(dt);
            self.poll_events(&mut first_mouse, &mut last_x, &mut last_y);

            game.update(self, dt);

            unsafe {
                gl::ClearColor(CLEAR_COLOR.0, CLEAR_COLOR.1, CLEAR_COLOR.2, CLEAR_COLOR.3);
                gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            }

            game.render(self);
            self.window.swap_buffers();
        }

        game.on_shutdown(self);
    }

    fn poll_events(&mut self, first_mouse: &mut bool, last_x: &mut f64, last_y: &mut f64) {
        self.glfw.poll_events();
        for (_, event) in glfw::flush_messages(&self.events) {
            match event {
                WindowEvent::CursorPos(x, y) => {
                    if *first_mouse { *last_x = x; *last_y = y; *first_mouse = false; }
                    let dx = (x - *last_x) as f32;
                    let dy = (y - *last_y) as f32;
                    *last_x = x; *last_y = y;
                    self.camera.process_mouse(dx, dy);
                }
                WindowEvent::FramebufferSize(w, h) => unsafe {
                    gl::Viewport(0, 0, w, h);
                    if h > 0 { self.camera.aspect = w as f32 / h as f32; }
                },
                WindowEvent::Key(key, _, action, _) => {
                    self.input.key_event(key, action);
                }
                _ => {}
            }
        }
    }

    fn update_input_begin(&mut self) { self.input.begin_frame(); }

    fn process_input(&mut self, dt: f32) {
        let cam_speed_scale = if self.input.is_key_down(Key::LeftShift) { 2.0 } else { 1.0 };
        if self.input.is_key_down(Key::W) { self.camera.process_keyboard(crate::engine::camera::CameraMove::Forward, dt * cam_speed_scale); }
        if self.input.is_key_down(Key::S) { self.camera.process_keyboard(crate::engine::camera::CameraMove::Backward, dt * cam_speed_scale); }
        if self.input.is_key_down(Key::A) { self.camera.process_keyboard(crate::engine::camera::CameraMove::Left, dt * cam_speed_scale); }
        if self.input.is_key_down(Key::D) { self.camera.process_keyboard(crate::engine::camera::CameraMove::Right, dt * cam_speed_scale); }
        if self.input.is_key_down(Key::Space) { self.camera.process_keyboard(crate::engine::camera::CameraMove::Up, dt * cam_speed_scale); }
        if self.input.is_key_down(Key::LeftControl) { self.camera.process_keyboard(crate::engine::camera::CameraMove::Down, dt * cam_speed_scale); }
        if self.input.was_key_pressed(Key::Escape) { self.window.set_should_close(true); }
    }
}
