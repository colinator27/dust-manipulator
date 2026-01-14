use std::{sync::{atomic::{AtomicBool, Ordering}, Arc, Mutex}, thread};

use defer_rs::defer;
use sdl3::{event::Event, keyboard::Keycode, mouse::MouseButton, pixels::{Color, PixelFormat}, rect::{Point, Rect}, render::{BlendMode, ScaleMode, Texture}, surface::Surface};

use crate::{compute_naming_search::{self, NamingSearchParameters, NamingSearchResult}, program_common::{self, window_to_world, FrameTimer, ScreenSpace}, rng::RNG, server::MessageToSend, windowing::{focus_game_window, window_set_focusable}, MainContext, SubProgram};

// Points on screen, such that if they aren't black, represents a random(0.5) that is definitely > 0.25
// Ordered by letters A-Z then a-z, with Y offset coming first due to reverse order argument evaluation.
const LETTER_DATA: [(i32, i32); 104] = [
    (122, 176), (132, 168), // A
    (186, 176), (196, 163), // B
    (252, 176), (260, 172), // C
    (314, 176), (324, 166), // D
    (378, 176), (388, 159), // E
    (442, 176), (452, 159), // F
    (508, 176), (516, 172), // G
    (122, 204), (132, 200), // H
    (186, 204), (196, 187), // I
    (252, 204), (260, 196), // J
    (314, 204), (324, 202), // K
    (378, 204), (388, 203), // L
    (442, 204), (454, 196), // M
    (506, 204), (516, 196), // N
    (124, 232), (132, 224), // O
    (186, 232), (196, 217), // P
    (256, 236), (260, 224), // Q
    (314, 232), (324, 228), // R
    (380, 232), (388, 228), // S
    (446, 232), (452, 215), // T
    (508, 232), (516, 224), // U
    (126, 260), (132, 248), // V
    (188, 260), (198, 248), // W
    (250, 260), (260, 258), // X
    (318, 260), (324, 247), // Y
    (378, 260), (388, 246), // Z
    (124, 296), (132, 292), // a
    (186, 296), (196, 292), // b
    (252, 296), (260, 292), // c
    (316, 296), (324, 293), // d
    (380, 296), (388, 288), // e
    (446, 296), (452, 279), // f
    (508, 302), (516, 296), // g
    (122, 324), (132, 321), // h
    (186, 324), (196, 323), // i
    (252, 330), (260, 325), // j
    (314, 324), (324, 322), // k
    (378, 324), (388, 323), // l
    (442, 324), (454, 320), // m
    (506, 324), (516, 320), // n
    (124, 352), (132, 347), // o
    (186, 358), (196, 347), // p
    (258, 358), (260, 348), // q
    (314, 352), (324, 342), // r
    (380, 352), (388, 348), // s
    (448, 352), (452, 339), // t
    (508, 352), (516, 347), // u
    (126, 380), (132, 370), // v
    (188, 380), (198, 372), // w
    (250, 380), (260, 378), // x
    (316, 386), (324, 381), // y
    (378, 380), (388, 368)  // z
];

struct NamingPixel {
    pub rect: Rect,
    pub hovered: bool,
    pub selected: bool
}

#[derive(PartialEq)]
enum NamingSearchState {
    Waiting,
    ClickingPixels,
    Found
}

const WORLD_WIDTH: u32 = 640;
const WORLD_HEIGHT: u32 = 480;

