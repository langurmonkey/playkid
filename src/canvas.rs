use sdl2::{pixels::PixelFormatEnum, render::Texture, render::TextureCreator, Sdl};
use std::cell::RefCell;

type Error = Box<dyn std::error::Error>;

/// Controls the canvas and the actual rendering.
/// See https://users.rust-lang.org/t/rust-sdl2-and-raw-textures-help/45636/25
#[allow(dead_code)]
pub struct Canvas<'a> {
    sdl: &'a Sdl,
    sdl_canvas: sdl2::render::Canvas<sdl2::video::Window>,
    creator: TextureCreator<sdl2::video::WindowContext>,
    texture: RefCell<Texture<'static>>,
    data: Vec<u8>,
    width: usize,
    height: usize,
}
impl<'a> Canvas<'a> {
    pub fn new(
        sdl: &'a Sdl,
        title: &str,
        width: usize,
        height: usize,
        scale: usize,
    ) -> Result<Self, Error> {
        let video_subsystem = sdl.video()?;
        let window = video_subsystem
            .window(title, (width * scale) as u32, (height * scale) as u32)
            .position_centered()
            .build()?;
        let sdl_canvas = window.into_canvas().build()?;
        let creator = sdl_canvas.texture_creator();
        let texture = creator.create_texture_target(
            PixelFormatEnum::ABGR8888,
            width as u32,
            height as u32,
        )?;

        let texture = unsafe { std::mem::transmute::<_, Texture<'static>>(texture) };

        Ok(Canvas {
            width,
            height,
            data: vec![0xff; (width * height * 4) as usize],
            sdl_canvas,
            sdl,
            creator,
            texture: RefCell::new(texture),
        })
    }

    /// Presents the current canvas.
    pub fn present(&mut self) {
        self.sdl_canvas.present();
    }

    /// Clears the screen.
    pub fn clear(&mut self) {
        self.sdl_canvas.clear();
    }

    /// Flushes the current texture.
    pub fn flush(&mut self) {
        let mut texture = self.texture.borrow_mut();
        texture
            .update(None, self.data_raw(), (self.width * 4) as usize)
            .unwrap();
        self.sdl_canvas.copy(&texture, None, None).unwrap();
        self.sdl_canvas.present();
    }

    /// Draws a full line.
    pub fn draw_line_rgba(&mut self, y: usize, dat: &[u8]) {
        let offset = y * self.width * 4;
        self.data[offset..offset + self.width * 4].clone_from_slice(dat);
    }

    /// Converts the internal data vector to a `u8` array.
    fn data_raw(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.data.as_ptr(), self.data.len()) }
    }
}
