use std::sync::atomic::Ordering;

use sdl3::{event::Event, keyboard::Keycode, pixels::Color, rect::Rect, render::{ScaleMode, Texture}};

use crate::{program_common::{rect_from_texture, FrameTimer, ScreenSpace}, MainContext, SubProgram};

pub fn run(main_context: &mut MainContext) -> SubProgram {
    let mut typed_string: String = "".to_owned();

    let mut event_pump = main_context.sdl_context.event_pump().unwrap();
    'running: loop {
        // Handle thread errors
        if main_context.panic_occurred.load(Ordering::Relaxed) {
            break;
        }
        
        let frame_timer = FrameTimer::start(30);
        let screen_space = ScreenSpace::new(&main_context);

        main_context.canvas.set_draw_color(Color::RGB(0, 0, 0));
        main_context.canvas.set_blend_mode(sdl3::render::BlendMode::Blend);
        main_context.canvas.clear();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} => {
                    return SubProgram::None;
                },
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                Event::KeyDown { keycode: Some(Keycode::Backspace), .. } => {
                    if typed_string.len() > 0 {
                        typed_string = typed_string[0..typed_string.len() - 1].to_string();
                    }
                },
                Event::KeyDown { keycode: Some(Keycode::Return), .. } => {
                    if typed_string.len() > 0 {
                        main_context.run_context.set_rng(typed_string.parse::<u32>().unwrap(), 0);
                        break 'running;
                    }
                },
                Event::KeyDown { keycode: Some(keycode), .. } => {
                    if typed_string.len() >= 5 {
                        continue;
                    }
                    if (keycode as i32) >= (Keycode::_0 as i32) && (keycode as i32) <= (Keycode::_9 as i32) {
                        let digit = (keycode as i32) - (Keycode::_0 as i32);
                        typed_string += &digit.to_string();
                    }
                },
                _ => {}
            }
        }

        // Draw the typed string
        if typed_string.len() > 0 {
            let surface = main_context.font.render_text(&typed_string, Color::RGB(255, 255, 255)).expect("Failed to render text to surface");
            let mut texture = Texture::from_surface(&surface, main_context.texture_creator).expect("Failed to create texture from surface");
            drop(surface);
            texture.set_scale_mode(ScaleMode::Nearest);
            let texture_src_rect = rect_from_texture(&texture);
            let texture_dst_rect = Rect::new(320 - texture_src_rect.w, 240 - texture_src_rect.h, texture_src_rect.w as u32 * 2, texture_src_rect.h as u32 * 2);
            _ = main_context.canvas.copy(&texture, texture_src_rect, screen_space.rect_world_to_screen(texture_dst_rect));
        }

        // Present latest canvas
        main_context.canvas.present();

        // Ignore any server messages
        main_context.ignore_server_messages();
        
        // Sleep until next frame
        frame_timer.end_and_sleep();
    }

    SubProgram::ProgramSelector
}