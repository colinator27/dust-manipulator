use std::{f32, ptr};

use sdl3::{pixels::Color, rect::Rect, render::{Canvas, FPoint, FRect, Texture}, video::Window};
use sdl3_sys::{pixels::SDL_FColor, rect::SDL_FPoint, render::{SDL_RenderGeometry, SDL_Vertex}, timer::{SDL_DelayPrecise, SDL_GetTicksNS}};

use crate::MainContext;

pub const DEFAULT_SCREEN_WIDTH: u32 = 640;
pub const DEFAULT_SCREEN_HEIGHT: u32 = 480;

pub struct ScreenSpace {
    scale_amount: f32,
    screen_rect: FRect
}
impl ScreenSpace {
    pub fn new(main_context: &MainContext) -> Self {
        let output_size = main_context.canvas.output_size().unwrap_or_else(|_| (DEFAULT_SCREEN_WIDTH, DEFAULT_SCREEN_HEIGHT));
        let scale_amount = if (output_size.0 as f32) < (output_size.1 as f32 * ((DEFAULT_SCREEN_WIDTH as f32) / (DEFAULT_SCREEN_HEIGHT as f32))) { 
            (output_size.0 as f32) / (DEFAULT_SCREEN_WIDTH as f32) 
        } else { 
            (output_size.1 as f32) / (DEFAULT_SCREEN_HEIGHT as f32) 
        };
        let dst_width = DEFAULT_SCREEN_WIDTH as f32 * scale_amount;
        let dst_height = DEFAULT_SCREEN_HEIGHT as f32 * scale_amount;
        let dst_xoff = (output_size.0 as f32 - dst_width) * 0.5; 
        let dst_yoff = (output_size.1 as f32 - dst_height) * 0.5; 
        
        ScreenSpace {
            scale_amount,
            screen_rect: FRect::new(dst_xoff, dst_yoff, dst_width, dst_height)
        }
    }
    pub fn scale(&self) -> f32 {
        self.scale_amount
    }
    pub fn rect(&self) -> FRect {
        self.screen_rect
    }
    pub fn irect(&self) -> Rect {
        Rect::new(self.screen_rect.x as i32, self.screen_rect.y as i32, self.screen_rect.w as u32, self.screen_rect.h as u32)
    }
    pub fn width(&self) -> f32 {
        self.screen_rect.w
    }
    pub fn height(&self) -> f32 {
        self.screen_rect.h
    }
    pub fn xoffset(&self) -> f32 {
        self.screen_rect.x
    }
    pub fn yoffset(&self) -> f32 {
        self.screen_rect.y
    }
    pub fn center_x(&self) -> f32 {
        self.xoffset() + (self.width() * 0.5)
    }
    pub fn center_y(&self) -> f32 {
        self.yoffset() + (self.height() * 0.5)
    }
    pub fn x_world_to_screen(&self, x: f32) -> f32 {
        self.xoffset() + (x * self.scale())
    }
    pub fn y_world_to_screen(&self, y: f32) -> f32 {
        self.yoffset() + (y * self.scale())
    }
    pub fn rect_world_to_screen(&self, rect: Rect) -> Rect {
        Rect::new(
            ((rect.x as f32 * self.scale()) + self.xoffset()) as i32,
            ((rect.y as f32 * self.scale()) + self.yoffset()) as i32,
            (rect.w as f32 * self.scale()) as u32,
            (rect.h as f32 * self.scale()) as u32
        )
    }
}

pub fn window_to_world(x: f32, y: f32, world_view: Rect, screen_rect: FRect) -> (i32, i32) {
    let normalized_x = (x - screen_rect.x) / screen_rect.w;
    let normalized_y = (y - screen_rect.y) / screen_rect.h;
    let world_x = world_view.x + f32::round(world_view.w as f32 * normalized_x) as i32;
    let world_y = world_view.y + f32::round(world_view.h as f32 * normalized_y) as i32;
    (world_x, world_y)
}
pub fn window_to_world_f32(x: f32, y: f32, world_view: Rect, screen_rect: FRect) -> (f32, f32) {
    let normalized_x = (x - screen_rect.x) / screen_rect.w;
    let normalized_y = (y - screen_rect.y) / screen_rect.h;
    let world_x = world_view.x as f32 + (world_view.w as f32 * normalized_x);
    let world_y = world_view.y as f32 + (world_view.h as f32 * normalized_y);
    (world_x, world_y)
}

