use std::{ffi::CString, marker::PhantomData};

use sdl3::{pixels::Color, rect::Rect, render::{FRect, ScaleMode, Texture}, surface::Surface};
use sdl3_sys::pixels::SDL_Color;
use sdl3_ttf_sys::ttf::{self, TTF_HorizontalAlignment};

use crate::{util, MainContext};

pub struct TTFContext {
    engine: *mut ttf::TTF_TextEngine
}
impl TTFContext {
    pub fn new() -> Result<Self, &'static str> {
        if !unsafe { ttf::TTF_Init() } {
            return Err("Failed to initialize TTF");
        }
    
        let engine = unsafe { ttf::TTF_CreateSurfaceTextEngine() };
        if engine.is_null() {
            return Err("Failed to initialize surface text engine");
        }
    
        Ok(TTFContext {
            engine
        })
    }
    
    pub fn load_font(&self, path: &str, size: f32) -> Result<Font, &'static str> {
        let font = unsafe { ttf::TTF_OpenFont(CString::new(util::get_exe_directory().join(path).to_str().unwrap()).unwrap().as_ptr(), size) };
        if font.is_null() {
            return Err("Failed to open file data as a font");
        }

        Ok(Font {
            ptr: font,
            _marker: PhantomData
        })
    }
}
impl Drop for TTFContext {
    fn drop(&mut self) {
        unsafe { 
            ttf::TTF_DestroySurfaceTextEngine(self.engine);
            ttf::TTF_Quit(); 
        }
    }
}

pub struct Font<'a> {
    ptr: *mut ttf::TTF_Font,
    _marker: PhantomData<&'a TTFContext>
}
impl Font<'_> {
    pub fn render_text(&self, text: &str, color: Color) -> Result<Surface, &'static str> {
        let text = unsafe { ttf::TTF_RenderText_Solid_Wrapped(self.ptr, CString::new(text).unwrap().as_ptr(), text.len(), SDL_Color {
            r: color.r, g: color.g, b: color.b, a: color.a
        }, 0) };
        if text.is_null() {
            Err("Failed to render text")
        } else {
            Ok(unsafe { Surface::from_ll(text) })
        }
    }
    pub fn render_text_autowrap(&self, text: &str, color: Color, wrap_length: i32) -> Result<Surface, &'static str> {
        let text = unsafe { ttf::TTF_RenderText_Solid_Wrapped(self.ptr, CString::new(text).unwrap().as_ptr(), text.len(), SDL_Color {
            r: color.r, g: color.g, b: color.b, a: color.a
        }, wrap_length) };
        if text.is_null() {
            Err("Failed to render text")
        } else {
            Ok(unsafe { Surface::from_ll(text) })
        }
    }
    pub fn draw_text(&self, main_context: &mut MainContext, text: &str, x: f32, y: f32, 
                     normalized_origin_x: f32, normalized_origin_y: f32, wrap_length: i32, scale: f32, color: Color) -> Result<(), &'static str> {
        let text_surface = self.render_text_autowrap(text, color, wrap_length)?;
        let texture = Texture::from_surface(&text_surface, &main_context.texture_creator);
        if texture.is_err() {
            return Err("Failed to create texture from surface");
        }
        let mut texture = texture.unwrap();
        texture.set_scale_mode(ScaleMode::Nearest);
        _ = main_context.canvas.copy(&texture, 
            Rect::new(0, 0, texture.width(), texture.height()), 
            Rect::new((x - ((texture.width() as f32 * scale) * normalized_origin_x)) as i32, 
                          (y - ((texture.height() as f32 * scale) * normalized_origin_y)) as i32, 
                          (texture.width() as f32 * scale) as u32, 
                          (texture.height() as f32 * scale) as u32));
        Ok(())
    }
    pub fn draw_text_bg(&self, main_context: &mut MainContext, text: &str, x: f32, y: f32, 
                        normalized_origin_x: f32, normalized_origin_y: f32, wrap_length: i32, scale: f32, color: Color, 
                        bg_color: Color, bg_padding: f32) -> Result<(), &'static str> {
        let text_surface = self.render_text_autowrap(text, color, wrap_length)?;
        let texture = Texture::from_surface(&text_surface, &main_context.texture_creator);
        if texture.is_err() {
            return Err("Failed to create texture from surface");
        }
        let mut texture = texture.unwrap();
        texture.set_scale_mode(ScaleMode::Nearest);
        let dst_rect = Rect::new((x - ((texture.width() as f32 * scale) * normalized_origin_x)) as i32, 
                                       (y - ((texture.height() as f32 * scale) * normalized_origin_y)) as i32, 
                                       (texture.width() as f32 * scale) as u32, 
                                       (texture.height() as f32 * scale) as u32);
        main_context.canvas.set_draw_color(bg_color);
        _ = main_context.canvas.fill_rect(FRect::new(dst_rect.x as f32 - bg_padding, dst_rect.y as f32 - bg_padding, dst_rect.w as f32 + (bg_padding * 2.0), dst_rect.h as f32 + (bg_padding * 2.0)));
        _ = main_context.canvas.copy(&texture, 
            Rect::new(0, 0, texture.width(), texture.height()), 
            dst_rect);
        Ok(())
    }
    pub fn set_alignment(&self, alignment: TTF_HorizontalAlignment) {
        unsafe { ttf::TTF_SetFontWrapAlignment(self.ptr, alignment) };
    }
}
impl Drop for Font<'_> {
    fn drop(&mut self) {
        unsafe { ttf::TTF_CloseFont(self.ptr) };
    }
}
