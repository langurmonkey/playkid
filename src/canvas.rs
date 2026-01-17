use crate::constants;
use sdl2::rect::Rect;
use sdl2::{pixels::PixelFormatEnum, render::Texture, render::TextureCreator, Sdl};
use std::cell::RefCell;

type Error = Box<dyn std::error::Error>;

/// Controls the canvas and the actual rendering.
/// See https://users.rust-lang.org/t/rust-sdl2-and-raw-textures-help/45636/25
#[allow(dead_code)]
pub struct Canvas<'a> {
    sdl: &'a Sdl,
    pub sdl_canvas: sdl2::render::Canvas<sdl2::video::Window>,
    pub creator: TextureCreator<sdl2::video::WindowContext>,
    texture: RefCell<Texture<'static>>,
    data: Vec<u8>,
    width: usize,
    height: usize,
    lcd_rect: Rect,
}
impl<'a> Canvas<'a> {
    pub fn new(
        sdl: &'a Sdl,
        title: &str,
        width: usize,
        height: usize,
        scale: usize,
    ) -> Result<Self, Error> {
        // Nearest filter.
        sdl2::hint::set("SDL_RENDER_SCALE_QUALITY", "nearest");
        // Create window.
        let video_subsystem = sdl.video()?;
        let window = video_subsystem
            .window(title, (width * scale) as u32, (height * scale) as u32)
            .resizable()
            .allow_highdpi()
            .position_centered()
            .build()?;
        let sdl_canvas = window.into_canvas().build()?;
        let creator = sdl_canvas.texture_creator();
        let texture = creator.create_texture_target(
            PixelFormatEnum::ABGR8888,
            width as u32,
            height as u32,
        )?;

        // The texture to render the Game Boy screen.
        let texture = unsafe { std::mem::transmute::<_, Texture<'static>>(texture) };

        Ok(Canvas {
            width,
            height,
            data: vec![0xff; (width * height * 4) as usize],
            sdl_canvas,
            sdl,
            creator,
            texture: RefCell::new(texture),
            lcd_rect: Rect::new(0, 0, 0, 0),
        })
    }

    /// Presents the current canvas.
    pub fn present(&mut self) {
        self.sdl_canvas.present();
    }

    /// Clears the screen.
    pub fn clear(&mut self) {
        self.sdl_canvas
            .set_draw_color(sdl2::pixels::Color::RGB(0, 0, 0));
        self.sdl_canvas.clear();
    }

    /// Flushes the current texture.
    /// If debug is false, the Game Boy display is presented letter-boxed.
    /// If debug is on, the Game Boy display is aligned to the left, and the debug
    /// info is to the right.
    pub fn flush(&mut self, debug: bool) {
        let mut texture = self.texture.borrow_mut();
        texture
            .update(None, self.data_raw(), (self.width * 4) as usize)
            .unwrap();

        // We want to respect the aspect ratio (160:144) when resizing.
        // Get current window size.
        let (w, h) = self.sdl_canvas.output_size().unwrap();

        // In debug mode, we have a 2:1 ratio for the debug UI.
        let render_width = self.width * if debug { 2 } else { 1 };

        // Aspect ratio scale.
        let scale = f32::min(
            w as f32 / render_width as f32,
            h as f32 / self.height as f32,
        );

        // New width and height.
        // In debug mode, LCD size is fixed.
        let (nw, nh) = if debug {
            // Fixed.
            (
                (constants::DISPLAY_WIDTH as f32 * 1.5) as u32,
                (constants::DISPLAY_HEIGHT as f32 * 1.5) as u32,
            )
        } else {
            // Compute from scale.
            (
                (self.width as f32 * scale) as u32,
                (self.height as f32 * scale) as u32,
            )
        };

        // Center the rectangle.
        // In debug mode, fixed position at [10,10].
        let x = if debug { 10 } else { (w - nw) / 2 };
        let y = if debug { 10 } else { (h - nh) / 2 };
        self.lcd_rect.set_x(x as i32);
        self.lcd_rect.set_y(y as i32);
        self.lcd_rect.set_width(nw);
        self.lcd_rect.set_height(nh);

        // Clear screen to black.
        self.sdl_canvas
            .set_draw_color(sdl2::pixels::Color::RGB(0, 0, 0));
        self.sdl_canvas.clear();

        // Render Game Boy frame.
        self.sdl_canvas
            .copy(&texture, None, Some(self.lcd_rect))
            .unwrap();
    }

    /// Get the bounds of the current LCD rectangle.
    pub fn get_lcd_rect(&self) -> (i32, i32, u32, u32) {
        (
            self.lcd_rect.x(),
            self.lcd_rect.y(),
            self.lcd_rect.width(),
            self.lcd_rect.height(),
        )
    }

    /// Return the scale factor of the current display.
    pub fn get_scale_factor(&self) -> f32 {
        let window_display_index = self.sdl_canvas.window().display_index().unwrap_or(0);
        let (ddpi, _, _) = self
            .sdl
            .video()
            .unwrap()
            .display_dpi(window_display_index)
            .unwrap_or((96.0, 96.0, 96.0));
        ddpi / 96.0
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
