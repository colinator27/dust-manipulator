use sdl3::{event::Event, keyboard::Keycode, pixels::Color, rect::{Point, Rect}, render::{ScaleMode, Texture}};

use crate::{program_common::{rect_from_texture, FrameTimer, ScreenSpace}, MainContext, SubProgram};

impl SubProgram {
    fn get_name(&self) -> &'static str {
        match self {
            SubProgram::None => "None",
            SubProgram::ProgramSelector => "Program Selector",
            SubProgram::DustManip => "Dust Manipulator",
            SubProgram::NamingSeedSearch => "Naming Seed Search",
            SubProgram::DogiManip => "Marriage Manipulator",
            SubProgram::Error => "Error",
            SubProgram::RNGOverride => "RNG Seed Override",
        }
    }
}

struct MenuItem<'a> {
    pub rect: Rect,
    pub texture: Texture<'a>
}

pub fn run(main_context: &mut MainContext) -> SubProgram {
    let mut chosen_program = SubProgram::None;
    let program_list = [SubProgram::NamingSeedSearch, SubProgram::DogiManip, SubProgram::DustManip, SubProgram::RNGOverride];
    let mut selection_index: i32 = -1;

    // Render text for all the options
    let mut program_list_items: Vec<MenuItem> = Vec::with_capacity(program_list.len());
    let mut curr_y = 8;
    for program in program_list {
        let surface = main_context.font.render_text(program.get_name(), Color::RGB(255, 255, 255)).expect("Failed to render text to surface");
        let mut texture = Texture::from_surface(&surface, main_context.texture_creator).expect("Failed to create texture from surface");
        texture.set_scale_mode(ScaleMode::Nearest);
        let menu_item = MenuItem {
            rect: Rect::new(8, curr_y, texture.width() * 2, texture.height() * 2),
            texture
        };
        curr_y += 4 + menu_item.rect.h;
        program_list_items.push(menu_item);
    }
    let version_surface = main_context.font.render_text(&format!("Dust Manipulator v{}", env!("CARGO_PKG_VERSION")), Color::RGB(128, 128, 128)).expect("Failed to render text to surface");
    let mut version_texture = Texture::from_surface(&version_surface, main_context.texture_creator).expect("Failed to create texture from surface");
    version_texture.set_scale_mode(ScaleMode::Nearest);

    let mut event_pump = main_context.sdl_context.event_pump().unwrap();
    'running: loop {
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
                Event::MouseMotion { x, y, .. } => {
                    let mut i = 0;
                    for item in &program_list_items {
                        if screen_space.rect_world_to_screen(item.rect).contains_point(Point::new(x as i32, y as i32)) {
                            selection_index = i;
                            break;
                        }
                        i += 1;
                    }
                },
                Event::MouseButtonDown { x, y, .. } => {
                    let mut i = 0;
                    selection_index = -1;
                    for item in &program_list_items {
                        if screen_space.rect_world_to_screen(item.rect).contains_point(Point::new(x as i32, y as i32)) {
                            selection_index = i;
                            break;
                        }
                        i += 1;
                    }
                    if selection_index >= 0 {
                        chosen_program = program_list[selection_index as usize];
                        break 'running;
                    }
                },
                Event::KeyDown { keycode, .. } => {
                    match keycode {
                        Some(Keycode::Up) => {
                            if selection_index == -1 {
                                selection_index = (program_list.len() - 1) as i32;
                            } else {
                                selection_index = (selection_index - 1) % (program_list.len() as i32);
                            }
                        },
                        Some(Keycode::Down) => {
                            if selection_index == -1 {
                                selection_index = 0;
                            } else {
                                selection_index = (selection_index + 1) % (program_list.len() as i32);
                            }
                        },
                        Some(Keycode::Return) => {
                            if selection_index >= 0 {
                                chosen_program = program_list[selection_index as usize];
                                break 'running;
                            }
                        },
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        // Draw the menu items
        let mut i = 0;
        for item in &mut program_list_items {
            if selection_index == i {
                item.texture.set_color_mod(255, 255, 0);
            } else {
                item.texture.set_color_mod(255, 255, 255);
            }
            _ = main_context.canvas.copy(&item.texture, rect_from_texture(&item.texture), screen_space.rect_world_to_screen(item.rect));
            i += 1;
        }

        // Draw other text
        let version_rect = rect_from_texture(&version_texture);
        let version_scale = 2;
        let version_dest_rect = Rect::new(
            screen_space.width() as i32 - (version_rect.width() as i32  * version_scale) - 8,
            screen_space.height() as i32 - (version_rect.height() as i32 * version_scale) - 8, 
            version_rect.width() as u32 * version_scale as u32, version_rect.height() as u32 * version_scale as u32
        );
        _ = main_context.canvas.copy(&version_texture, version_rect, version_dest_rect);

        // Present latest canvas
        main_context.canvas.present();

        // Ignore any server messages
        main_context.ignore_server_messages();

        // Show window, if not already done
        if !main_context.window_shown {
            main_context.canvas.window_mut().show();
            main_context.window_shown = true;
        }

        // Sleep until next frame
        frame_timer.end_and_sleep();
    }

    // Show window, if not already done
    if !main_context.window_shown {
        main_context.canvas.window_mut().show();
        main_context.window_shown = true;
    }

    chosen_program
}