pub fn draw_connected_text(main_context: &mut MainContext, screen_space: &ScreenSpace, is_connected: bool) -> Result<(), &'static str> {
    let text_to_show = if is_connected {
        "Connected to OBS."
    } else {
        "Not connected."
    };
    let text_color = if is_connected {
        Color::RGB(0, 255, 0)
    } else {
        Color::RGB(255, 0, 0)
    };
    main_context.font.draw_text(main_context, 
        text_to_show, 
        screen_space.xoffset() + (screen_space.width() * 0.5), screen_space.yoffset() + (screen_space.height() * 0.5),
        0.5, 0.5,
        0,
        screen_space.scale() * 2.0,
        text_color)?;
    Ok(())
}

pub struct FrameTimer {
    frame_start_time: u64,
    target_fps: u32
}
impl FrameTimer {
    pub fn start(target_fps: u32) -> Self {
        FrameTimer {
            frame_start_time: unsafe { SDL_GetTicksNS() },
            target_fps
        }
    }
    pub fn end_and_sleep(&self) {
        // Sleep to match desired FPS, subtracting sleep time depending on how long this frame took.
        // This isn't perfect for frame pacing, but all we really care about is that frames don't take way longer than they should.
        let frame_end_time = unsafe { SDL_GetTicksNS() };
        let desired_delay = (1_000_000_000i64 / (self.target_fps as i64)) - ((frame_end_time as i64) - (self.frame_start_time as i64));
        if desired_delay > 0 {
            unsafe { SDL_DelayPrecise(desired_delay as u64) };
        }
    }
}

pub fn rect_to_frect(rect: Rect) -> FRect {
    FRect::new(rect.x as f32, rect.y as f32, rect.w as f32, rect.h as f32)
}
pub fn frect_to_rect(frect: FRect) -> Rect {
    Rect::new(frect.x as i32, frect.y as i32, frect.w as u32, frect.h as u32)
}
pub fn rect_from_texture(texture: &Texture) -> Rect {
    Rect::new(0, 0, texture.width(), texture.height())
}
pub fn frect_camera_transform(frect: FRect, camera_position: FPoint, camera_scale: f32) -> FRect {
    FRect::new((frect.x - camera_position.x) * camera_scale, (frect.y - camera_position.y) * camera_scale, frect.w * camera_scale, frect.h * camera_scale)
}
pub fn fpoint_camera_transform(fpoint: FPoint, camera_position: FPoint, camera_scale: f32) -> FPoint {
    FPoint::new((fpoint.x - camera_position.x) * camera_scale, (fpoint.y - camera_position.y) * camera_scale)
}

pub fn draw_circle(canvas: &mut Canvas<Window>, x: f32, y: f32, radius: f32, precision: u32, color: Color) {
    // Precision is rounded to a multiple of 4
    let precision = (precision / 4) << 2;

    // Color converts to floating point
    let color = SDL_FColor { 
        r: color.r as f32 / 255.0,
        g: color.g as f32 / 255.0,
        b: color.b as f32 / 255.0,
        a: color.a as f32 / 255.0
    };

    // Create vertex buffer (triangle list)
    let mut vertices: Vec<SDL_Vertex> = Vec::with_capacity((precision * 3) as usize);
    for i in 0..precision {
        vertices.push(SDL_Vertex {
            position: SDL_FPoint { x, y },
            color,
            ..Default::default()
        });
        vertices.push(SDL_Vertex {
            position: SDL_FPoint { 
                x: x + (radius * f32::cos((i as f32 * 2.0 * f32::consts::PI) / (precision as f32))),
                y: y + (radius * f32::sin((i as f32 * 2.0 * f32::consts::PI) / (precision as f32))),
            },
            color,
            ..Default::default()
        });
        vertices.push(SDL_Vertex {
            position: SDL_FPoint { 
                x: x + (radius * f32::cos(((i + 1) as f32 * 2.0 * f32::consts::PI) / (precision as f32))),
                y: y + (radius * f32::sin(((i + 1) as f32 * 2.0 * f32::consts::PI) / (precision as f32))),
            },
            color,
            ..Default::default()
        });
    }

    // Draw triangle list
    unsafe { SDL_RenderGeometry(canvas.raw(), ptr::null_mut(), vertices.as_ptr(), vertices.len() as i32, ptr::null(), 0) };
}
