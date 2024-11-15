use sdl2::{
    event::Event, keyboard::Keycode, pixels::PixelFormatEnum, render::Texture,
    render::TextureCreator,
};
use std::cell::RefCell;
use std::{thread::sleep, time::Duration};

type Error = Box<dyn std::error::Error>;

#[allow(dead_code)]
pub struct Canvas {
    sdl_context: sdl2::Sdl,
    sdl_canvas: sdl2::render::Canvas<sdl2::video::Window>,
    creator: TextureCreator<sdl2::video::WindowContext>,
    texture: RefCell<Texture<'static>>,
    data: Vec<u32>,
    width: usize,
    height: usize,
}
impl Canvas {
    pub fn new(title: &str, width: usize, height: usize, scale: usize) -> Result<Self, Error> {
        // std::env::set_var("DBUS_FATAL_WARNINGS","0");
        let sdl_context = sdl2::init()?;
        let video_subsystem = sdl_context.video()?;
        let window = video_subsystem
            .window(title, (width * scale) as u32, (height * scale) as u32)
            .position_centered()
            .build()?;
        let sdl_canvas = window.into_canvas().build()?;
        let creator = sdl_canvas.texture_creator();
        let texture = creator.create_texture_target(
            PixelFormatEnum::RGBA8888,
            width as u32,
            height as u32,
        )?;

        let texture = unsafe { std::mem::transmute::<_, Texture<'static>>(texture) };

        Ok(Canvas {
            width,
            height,
            data: vec![0xff; (width * height) as usize],
            sdl_canvas,
            sdl_context,
            creator,
            texture: RefCell::new(texture),
        })
    }

    pub fn present(&mut self) {
        self.sdl_canvas.present();
    }
    pub fn flush(&mut self) {
        let mut texture = self.texture.borrow_mut();
        texture
            .update(None, self.data_raw(), (self.width * 4) as usize)
            .unwrap();
        self.sdl_canvas.copy(&texture, None, None).unwrap();
        self.sdl_canvas.present();
    }
    pub fn draw_pixel_rgba(&mut self, x: usize, y: usize, r: u8, g: u8, b: u8, a: u8) {
        let color = u32::from_be_bytes([r, g, b, a]);
        self.data[y * self.width + x] = color;
    }
    pub fn draw_pixel(&mut self, x: usize, y: usize, color: u32) {
        self.data[y * self.width + x] = color;
    }
    fn data_raw(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.data.as_ptr() as *const u8, self.data.len() * 4) }
    }
    pub fn wait(&self) {
        let duration = Duration::from_millis(100);
        let mut event_pump = self.sdl_context.event_pump().unwrap();
        'running: loop {
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => break 'running,
                    _ => {}
                }
            }
            sleep(duration);
        }
    }
}
