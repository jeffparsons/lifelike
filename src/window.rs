// In its current form, this is heavily inspired by:
// <https://github.com/AngryLawyer/rust-sdl2/blob/master/examples/renderer-texture.rs>

extern crate sdl2;

use self::sdl2::{video, render, keycode, event};

pub struct Window {
    pub width: u32,
    pub height: u32,
}

impl Window {
    pub fn new(width: u32, height: u32) -> Window {
        Window {
            width: width,
            height: height,
        }
    }

    pub fn run(&self) {
        let sdl_context = sdl2::init(sdl2::INIT_VIDEO).unwrap();

        let window = video::Window::new(
            &sdl_context,
            "lifelike",
            video::WindowPos::PosCentered,
            video::WindowPos::PosCentered,
            self.width as i32,
            self.height as i32,
            video::OPENGL
        ).unwrap();

        let mut renderer = render::Renderer::from_window(
            window,
            render::RenderDriverIndex::Auto,
            render::ACCELERATED
        ).unwrap();

        let mut drawer = renderer.drawer();
        drawer.set_draw_color(sdl2::pixels::Color::RGB(0, 0, 0));
        drawer.clear();
        drawer.present();

        let mut running = true;
        let mut event_pump = sdl_context.event_pump();

        while running {
            for event in event_pump.poll_iter() {
                match event {
                    event::Event::Quit {..} | event::Event::KeyDown { keycode: keycode::KeyCode::Escape, .. } => {
                        running = false
                    },
                    _ => {}
                }
            }

            // ...

        }
    }
}