pub fn run(main_context: &mut MainContext) -> SubProgram {
    // Current program state
    let mut naming_search_state = NamingSearchState::Waiting;
    let mut naming_pixels: Vec<NamingPixel> = Vec::with_capacity(LETTER_DATA.len());
    for pos in LETTER_DATA {
        naming_pixels.push(NamingPixel {
            rect: Rect::new(pos.0 - 3, pos.1 - 3, 7, 7),
            hovered: false,
            selected: false
        });
    }
    let mut naming_rect_index = 0;
    let mut naming_rect = &main_context.config.naming_rects[naming_rect_index];
    let mut naming_rect_zoom = Rect::new(naming_rect.zoom.x as i32, naming_rect.zoom.y as i32, naming_rect.zoom.w, naming_rect.zoom.h);
    let mut naming_rect_crop = Rect::new(naming_rect.crop.x as i32, naming_rect.crop.y as i32, naming_rect.crop.w, naming_rect.crop.h);
    let naming_rect_count = main_context.config.naming_rects.len();

    // Last server connected state
    let mut last_server_connected = main_context.server_connected.load(Ordering::Relaxed);
    
    // Make sure plugin takes one screenshot, and instantly
    _ = main_context.message_to_send_sender.send(MessageToSend::new_screenshot_mode(true));
    _ = main_context.message_to_send_sender.send(MessageToSend::new_screenshot_start_delay(0));

    // Initialize compute thread
    let compute_end_signal = Arc::new(AtomicBool::new(false));
    let compute_end_signal_thread = compute_end_signal.clone();
    let compute_perform_search_signal = Arc::new(AtomicBool::new(false));
    let compute_perform_search_signal_thread = compute_perform_search_signal.clone();
    let runner_version = &main_context.config.runner_version;
    let unique_seeds = Arc::new(Mutex::new(RNG::calculate_unique_seeds(runner_version.rng_15bit(), runner_version.rng_signed())));
    let unique_seeds_thread = unique_seeds.clone();
    let compute_parameters = Arc::new(Mutex::new(NamingSearchParameters {
        search_range: 0,
        matching_pixels: vec![],
        rng_15bit: false,
        rng_old_poly: false,
        rng_signed: false
    }));
    let compute_parameters_thread = compute_parameters.clone();
    let compute_result = Arc::new(Mutex::new(NamingSearchResult { match_count: 0, single_matched_seed: 0, single_matched_position: 0 }));
    let compute_result_thread = compute_result.clone();
    let compute_join_handle = thread::spawn(move || {
        compute_naming_search::thread_func(
            Arc::clone(&compute_end_signal_thread), Arc::clone(&compute_perform_search_signal_thread), 
            Arc::clone(&unique_seeds_thread), Arc::clone(&compute_parameters_thread), Arc::clone(&compute_result_thread));
    });
    defer! {
        // End compute thread
        compute_end_signal.store(true, Ordering::Relaxed);
        compute_join_handle.thread().unpark();
    };

    // State for whether a search is currently queued, or whether a search is currently in progress
    let mut queued_search = false;
    let mut waiting_for_search_result = false;

    // Whether RNG was just found by this tool or not
    let mut rng_just_found = false;

    // Match count for when a search fails
    let mut rng_fail_match_count = -1;

    // Countdown for automatically advancing, if enabled
    let mut auto_advance_countdown = 0;

    // State for mouse dragging
    let mut last_selected_toggle_result = true;

    // Screenshot for naming screen to be displayed
    let mut screenshot_texture: Option<Texture> = None;
    
    // Make overlay texture
    let mut overlay_texture = main_context.texture_creator
        .create_texture_target(PixelFormat::RGBA32, WORLD_WIDTH, WORLD_HEIGHT)
        .expect("Failed to create texture target");
    overlay_texture.set_scale_mode(sdl3::render::ScaleMode::Nearest);
    
    // Start main loop
    let mut event_pump = main_context.sdl_context.event_pump().unwrap();
    'running: loop {
        // Handle thread errors
        if main_context.panic_occurred.load(Ordering::Relaxed) {
            break;
        }

        // Advance to next tool if wanted
        if auto_advance_countdown > 0 {
            auto_advance_countdown -= 1;
            if naming_search_state == NamingSearchState::Found {
                if auto_advance_countdown == 0 {
                    return main_context.config.naming_advance_tool;
                }
            } else {
                auto_advance_countdown = 0;
            }
        }

        let frame_timer = FrameTimer::start(30);
        let screen_space = ScreenSpace::new(&main_context);

        main_context.canvas.set_draw_color(Color::RGB(0, 0, 0));
        main_context.canvas.set_blend_mode(sdl3::render::BlendMode::Blend);
        main_context.canvas.clear();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} => {
                    return SubProgram::None
                },
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                Event::MouseMotion { x, y, mousestate, .. } => {
                    match naming_search_state {
                        NamingSearchState::ClickingPixels => {
                            let (selector_x, selector_y) = window_to_world(x, y, naming_rect_zoom, screen_space.rect());
                            for pixel in naming_pixels.iter_mut() {
                                if !naming_rect_crop.contains_rect(pixel.rect) {
                                    continue;
                                }
                                if pixel.rect.contains_point(Point::new(selector_x, selector_y)) {
                                    if !pixel.hovered && mousestate.left() {
                                        pixel.selected = last_selected_toggle_result;
                                    }
                                    pixel.hovered = true;
                                } else {
                                    pixel.hovered = false;
                                }
                            }
                        },
                        _ => {}
                    }
                },
                Event::MouseButtonDown { mouse_btn, x, y, .. } => {
                    if mouse_btn == MouseButton::Right {
                        naming_rect_index = (naming_rect_index + 1) % naming_rect_count;
                        naming_rect = &main_context.config.naming_rects[naming_rect_index];
                        naming_rect_zoom = Rect::new(naming_rect.zoom.x as i32, naming_rect.zoom.y as i32, naming_rect.zoom.w, naming_rect.zoom.h);
                        naming_rect_crop = Rect::new(naming_rect.crop.x as i32, naming_rect.crop.y as i32, naming_rect.crop.w, naming_rect.crop.h);
                        for pixel in naming_pixels.iter_mut() {
                            pixel.hovered = false;
                        }
                        continue;
                    }
                    if mouse_btn != MouseButton::Left {
                        continue;
                    }
                    last_selected_toggle_result = true;
                    let (selector_x, selector_y) = window_to_world(x, y, naming_rect_zoom, screen_space.rect());
                    for pixel in naming_pixels.iter_mut() {
                        if !naming_rect_crop.contains_rect(pixel.rect) {
                            continue;
                        }
                        if pixel.rect.contains_point(Point::new(selector_x, selector_y)) {
                            if mouse_btn == MouseButton::Left {
                                pixel.selected = !pixel.selected;
                                last_selected_toggle_result = pixel.selected;
                            } else {
                                pixel.selected = false;
                            }
                        }
                    }
                },
                _ => {}
            }
        }

        if waiting_for_search_result && !compute_perform_search_signal.load(Ordering::Relaxed) {
            waiting_for_search_result = false;

            let search_result = compute_result.lock().unwrap();
            if search_result.match_count == 1 {
                // Singular match!
                rng_fail_match_count = -1;
                println!("Found seed = {}, pos = {}", search_result.single_matched_seed, search_result.single_matched_position);

                // Set current RNG for the run
                main_context.run_context.set_rng(search_result.single_matched_seed, search_result.single_matched_position as usize);

                // Progress to next state
                naming_search_state = NamingSearchState::Found;
                rng_just_found = true;
                if main_context.config.naming_auto_advance_seconds > 0 {
                    auto_advance_countdown = frame_timer.target_fps() * main_context.config.naming_auto_advance_seconds;
                }
            } else {
                rng_fail_match_count = search_result.match_count as i32;
                println!("Match count = {}, data1 = {}, data2 = {}", search_result.match_count, search_result.single_matched_seed, search_result.single_matched_position);
            }
        }

        // Check for any incoming hotkeys
        for hotkey_id in main_context.hotkey_receiver.try_iter() {
            match hotkey_id {
                0 => {
                    // Screenshots - ignore
                }
                1 => {
                    // Raise window (and warp mouse) but make unfocusable
                    let window = main_context.canvas.window_mut();
                    if main_context.config.mouse_warps {
                        main_context.sdl_context.mouse().warp_mouse_in_window(window, window.size().0 as f32 / 2.0, window.size().1 as f32 / 2.0);
                    }
                    window_set_focusable(window, false);
                    window.sync();
                    focus_game_window();
                }
                2 => {
                    // Perform actual search, or if in found state, progress to next tool
                    if rng_just_found {
                        return main_context.config.naming_advance_tool;
                    } else {
                        queued_search = true;
                    }
                }
                3 => {
                    // Focus window
                    let window = main_context.canvas.window_mut();
                    window_set_focusable(window, true);
                    window.raise();
                }
                _ => {}
            }
        }
        
        // Perform search if queued
        if queued_search && !waiting_for_search_result {
            queued_search = false;

            *compute_parameters.lock().unwrap() = NamingSearchParameters {
                search_range: 30_000u32,
                rng_15bit: runner_version.rng_15bit(),
                rng_signed: runner_version.rng_signed(),
                rng_old_poly: runner_version.rng_old_poly(),
                matching_pixels: naming_pixels.iter().map(|p| p.selected).collect()
            };

            waiting_for_search_result = true;
            compute_perform_search_signal.store(true, Ordering::Relaxed);
            compute_join_handle.thread().unpark();
        }

        // Check for incoming screenshot from the server
        let mut local_screenshot_data = main_context.screenshot_data.lock().unwrap();
        if local_screenshot_data.len() >= 1 {
            // Switch to the clicking pixels state
            naming_search_state = NamingSearchState::ClickingPixels;

            // Get screenshot data
            let screenshot_data = &mut local_screenshot_data.pop().unwrap();

            // Create texture
            let surface = Surface::from_data(&mut screenshot_data.data, 
                screenshot_data.width, screenshot_data.height, screenshot_data.stride, PixelFormat::RGBA32).unwrap();
            let mut texture = Texture::from_surface(&surface, &main_context.texture_creator).unwrap();
            texture.set_scale_mode(ScaleMode::Nearest);
            screenshot_texture = Some(texture);
        }
        drop(local_screenshot_data);

        // Check whether connected
        let is_connected = main_context.server_connected.load(Ordering::Relaxed);
        if is_connected != last_server_connected {
            last_server_connected = is_connected;

            // Update newly-connected plugin with latest info
            if is_connected {
                _ = main_context.message_to_send_sender.send(MessageToSend::new_screenshot_mode(true));
                _ = main_context.message_to_send_sender.send(MessageToSend::new_screenshot_start_delay(0));
            }
        }

        // Draw different contents depending on the current state
        match naming_search_state {
            NamingSearchState::Waiting => {
                // Draw text for whether connected or not
                _ = program_common::draw_connected_text(main_context, &screen_space, is_connected);
            },
            NamingSearchState::ClickingPixels => {
                // Draw inside of the overlay texture
                _ = main_context.canvas.with_texture_canvas(&mut overlay_texture, |texture_canvas| {
                    // Clear the overlay texture
                    texture_canvas.set_draw_color(Color::RGBA(0, 0, 0, 220));
                    texture_canvas.clear();

                    // Draw pixels
                    texture_canvas.set_blend_mode(BlendMode::None);
                    texture_canvas.set_draw_color(Color::RGBA(0, 0, 0, 0));
                    for pixel in LETTER_DATA {
                        if !naming_rect_crop.contains_point(Point::new(pixel.0, pixel.1)) {
                            continue;
                        }
                        _ = texture_canvas.fill_rect(Rect::new(pixel.0, pixel.1, 1, 1));
                    }
                    texture_canvas.set_blend_mode(BlendMode::Blend);
                    for pixel in &naming_pixels {
                        if !naming_rect_crop.contains_rect(pixel.rect) {
                            continue;
                        }
                        if pixel.selected {
                            texture_canvas.set_draw_color(Color::RGBA(255, 0, 0, 196));
                            _ = texture_canvas.fill_rect(pixel.rect);
                        } else if pixel.hovered {
                            texture_canvas.set_draw_color(Color::RGBA(255, 0, 0, 96));
                            _ = texture_canvas.fill_rect(pixel.rect);
                        }
                    }
                });

                // Get a destination rectangle that accounts for crop
                let mut dest_rect = screen_space.rect();
                dest_rect.x += dest_rect.w * ((naming_rect_crop.x - naming_rect_zoom.x) as f32 / (naming_rect_zoom.w as f32));
                dest_rect.y += dest_rect.h * ((naming_rect_crop.y - naming_rect_zoom.y) as f32 / (naming_rect_zoom.h as f32));
                dest_rect.w *= (naming_rect_crop.w as f32) / (naming_rect_zoom.w as f32);
                dest_rect.h *= (naming_rect_crop.h as f32) / (naming_rect_zoom.h as f32);
                let dest_rect = Rect::new(dest_rect.x as i32, dest_rect.y as i32, dest_rect.w as u32, dest_rect.h as u32);

                // Draw screenshot behind canvas (so blending works correctly)
                if let Some(screenshot) = &screenshot_texture {
                    _ = main_context.canvas.copy(&screenshot, naming_rect_crop, dest_rect);
                }

                // Copy the overlay texture to the canvas
                _ = main_context.canvas.copy(&overlay_texture, naming_rect_crop, dest_rect);
            },
            NamingSearchState::Found => {
                if let Some(seed) = main_context.run_context.rng_seed() {
                    if let Some(pos) = main_context.run_context.min_rng_position() {
                        let auto_advance_text = if main_context.config.naming_auto_advance_seconds > 0 {
                            "\n(Automatically advancing.)"
                        } else {
                            ""
                        };
                        _ = main_context.font.draw_text(
                            main_context, 
                            &format!("Seed found: {} at position {}{}", seed, pos, auto_advance_text), 
                            screen_space.center_x(), screen_space.center_y(),
                            0.5, 0.0,
                            0, 
                            screen_space.scale() * 2.0, 
                            Color::RGB(255, 255, 255));
                    }
                }
            }
        };

        // Draw hotkeys
        _ = main_context.font.draw_text_bg(
            main_context, 
            &format!("[{}] - Screenshot\n[{}] - Raise window\n[{}] - {}\n[{}] - Focus window\n[LMB] - Drag & toggle pixels\n[RMB] - Switch views", 
                          main_context.config.hotkey_1_name, main_context.config.hotkey_2_name, 
                          main_context.config.hotkey_3_name,
                          if rng_just_found { "Progress to next tool" } else { "Begin search" },
                          main_context.config.hotkey_4_name), 
            screen_space.x_world_to_screen(8.0), screen_space.y_world_to_screen(8.0),
            0.0, 0.0,
            0, 
            screen_space.scale(), 
            Color::RGB(255, 255, 255),
            Color::RGBA(0, 0, 0, 128),
            16.0);

        // Draw text if search failed
        if rng_fail_match_count != -1 {
            _ = main_context.font.draw_text(
                main_context, 
                &format!("Seed search failed: matched {} seeds/patterns", rng_fail_match_count), 
                screen_space.x_world_to_screen(8.0), screen_space.y_world_to_screen(WORLD_HEIGHT as f32 - 8.0),
                0.0, 1.0,
                0, 
                screen_space.scale(), 
                Color::RGB(255, 0, 0));
        }

        // Present latest canvas
        main_context.canvas.present();

        // Sleep until next frame
        frame_timer.end_and_sleep();
    }

    SubProgram::ProgramSelector
}