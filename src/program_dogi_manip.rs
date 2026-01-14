use std::{sync::{atomic::{AtomicBool, Ordering}, Arc, Mutex}, thread};

use defer_rs::defer;
use sdl3::{event::Event, keyboard::Keycode, mouse::MouseButton, pixels::{Color, PixelFormat}, rect::Rect, render::{FPoint, ScaleMode, Texture}, surface::Surface};

use crate::{MainContext, SubProgram, compute_shaders::PointU32, compute_snowball_search::{self, SnowballSearchParameters, SnowballSearchResult}, frame_images, program_common::{FrameTimer, ScreenSpace, draw_circle, fpoint_camera_transform, window_to_world_f32}, rng::{LinearPrecomputedRNG, LinearRNG, PrecomputedRNG, RNG}, server::MessageToSend, snowballs::{SNOWBALLS_ORIGIN_X, SNOWBALLS_ORIGIN_Y, SnowArea}, windowing::{focus_game_window, window_set_focusable}};

#[derive(Clone)]
struct PlacedSnowball {
    pub x: f32,
    pub y: f32,
}

const WORLD_WIDTH: u32 = 640;
const WORLD_HEIGHT: u32 = 480;

pub fn run(main_context: &mut MainContext) -> SubProgram {
    // Visualization stuff
    let mut snow_areas = SnowArea::new_array();
    let mut show_visualization = false;

    // Last server connected state
    let mut last_server_connected = main_context.server_connected.load(Ordering::Relaxed);
    
    // Make sure plugin takes one screenshot, and instantly
    _ = main_context.message_to_send_sender.send(MessageToSend::new_screenshot_mode(true));
    _ = main_context.message_to_send_sender.send(MessageToSend::new_screenshot_start_delay(0));

    // Initialize snowball placement structures
    let mut placed_snowballs: Vec<PlacedSnowball> = Vec::with_capacity(32);
    let mut placing_snowball: Option<PlacedSnowball> = None;
    let mut hovering_snowball: Option<PlacedSnowball> = None;
    let num_to_click = 4;

    // Initialize RNG
    let runner_version = &main_context.config.runner_version;
    let rngs: Arc<Vec<RNG>>;
    let rng_range: u32;
    if main_context.error_returning {
        main_context.error_returning = false;

        // Create precomputed RNGs for all seeds
        let seeds = RNG::calculate_unique_seeds(runner_version.rng_15bit(), runner_version.rng_signed());
        let mut rngs_vec = Vec::with_capacity(seeds.len());
        for seed in seeds {
            rngs_vec.push(RNG::new(seed, runner_version.rng_15bit(), runner_version.rng_signed(), runner_version.rng_old_poly()));
        }
        rngs = Arc::new(rngs_vec);
        rng_range = 0;
    } else {
        let rng_seed = match main_context.run_context.rng_seed() {
            Some(seed) => seed,
            None => {
                main_context.error_message = "Error: Need to first find the RNG seed before using this program.\n\n(Or, press Enter to use the classic version of the tool, requiring a game reload.)";
                main_context.error_return_to = SubProgram::DogiManip;
                return SubProgram::Error;
            }
        };
        let min_rng_position = match main_context.run_context.min_rng_position() {
            Some(pos) => pos,
            None => panic!()
        };
        let mut rng = RNG::new(rng_seed, runner_version.rng_15bit(), runner_version.rng_signed(), runner_version.rng_old_poly());
        rng.skip(min_rng_position + main_context.config.snowball_search_start_offset as usize);

        rng_range = main_context.config.snowball_search_range;
        rngs = Arc::new(vec![rng]);
    }

    // Initialize compute thread
    let rngs_thread = rngs.clone();
    let compute_end_signal = Arc::new(AtomicBool::new(false));
    let compute_end_signal_thread = compute_end_signal.clone();
    let compute_perform_search_signal = Arc::new(AtomicBool::new(false));
    let compute_perform_search_signal_thread = compute_perform_search_signal.clone();
    let compute_preload_completed_signal = Arc::new(AtomicBool::new(false));
    let compute_preload_completed_signal_thread = compute_preload_completed_signal.clone();
    let compute_parameters = Arc::new(Mutex::new(SnowballSearchParameters {
        search_range: 0,
        matching_snowballs: vec![]
    }));
    let compute_parameters_thread = compute_parameters.clone();
    let compute_result = Arc::new(Mutex::new(SnowballSearchResult::new()));
    let compute_result_thread = compute_result.clone();
    let compute_precomputed_rng = Arc::new(Mutex::new(None as Option<PrecomputedRNG>));
    let compute_precomputed_rng_thread = compute_precomputed_rng.clone();
    let compute_join_handle = thread::spawn(move || {
        let rngs = Arc::clone(&rngs_thread);

        // If only one seed, precompute the RNG for it for fast visual display/lookup later
        if rngs.len() == 1 {
            let mut precomputed_rng = compute_precomputed_rng_thread.lock().unwrap();
            *precomputed_rng = Some(rngs[0].clone().precompute(rng_range as usize + 10_000));
            drop(precomputed_rng);
        }

        compute_snowball_search::thread_func(&rngs, rng_range as usize,
            Arc::clone(&compute_end_signal_thread), Arc::clone(&compute_perform_search_signal_thread), 
            Arc::clone(&compute_preload_completed_signal_thread), Arc::clone(&compute_parameters_thread), 
            Arc::clone(&compute_result_thread));
    });
    defer! {
        // End compute thread
        compute_end_signal.store(true, Ordering::Relaxed);
        compute_join_handle.thread().unpark();
    };

    // Set up for drawing
    let camera_position = FPoint::new(0.0, 580.0 - 240.0);
    let camera_scale: f32 = 2.0;
    let final_view_rect = Rect::new(210, 68, 234, 176);
    let actual_world_view = Rect::new(camera_position.x as i32 + (final_view_rect.x / 2), camera_position.y as i32 + (final_view_rect.y / 2), final_view_rect.w as u32 / 2, final_view_rect.h as u32 / 2);
    let circle_draw_offset = main_context.config.runner_version.circle_draw_offset();
    let x_limit = SNOWBALLS_ORIGIN_X + circle_draw_offset;

    // State for whether a search is currently queued, or whether a search is currently in progress
    let mut queued_search = false;
    let mut waiting_for_search_result = false;

    // Screenshot with snowballs to be displayed
    let mut screenshot_texture: Option<Texture> = None;

    // Instructions, once found
    let mut instructions: Option<String> = None;

    // Whether to show an error message due to no patterns matching at all
    let mut no_patterns_matching = false;

    // Begin main loop
    let mut event_pump = main_context.sdl_context.event_pump().unwrap();
    let mut world_texture = main_context.texture_creator
        .create_texture_target(main_context.texture_creator.default_pixel_format(), WORLD_WIDTH, WORLD_HEIGHT)
        .expect("Failed to create texture target");
    world_texture.set_scale_mode(sdl3::render::ScaleMode::Nearest);
    'running: loop {
        // Handle thread errors
        if main_context.panic_occurred.load(Ordering::Relaxed) {
            break;
        }

        // Start frame
        let frame_timer = FrameTimer::start(30);
        let screen_space = ScreenSpace::new(&main_context);

        // Process events
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} => {
                    return SubProgram::None
                },
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                
                Event::MouseButtonDown { mouse_btn: MouseButton::Left, x, y, .. } => {
                    let (world_x, world_y) = window_to_world_f32(x, y, actual_world_view, screen_space.rect());
                    if main_context.config.snowball_immediate_place {
                        // Immediately place snowball
                        placed_snowballs.push(PlacedSnowball {
                            x: f32::max(world_x, x_limit as f32),
                            y: world_y
                        });
                        if placed_snowballs.len() >= num_to_click {
                            // Enough snowballs have been clicked - queue a search
                            queued_search = true;
                        }
                    } else {
                        // Start placing snowball
                        placing_snowball = Some(PlacedSnowball {
                            x: f32::max(world_x, x_limit as f32),
                            y: world_y
                        });
                    }
                },
                Event::MouseButtonUp { mouse_btn: MouseButton::Left, x, y, .. } => {
                    // Place snowball, if one was being placed
                    if let Some(ref mut placing_snowball) = placing_snowball {
                        let (world_x, world_y) = window_to_world_f32(x, y, actual_world_view, screen_space.rect());
                        (placing_snowball.x, placing_snowball.y) = (f32::max(world_x, x_limit as f32), world_y);
                        placed_snowballs.push(placing_snowball.clone());

                        if placed_snowballs.len() >= num_to_click {
                            // Enough snowballs have been clicked - queue a search
                            queued_search = true;
                        }
                    }
                    placing_snowball = None;
                },
                Event::MouseMotion { mousestate, x, y, .. } => {
                    // Move hovering/placing snowball
                    let (world_x, world_y) = window_to_world_f32(x, y, actual_world_view, screen_space.rect());
                    hovering_snowball = Some(PlacedSnowball { 
                        x: f32::max(world_x, x_limit as f32), 
                        y: world_y
                    });
                    if !mousestate.left() {
                        continue;
                    }
                    if let Some(ref mut placing_particle) = placing_snowball {
                        (placing_particle.x, placing_particle.y) = (f32::max(world_x, x_limit as f32), world_y);
                    }
                },
                Event::MouseButtonDown { mouse_btn: MouseButton::Middle, .. } => {
                    // Clear placed snowballs
                    placed_snowballs.clear();
                    no_patterns_matching = false;
                },
                Event::MouseButtonDown { mouse_btn: MouseButton::Right, .. } => {
                    // Undo last snowball placed
                    if !placed_snowballs.is_empty() {
                        placed_snowballs.pop();

                        // Get rid of text once too few to search
                        if placed_snowballs.len() < num_to_click {
                            no_patterns_matching = false;
                        }
                    }
                }
                _ => {}
            }
        }

        // Handle queued searches
        if queued_search && !waiting_for_search_result {
            queued_search = false;

            // Create list of matching snowballs to pass to search
            let mut matching_snowballs: Vec<PointU32> = Vec::with_capacity(32);
            for snowball in placed_snowballs.iter() {
                matching_snowballs.push(PointU32::new(
                    (f32::round(snowball.x) as i32 - SNOWBALLS_ORIGIN_X - circle_draw_offset) as i16, 
                    (f32::round(snowball.y) as i32 - SNOWBALLS_ORIGIN_Y - circle_draw_offset) as i16));
            }

            // Begin search
            *compute_parameters.lock().unwrap() = SnowballSearchParameters {
                search_range: if rngs.len() == 1 { rng_range } else { rngs.len() as u32 },
                matching_snowballs
            };
            compute_perform_search_signal.store(true, Ordering::Relaxed);
            compute_join_handle.thread().unpark();
            waiting_for_search_result = true;
        }

        // Check for incoming snowball search results
        if waiting_for_search_result && !compute_perform_search_signal.load(Ordering::Relaxed) {
            waiting_for_search_result = false;

            let search_result = compute_result.lock().unwrap();
            if search_result.match_count == 1 {
                // Singular match! Figure out past RNG...
                println!("Matched position is {}", search_result.single_matched_position);

                // Use precomputed RNG to get the step count from just before the match, and create visualization
                let step_count: u32;
                if rngs.len() == 1 {
                    // Get RNG from single seed, at correct position
                    let precomputed_rng = compute_precomputed_rng.lock().unwrap();
                    let mut lprng = LinearPrecomputedRNG::new(precomputed_rng.as_ref().unwrap(), (search_result.single_matched_position - 1) as usize);

                    // Use precomputed RNG for step count
                    step_count = f64::round_ties_even(lprng.next_f64(30.0)) as u32;

                    // Use the same precomputed RNG to simulate the snowballs for a visualization
                    snow_areas = SnowArea::new_array();
                    SnowArea::simulate_array(&mut snow_areas, &mut lprng);
                } else {
                    // Get RNG from one of the seeds, at initial position (classic mode)
                    let mut rng = rngs[search_result.single_matched_position as usize].clone();

                    // Use regular RNG for step count
                    _ = rng.next_u32();
                    step_count = f64::round_ties_even(rng.next_f64(30.0)) as u32;

                    // Use the same regular RNG to simulate the snowballs for a visualization
                    snow_areas = SnowArea::new_array();
                    SnowArea::simulate_array(&mut snow_areas, &mut rng);
                }
                show_visualization = true;

                // Create instructions
                let menu_buffer = step_count % 2 == 1;
                let up_down_times = (step_count / 2) + (if menu_buffer { 1 } else { 0 });
                instructions = Some(match up_down_times {
                    0 => "NO UP/DOWN".to_owned(),
                    times => if menu_buffer {
                        format!("Menu buffer\nUp/down {} time{}", times, if times != 1 { "s" } else { "" })
                    } else {
                        format!("Up/down {} time{}", times, if times != 1 { "s" } else { "" })
                    }
                });

                no_patterns_matching = false;
            } else {
                println!("Match count = {}, data = {}", search_result.match_count, search_result.single_matched_position);

                // Display extra message if no patterns matched at all...
                if search_result.match_count == 0 {
                    no_patterns_matching = true;
                }
            }
        }

        // Draw
        main_context.canvas.set_draw_color(Color::RGB(0, 0, 0));
        main_context.canvas.set_blend_mode(sdl3::render::BlendMode::Blend);
        main_context.canvas.clear();
        _ = main_context.canvas.with_texture_canvas(&mut world_texture, |texture_canvas| {
            // Clear the world texture
            texture_canvas.set_draw_color(Color::RGBA(0, 0, 0, 255));
            texture_canvas.clear();

            // Draw the current screenshot
            if let Some(screenshot_texture) = &screenshot_texture {
                _ = texture_canvas.copy(&screenshot_texture, 
                    Rect::new(0, 0, screenshot_texture.width(), screenshot_texture.height()), 
                    Rect::new(final_view_rect.x, final_view_rect.y, final_view_rect.w as u32, final_view_rect.h as u32));
            }

            if show_visualization {
                // Draw visualization snowballs
                for snow_area in snow_areas.iter() {
                    for snowball in snow_area.snowballs.iter() {
                        let point = fpoint_camera_transform(FPoint::new(snowball.x + (circle_draw_offset as f32), snowball.y + (circle_draw_offset as f32)), camera_position, camera_scale);
                        draw_circle(texture_canvas, point.x, point.y, 2.8 * camera_scale, 24, Color::RGBA(0, 0, 255, 128));
                    }
                }
            }

            // Draw black over unnecessary part
            texture_canvas.set_draw_color(Color::RGB(0, 0, 0));
            _ = texture_canvas.fill_rect(Rect::new(0, 0, (x_limit as u32 * 2) - 5, 480));
            
            // Draw placed/placing/hovering snowballs
            if let Some(placing_snowball) = &placing_snowball {
                let point = fpoint_camera_transform(FPoint::new(placing_snowball.x, placing_snowball.y), camera_position, camera_scale);
                draw_circle(texture_canvas, point.x, point.y, 2.8 * camera_scale, 24, Color::RGB(255, 0, 0));
            }
            for placed_snowball in &placed_snowballs {
                let point = fpoint_camera_transform(FPoint::new(placed_snowball.x, placed_snowball.y), camera_position, camera_scale);
                draw_circle(texture_canvas, point.x, point.y, 2.8 * camera_scale, 24, Color::RGB(128, 0, 0));
            }
            if let Some(hovering_snowball) = &hovering_snowball {
                let point = fpoint_camera_transform(FPoint::new(hovering_snowball.x, hovering_snowball.y), camera_position, camera_scale);
                draw_circle(texture_canvas, point.x, point.y, 2.8 * camera_scale, 24, Color::RGBA(255, 0, 0, 64));
            }
        });

        // Copy the world texture to the canvas
        _ = main_context.canvas.copy(&world_texture, final_view_rect, screen_space.irect());

        // Draw instructions
        if let Some(instructions) = instructions.clone() {
            _ = main_context.font.draw_text(
                main_context, 
                instructions.as_str(), 
                screen_space.x_world_to_screen(16.0), screen_space.y_world_to_screen(164.0),
                0.0, 0.0,
                0, 
                screen_space.scale() * 2.0, 
                Color::RGB(255, 255, 255));
        }

        // Draw hotkeys
        _ = main_context.font.draw_text_bg(
            main_context, 
            &format!("[{}] - Screenshot & raise\n[{}] - Focus window\n[LMB] - Place snowballs\n[RMB] - Undo a snowball\n[MMB] - Clear all snowballs", 
                          main_context.config.hotkey_1_name, main_context.config.hotkey_4_name), 
            screen_space.x_world_to_screen(8.0), screen_space.y_world_to_screen(8.0),
            0.0, 0.0,
            0, 
            screen_space.scale(), 
            Color::RGB(255, 255, 255),
            Color::RGBA(0, 0, 0, 128),
            16.0);

        // Draw text if no patterns matched at all
        if no_patterns_matching {
            _ = main_context.font.draw_text(
                main_context, 
                "No matching patterns found!", 
                screen_space.x_world_to_screen(8.0), screen_space.y_world_to_screen(WORLD_HEIGHT as f32 - 32.0),
                0.0, 1.0,
                0, 
                screen_space.scale(), 
                Color::RGB(255, 0, 0));
        }
        
        // Draw text if preloading is still happening
        if !compute_preload_completed_signal.load(Ordering::Relaxed) {
            _ = main_context.font.draw_text(
                main_context, 
                "Preloading snowball data...", 
                screen_space.x_world_to_screen(8.0), screen_space.y_world_to_screen(WORLD_HEIGHT as f32 - 8.0),
                0.0, 1.0,
                0, 
                screen_space.scale(), 
                Color::RGB(128, 128, 128));
        } else {
            _ = main_context.font.draw_text(
                main_context, 
                "Preload complete.", 
                screen_space.x_world_to_screen(8.0), screen_space.y_world_to_screen(WORLD_HEIGHT as f32 - 8.0),
                0.0, 1.0,
                0, 
                screen_space.scale(), 
                Color::RGB(128, 128, 128));
        }

        // Present latest canvas
        main_context.canvas.present();

        // Check for any incoming hotkeys
        for hotkey_id in main_context.hotkey_receiver.try_iter() {
            match hotkey_id {
                0 => {
                    // Screenshots
                    let window = main_context.canvas.window_mut();
                    if main_context.config.mouse_warps {
                        main_context.sdl_context.mouse().warp_mouse_in_window(window, window.size().0 as f32 / 2.0, window.size().1 as f32 / 2.0);
                    }
                    window_set_focusable(window, false);
                    window.sync();
                    focus_game_window();
                },
                3 => {
                    // Focus window
                    let window = main_context.canvas.window_mut();
                    window_set_focusable(window, true);
                    window.raise();
                }
                _ => {}
            }
        }

        // Check for incoming screenshot from the server
        let mut local_screenshot_data = main_context.screenshot_data.lock().unwrap();
        if local_screenshot_data.len() >= 1 {
            // Hide old stuff
            placed_snowballs.clear();
            show_visualization = false;
            no_patterns_matching = false;

            // Get screenshot data
            let screenshot_data = &local_screenshot_data.pop().unwrap();

            // Preprocess image
            let mut cleaned_data: Vec<u8> = Vec::with_capacity((final_view_rect.w * final_view_rect.h * 4) as usize);
            frame_images::clear_unwanted_pixels_snowballs(&mut cleaned_data, &screenshot_data, final_view_rect);
            
            // Create texture
            let surface = Surface::from_data(&mut cleaned_data, 
                final_view_rect.w as u32, final_view_rect.h as u32, final_view_rect.w as u32 * 4, PixelFormat::RGBA32).unwrap();
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
        
        // Sleep until next frame
        frame_timer.end_and_sleep();
    }

    SubProgram::ProgramSelector
}
