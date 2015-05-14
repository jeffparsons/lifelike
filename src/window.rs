// In its current form, this is heavily inspired by:
// <https://github.com/AngryLawyer/rust-sdl2/blob/master/examples/renderer-texture.rs>

extern crate sdl2;

use self::sdl2::{
    video,
    render,
    keycode,
    event,
    timer,
    pixels,
    rect,
};

use world;

pub struct Window {
    pub width: u32,
    pub height: u32,
    pub world: world::World,
}

impl Window {
    pub fn new(world: world::World) -> Window {
        Window {
            width: world.image().width,
            height: world.image().height,
            world: world,
        }
    }

    pub fn run(&mut self) {
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

        let mut texture = renderer.create_texture_streaming(pixels::PixelFormatEnum::ABGR8888, (self.width as i32, self.height as i32)).unwrap();

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

            // Render and display the most recently calculated frame.
            self.world.update_world_image();
            texture.update(None, &self.world.image().pixel_data, 4 * self.width as i32).unwrap();
            drawer.copy(&texture, None, Some(rect::Rect::new(0, 0, self.width as i32, self.height as i32)));
            // drawer.copy_ex(&texture, None, Some(rect::Rect::new(450, 100, 256, 256)), 30.0, None, (false, false));
            drawer.present();

            // Step the world.
            self.world.step();

            // TODO: Delay by a minimum of S since start of frame--not a fixed amount.
            timer::delay(50);
        }
    }
}
