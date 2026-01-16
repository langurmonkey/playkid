use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::ttf::Font;
use sdl2::{pixels::PixelFormatEnum, render::Texture, render::TextureCreator, Sdl};
use std::cell::RefCell;
use std::collections::HashMap;

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
    /// The font to render text.
    font: Font<'a, 'static>,
    /// Map text ID to the string and layers (Texture, Rect).
    text_cache: HashMap<usize, (String, Vec<(Texture<'static>, Rect)>, (f32, f32))>,
}
impl<'a> Canvas<'a> {
    pub fn new(
        sdl: &'a Sdl,
        ttf: &'a sdl2::ttf::Sdl2TtfContext,
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

        // The text hash map.
        let text_cache = HashMap::new();

        // Load font file.
        let font = ttf.load_font("assets/fnt/PixelatedElegance.ttf", 14)?;

        Ok(Canvas {
            width,
            height,
            data: vec![0xff; (width * height * 4) as usize],
            sdl_canvas,
            sdl,
            creator,
            texture: RefCell::new(texture),
            font,
            text_cache,
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

    /// Add a new text to the text cache with the given ID, string, position, and color.
    /// Only update the texture if the ID doesn't exist.
    /// The text will be rendered to the canvas during the next call to `flush()`.
    /// X and Y are given in pixel coordinates.
    pub fn draw_text(&mut self, id: usize, text: &str, x: f32, y: f32, color: Color) {
        self.draw_text_shadow(id, text, x, y, color, false);
    }

    /// Variant of `draw_text()` where a shadow is optionally rendered behind the text,
    /// in black.
    /// X and Y are given in pixel coordinates.
    pub fn draw_text_shadow(
        &mut self,
        id: usize,
        text: &str,
        x: f32,
        y: f32,
        color: Color,
        shadow: bool,
    ) {
        // Check if we can skip the expensive texture creation.
        if let Some((old_text, _, coords)) = self.text_cache.get_mut(&id) {
            if old_text == text {
                *coords = (x, y);
                return;
            }
        }

        // If we reach here, text changed or ID is new. Re-render surfaces.
        let mut layers = Vec::new();
        // Shadow offset, in pixels.
        let offset = 1;

        if shadow {
            // Shadow layer.
            let s_surf = self.font.render(text).blended(Color::RGB(0, 0, 0)).unwrap();
            let s_tex = self.creator.create_texture_from_surface(&s_surf).unwrap();
            let s_tex = unsafe { std::mem::transmute::<_, Texture<'static>>(s_tex) };
            layers.push((
                s_tex,
                Rect::new(offset, offset, s_surf.width(), s_surf.height()),
            ));
        }

        // Text layer.
        let f_surf = self.font.render(text).blended(color).unwrap();
        let f_tex = self.creator.create_texture_from_surface(&f_surf).unwrap();
        let f_tex = unsafe { std::mem::transmute::<_, Texture<'static>>(f_tex) };
        layers.push((f_tex, Rect::new(0, 0, f_surf.width(), f_surf.height())));

        // Put in cache.
        self.text_cache
            .insert(id, (text.to_string(), layers, (x, y)));
    }

    /// Remove the text with the given ID from the text cache.
    pub fn remove_text(&mut self, id: usize) {
        self.text_cache.remove(&id);
    }

    /// Checks if the text with the given ID is in the cache.
    pub fn has_text(&self, id: usize) -> bool {
        self.text_cache.contains_key(&id)
    }

    /// Flushes the current texture.
    pub fn flush(&mut self, debug: bool) {
        let mut texture = self.texture.borrow_mut();
        texture
            .update(None, self.data_raw(), (self.width * 4) as usize)
            .unwrap();

        // We want to respect the aspect ratio (160:144) when resizing.
        // Get current window size.
        let (w, h) = self.sdl_canvas.output_size().unwrap();

        let render_width = self.width * if debug { 2 } else { 1 };

        // Aspect ratio scale.
        let scale = f32::min(
            w as f32 / render_width as f32,
            h as f32 / self.height as f32,
        );

        // New width and height.
        let nw = (self.width as f32 * scale) as u32;
        let nh = (self.height as f32 * scale) as u32;

        // Center the rectangle.
        let x = if debug { 0 } else { 1 } * (w - nw) / 2;
        let y = (h - nh) / 2;
        let target_rect = Rect::new(x as i32, y as i32, nw, nh);

        // Clear screen to black.
        self.sdl_canvas
            .set_draw_color(sdl2::pixels::Color::RGB(0, 0, 0));
        self.sdl_canvas.clear();
        // Render Game Boy frame.
        self.sdl_canvas
            .copy(&texture, None, Some(target_rect))
            .unwrap();

        // Render UI (text elements).
        // Get DPI scale factor of current display.
        let window_display_index = self.sdl_canvas.window().display_index().unwrap_or(0);
        let (ddpi, _, _) = self
            .sdl
            .video()
            .unwrap()
            .display_dpi(window_display_index)
            .unwrap_or((96.0, 96.0, 96.0));
        let scale_factor = ddpi / 96.0;

        self.sdl_canvas
            .set_draw_color(sdl2::pixels::Color::RGB(0, 0, 0));
        self.sdl_canvas.clear();
        self.sdl_canvas
            .copy(&texture, None, Some(target_rect))
            .unwrap();

        // Draw UI with DPI scaling.
        self.sdl_canvas
            .set_blend_mode(sdl2::render::BlendMode::Blend);

        for (_, (_, layers, (lx, ly))) in &self.text_cache {
            // Calculate the base anchor point in physical pixels
            let base_x = target_rect.x() + *lx as i32;
            let base_y = target_rect.y() + *ly as i32;

            // Draw background box (using the last layer, which is always the foreground text).
            if let Some((_, text_info)) = layers.last() {
                let padding = (4.0 * scale_factor) as i32;
                let bg_rect = Rect::new(
                    base_x - padding,
                    base_y - padding,
                    (text_info.width() as f32 * scale_factor) as u32 + (padding as u32 * 2),
                    (text_info.height() as f32 * scale_factor) as u32 + (padding as u32 * 2),
                );

                self.sdl_canvas.set_draw_color(Color::RGBA(0, 0, 0, 160));
                self.sdl_canvas.fill_rect(bg_rect).unwrap();
            }

            // Draw text layers (shadows then foreground).
            for (tex, local_rect) in layers {
                let dest_rect = Rect::new(
                    base_x + (local_rect.x() as f32 * scale_factor) as i32,
                    base_y + (local_rect.y() as f32 * scale_factor) as i32,
                    (local_rect.width() as f32 * scale_factor) as u32,
                    (local_rect.height() as f32 * scale_factor) as u32,
                );
                self.sdl_canvas.copy(tex, None, Some(dest_rect)).unwrap();
            }
        }

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
