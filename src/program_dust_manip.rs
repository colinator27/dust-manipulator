extern crate sdl3;

use compute_dust_search::{DustSearchMode, DustSearchParameters, DustSearchResult};
use compute_shaders::PointU32;
use defer_rs::defer;
use encounter_data::{Battlegroup, Encounterer};
use image::ImageReader;
use manip_data::MANIP_SETUPS_CORE;
use sdl3::mouse::MouseButton;
use sdl3::pixels::PixelFormat;
use sdl3::rect::{Point, Rect};
use sdl3::render::{Canvas, FRect, ScaleMode, Texture, TextureCreator};
use sdl3::surface::Surface;
use sdl3::sys::pixels::SDL_PIXELFORMAT_RGBA32;
use sdl3::video::{Window, WindowContext};
use sdl3::pixels::Color;
use sdl3::event::Event;
use sdl3::keyboard::Keycode;
use sdl3_ttf_sys::ttf::{TTF_HORIZONTAL_ALIGN_LEFT, TTF_HORIZONTAL_ALIGN_RIGHT};
use server::ScreenshotData;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

use rng::RNG;
use dust::{DustAnimation, DustSearchConfig};

use crate::windowing::window_set_focusable;
use crate::program_common::{rect_from_texture, rect_to_frect, window_to_world, FrameTimer, ScreenSpace};
use crate::rng::LinearRNG;
use crate::server::MessageToSend;
use crate::{compute_dust_search, compute_shaders, dust, encounter_data, frame_images, manip_data, windowing, program_common, rng, server, util, MainContext, SubProgram};

#[derive(Clone)]
struct PlacedDustParticle {
    pub x: i32,
    pub y: i32,
    pub xscale: u32
}

impl PlacedDustParticle {
    pub fn draw(&self, texture_canvas: &mut Canvas<Window>) {
        _ = texture_canvas.draw_rect(FRect {
            x: self.x as f32,
            y: self.y as f32,
            w: 2.0 * self.xscale as f32,
            h: 2.0
        });
    }
}

struct DustFramePair<'a> {
    pub texture: Texture<'a>,
    pub rect: Rect,
    pub first_frame_index: usize,
    pub hovered: bool
}

#[derive(PartialEq)]
enum DustManipState {
    Waiting,
    SelectingFrame,
    PlacingParticles,
    FoundPosition
}

struct DustManipContext {
    pub search_config: DustSearchConfig,
    pub search_mode: DustSearchMode
}
impl DustManipContext {
    pub fn new(main_context: &MainContext, search_config: DustSearchConfig, search_mode: DustSearchMode) -> Self {
        let mut res = DustManipContext {
            search_config,
            search_mode
        };
        res.update_screenshot_delay_time(main_context);
        res
    }
    pub fn update_screenshot_delay_time(&mut self, main_context: &MainContext) {
        // Instantiate the dust animation so we can figure out how many frames there are (for timing purposes)
        let animation: DustAnimation = self.search_config.dust_data.create_animation();
        let num_frames_early = match self.search_mode {
            DustSearchMode::LastFrame | DustSearchMode::LastFrameEarly => 1,
            DustSearchMode::SecondToLastFrame | DustSearchMode::SecondToLastFrameEarly => 2
        };
        let time_in_frames = animation.get_length() - num_frames_early;

        // Send signal to client for the new delay time
        let new_delay_time = (time_in_frames as f32 * (1000.0 / 30.0)) as u32;
        _ = main_context.message_to_send_sender.send(MessageToSend::new_screenshot_start_delay(new_delay_time));
    }
}

const EXTRA_RAISE_DELAY_MS: u32 = 450;

fn get_new_screenshot<'a>(texture_creator: &'a TextureCreator<WindowContext>, screenshot_data: &mut ScreenshotData, world_view: Rect, search_mode: DustSearchMode) -> Texture<'a> {
    // Preprocess image
    let mut cleaned_data: Vec<u8> = Vec::with_capacity((world_view.w * world_view.h * 4) as usize);
    frame_images::clear_unwanted_pixels_dust(&mut cleaned_data, &screenshot_data, world_view, match search_mode {
        DustSearchMode::LastFrame | DustSearchMode::SecondToLastFrame => false,
        DustSearchMode::LastFrameEarly | DustSearchMode::SecondToLastFrameEarly => true
    });
    
    // Create texture
    let surface = Surface::from_data(&mut cleaned_data, world_view.w as u32, world_view.h as u32, world_view.w as u32 * 4, 
                                                  PixelFormat::from(SDL_PIXELFORMAT_RGBA32.0 as i64)).unwrap();
    let mut texture = Texture::from_surface(&surface, &texture_creator).unwrap();
    texture.set_scale_mode(ScaleMode::Nearest);
    texture
}

fn set_new_search_config(main_context: &MainContext, context: &mut DustManipContext, new_config: DustSearchConfig) {
    // Set the new config
    context.search_config = new_config;

    // Update the current mode
    context.search_mode = context.search_config.dust_data.search_mode;

    // Update screenshot delay time
    context.update_screenshot_delay_time(main_context);
}

const WORLD_WIDTH: u32 = 640;
const WORLD_HEIGHT: u32 = 480;

pub fn run(main_context: &mut MainContext) -> SubProgram {
    // Initialize dust manip state
    let battlegroup_order = [
        Battlegroup::KnightKnight_Madjick,
        Battlegroup::FinalFroggit_Astigmatism_Whimsalot
    ];
    let mut curr_encounterer: Encounterer = Encounterer::Core;
    let mut curr_battlegroup: Battlegroup = battlegroup_order[0];
    let search_config: DustSearchConfig = curr_battlegroup.get_dust_config();
    //let search_config = encounter_data::get_debug_search_config();
    let search_mode = search_config.dust_data.search_mode;
    let mut context = DustManipContext::new(&main_context, search_config, search_mode);
    let num_to_click = 2;
    let mut battlegroup_order_pos = 0;
    let mut num_attacks: i32 = 1; // used to track # of attacks besides the last one
    let mut leveled_up = false;
    let mut added_level_up_delay_already = false;

    let mut search_anim: Option<DustAnimation> = None;
    let mut debug_anim: Option<DustAnimation> = None;

    // Initialize RNG
    let runner_version = &main_context.config.runner_version;
    let rng_seed = match main_context.run_context.rng_seed() {
        Some(seed) => seed,
        None => {
            main_context.error_message = "Error: Need to first find the RNG seed before using this program.";
            return SubProgram::Error;
        }
    };
    let min_rng_position = match main_context.run_context.min_rng_position() {
        Some(pos) => pos,
        None => panic!()
    };
    let mut rng = RNG::new(rng_seed, runner_version.rng_15bit(), runner_version.rng_signed(), runner_version.rng_old_poly());
    rng.skip(min_rng_position);

    let num_to_compute = 500_000;
    let prng = Arc::new(rng.precompute(num_to_compute));

    // Initialize compute thread
    let compute_end_signal = Arc::new(AtomicBool::new(false));
    let compute_end_signal_thread = compute_end_signal.clone();
    let compute_perform_search_signal = Arc::new(AtomicBool::new(false));
    let compute_perform_search_signal_thread = compute_perform_search_signal.clone();
    let prng_thread = prng.clone();
    let compute_parameters = Arc::new(Mutex::new(DustSearchParameters {
        search_mode: DustSearchMode::LastFrame,
        search_range: 0,
        last_frame_rng_offset: 0,
        last_frame_particle_count: 0,
        second_last_frame_particle_count: 0,
        initial_rng_skip_amount: 0,
        matching_particles: vec![],
        initial_particles: vec![],
    }));
    let compute_parameters_thread = compute_parameters.clone();
    let compute_result = Arc::new(Mutex::new(DustSearchResult { match_count: 0, single_matched_position: 0 }));
    let compute_result_thread = compute_result.clone();
    let compute_join_handle = thread::spawn(move || {
        compute_dust_search::thread_func(
            Arc::clone(&compute_end_signal_thread), Arc::clone(&compute_perform_search_signal_thread), 
            Arc::clone(&prng_thread), Arc::clone(&compute_parameters_thread), Arc::clone(&compute_result_thread));
    });
    defer! {
        // End compute thread
        compute_end_signal.store(true, Ordering::Relaxed);
        compute_join_handle.thread().unpark();
    };

    // Initialize screenshot structures
    let mut screenshots: Vec<ScreenshotData> = Vec::with_capacity(32);
    let mut selected_screenshot_texture: Option<Texture> = None;
    let mut selected_screenshot: usize = 0;

    // Initialize particle placement structures
    let mut placed_particles: Vec<PlacedDustParticle> = Vec::with_capacity(32);
    let mut placing_particle: Option<PlacedDustParticle> = None;
    let mut hovering_particle: Option<PlacedDustParticle> = None;

    // State for whether a search is currently queued, or whether a search is currently in progress
    let mut queued_search = false;
    let mut waiting_for_search_result = false;

    // Initialize frame pairs, loading the template image from the assets
    let mut dust_search_frame_pairs: Vec<DustFramePair> = Vec::with_capacity(100);
    let dust_search_frame_pair_image = 
        ImageReader::open(util::get_exe_directory().join("./assets/dust_search_frame_pair.png"))
            .expect("Failed to open image").decode().expect("Failed to decode image");
    let dust_search_frame_pair_image_data = dust_search_frame_pair_image.to_rgba8().to_vec();

    // Main state of the sub-program
    let mut dust_manip_state = DustManipState::Waiting;

    // String to display for instructions
    let mut dust_manip_string: Option<String> = None;

    // Times (in milliseconds) for raising this program window, and for preventing focusing of this window, respectively
    let mut raise_window_time: Option<u64> = None;
    let mut focus_window_buffer_time: Option<u64> = None;

    // Last server connected state
    let mut last_server_connected = main_context.server_connected.load(Ordering::Relaxed);

    // Whether debug animation should be shown
    let mut show_anim = false;

    // Make sure plugin takes more than one screenshot... (the delay is managed by the manip context)
    _ = main_context.message_to_send_sender.send(MessageToSend::new_screenshot_mode(false));

    // Start main loop
    let mut world_texture = main_context.texture_creator
        .create_texture_target(main_context.texture_creator.default_pixel_format(), WORLD_WIDTH, WORLD_HEIGHT)
        .expect("Failed to create texture target");
    world_texture.set_scale_mode(sdl3::render::ScaleMode::Nearest);
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
                    return SubProgram::None
                },
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                Event::KeyDown { keycode: Some(Keycode::Space), .. } => {
                    let mut new_debug_anim = context.search_config.dust_data.create_animation();
                    let frame_end_offset: usize = match context.search_mode {
                        DustSearchMode::LastFrame | DustSearchMode::LastFrameEarly => 1,
                        DustSearchMode::SecondToLastFrame | DustSearchMode::SecondToLastFrameEarly => 2
                    };
                    let early_delay_offset: usize = match context.search_mode {
                        DustSearchMode::LastFrame | DustSearchMode::SecondToLastFrame => 0,
                        DustSearchMode::LastFrameEarly | DustSearchMode::SecondToLastFrameEarly => 1
                    };

                    new_debug_anim.set_start_process_frame(match context.search_mode {
                        DustSearchMode::LastFrame | DustSearchMode::LastFrameEarly => new_debug_anim.get_frame_count() - 1,
                        DustSearchMode::SecondToLastFrame | DustSearchMode::SecondToLastFrameEarly => new_debug_anim.get_frame_count() - 2
                    });
                    let test_rng_position = 0;
                    new_debug_anim.start_animating(&prng, test_rng_position);

                    let mut num_updates = 0;
                    while num_updates < ((new_debug_anim.get_length() - frame_end_offset) - early_delay_offset) {
                        new_debug_anim.update();
                        num_updates += 1;
                    }

                    debug_anim = Some(new_debug_anim.clone());
                },
                Event::KeyDown { keycode: Some(Keycode::Z), .. } => {
                    if let Some(debug_anim) = &mut debug_anim {
                        debug_anim.update();
                    }
                },
                Event::KeyDown { keycode: Some(Keycode::Left), .. } => {
                    if dust_manip_state == DustManipState::PlacingParticles {
                        if screenshots.len() > 0 {
                            if selected_screenshot == 0 {
                                selected_screenshot = screenshots.len() - 1;
                            } else {
                                selected_screenshot = usize::min(screenshots.len() - 1, selected_screenshot - 1);
                            }
                            drop(selected_screenshot_texture);
                            selected_screenshot_texture = Some(get_new_screenshot(&main_context.texture_creator, &mut screenshots[selected_screenshot], 
                                context.search_config.view_rect, context.search_mode));
                        }
                    }
                },
                Event::KeyDown { keycode: Some(Keycode::Right), .. } => {
                    if dust_manip_state == DustManipState::PlacingParticles {
                        if screenshots.len() > 0 {
                            selected_screenshot = (selected_screenshot + 1) % screenshots.len();
                            drop(selected_screenshot_texture);
                            selected_screenshot_texture = Some(get_new_screenshot(&main_context.texture_creator, &mut screenshots[selected_screenshot],
                                context.search_config.view_rect, context.search_mode));
                        }
                    }
                },
                Event::MouseButtonDown { mouse_btn: MouseButton::Left, x, y, .. } => {
                    match dust_manip_state {
                        DustManipState::SelectingFrame => {
                            let (selector_x, selector_y) = window_to_world(x, y, Rect::new(0, 0, WORLD_WIDTH, WORLD_HEIGHT), screen_space.rect());
                            for pair in dust_search_frame_pairs.iter_mut() {
                                if pair.rect.contains_point(Point::new(selector_x, selector_y)) {
                                    selected_screenshot = pair.first_frame_index;
                                    drop(selected_screenshot_texture);
                                    context.search_mode = context.search_mode.to_normal();
                                    selected_screenshot_texture = Some(get_new_screenshot(&main_context.texture_creator, &mut screenshots[selected_screenshot], context.search_config.view_rect, context.search_mode));
                                    dust_manip_state = DustManipState::PlacingParticles;
                                    break;
                                }
                            }
                        },
                        DustManipState::PlacingParticles => {
                            let (world_x, world_y) = window_to_world(x, y, context.search_config.view_rect, screen_space.rect());
                            placing_particle = Some(PlacedDustParticle {
                                x: world_x - 1,
                                y: world_y - 1,
                                xscale: 1
                            });
                        }
                        _ => {}
                    }
                },
                Event::MouseButtonUp { mouse_btn: MouseButton::Left, x, y, .. } => {
                    if let Some(ref mut placing_particle) = placing_particle {
                        // TODO: maybe allow for changing xscale somehow...
                        let (world_x, world_y) = window_to_world(x, y, context.search_config.view_rect, screen_space.rect());
                        (placing_particle.x, placing_particle.y) = (world_x - 1, world_y - 1);
                        placed_particles.push(placing_particle.clone());

                        if placed_particles.len() >= num_to_click {
                            // Start searching now!
                            queued_search = true;
                        }
                    }
                    placing_particle = None;
                },
                Event::MouseMotion { mousestate, x, y, .. } => {
                    match dust_manip_state {
                        DustManipState::SelectingFrame => {
                            let (selector_x, selector_y) = window_to_world(x, y, Rect::new(0, 0, WORLD_WIDTH, WORLD_HEIGHT), screen_space.rect());
                            for pair in dust_search_frame_pairs.iter_mut() {
                                pair.hovered = pair.rect.contains_point(Point::new(selector_x, selector_y));
                            }
                        },
                        DustManipState::PlacingParticles => {
                            let (world_x, world_y) = window_to_world(x, y, context.search_config.view_rect, screen_space.rect());
                            hovering_particle = Some(PlacedDustParticle { 
                                x: world_x - 1, 
                                y: world_y - 1, 
                                xscale: 1 
                            });
                            if !mousestate.left() {
                                continue;
                            }
                            if let Some(ref mut placing_particle) = placing_particle {
                                // TODO: maybe allow for changing xscale somehow...
                                (placing_particle.x, placing_particle.y) = (world_x - 1, world_y - 1);
                            }
                        },
                        _ => {}
                    }
                },
                Event::MouseButtonDown { mouse_btn: MouseButton::Right, x, y, .. } => {
                    match dust_manip_state {
                        DustManipState::SelectingFrame => {
                            let (selector_x, selector_y) = window_to_world(x, y, Rect::new(0, 0, WORLD_WIDTH, WORLD_HEIGHT), screen_space.rect());
                            for pair in dust_search_frame_pairs.iter_mut() {
                                if pair.rect.contains_point(Point::new(selector_x, selector_y)) {
                                    selected_screenshot = pair.first_frame_index;
                                    drop(selected_screenshot_texture);
                                    context.search_mode = context.search_mode.to_early();
                                    selected_screenshot_texture = Some(get_new_screenshot(&main_context.texture_creator, &mut screenshots[selected_screenshot], context.search_config.view_rect, context.search_mode));
                                    dust_manip_state = DustManipState::PlacingParticles;
                                    break;
                                }
                            }
                        },
                        DustManipState::PlacingParticles => {
                            placed_particles.clear();
                        },
                        _ => {}
                    }
                },
                Event::KeyDown { keycode: Some(Keycode::V), .. } => {
                    show_anim = !show_anim;
                },
                Event::KeyDown { keycode: Some(Keycode::Backspace), .. } => {
                    if dust_manip_state == DustManipState::FoundPosition {
                        dust_manip_state = DustManipState::PlacingParticles;
                        selected_screenshot_texture = Some(get_new_screenshot(&main_context.texture_creator, &mut screenshots[selected_screenshot], context.search_config.view_rect, context.search_mode));
                    }
                },
                Event::KeyDown { keycode: Some(Keycode::Return), .. } => {
                    queued_search = true;
                },
                _ => {}
            }
        }

        // Check for incoming dust search results
        if let Some(search_anim) = &search_anim {
            if waiting_for_search_result && !compute_perform_search_signal.load(Ordering::Relaxed) {
                waiting_for_search_result = false;

                let search_result = compute_result.lock().unwrap();
                if search_result.match_count == 1 {
                    // Singular match! Predict future RNG...
                    let predicted_pos = search_result.single_matched_position as usize + search_anim.get_after_battle_rng_calls(match leveled_up {
                        true => context.search_config.text_length_lvup,
                        false => context.search_config.text_length
                    });
                    if battlegroup_order_pos < battlegroup_order.len() - 1 {
                        battlegroup_order_pos += 1;
                    }
                    println!("Matched position is {}", search_result.single_matched_position);
                    println!("Predicted position is {}", predicted_pos);
                    //for i in 0..4 {
                    //    println!("RNG value {} is {}", i, prng.get_f64(100.0, predicted_pos + i));
                    //    println!("Encounter {} is {}", i, Encounterer::Core.get_battlegroup_at_pos(&prng, predicted_pos + i).get_name());
                    //}
                    let mut str: String = "(unlucky, no good setup)".to_owned();
                    //str += &format!("Next encounter:\n{}", Encounterer::Core.get_battlegroup_at_pos(&prng, predicted_pos + 3).get_name())[0..];
                    /*
                    let mut attempt_counter = 0;
                    while attempt_counter < 30 {
                        let bg = Encounterer::Core.get_battlegroup_at_pos(&prng., predicted_pos + 3 + attempt_counter);
                        if bg == battlegroup_order[battlegroup_order_pos] {
                            str += &format!("\n\nRe-enter {} time(s) for {}", attempt_counter, battlegroup_order[battlegroup_order_pos].get_name())[0..];
                            break;
                        }
                        attempt_counter += 1;
                    }
                    dust_manip_string = Some(str);
                    */
                    for setup in MANIP_SETUPS_CORE {
                        let bg = curr_encounterer.get_battlegroup_at_pos(&prng, predicted_pos + 3 + setup.rng_amount);
                        if bg == battlegroup_order[battlegroup_order_pos] {
                            println!("Setup targets position {}", predicted_pos + setup.rng_amount);
                            str = setup.text.to_owned();
                            break;
                        }
                    }
                    dust_manip_string = Some(str);

                    // Make this program window unfocusable (for a fixed amount of buffer time), and focus the game window if possible
                    if window_set_focusable(main_context.canvas.window_mut(), false) {
                        focus_window_buffer_time = Some(sdl3::timer::ticks() + 2000);
                    }
                    windowing::focus_game_window();

                    num_attacks = 1;
                    leveled_up = false;
                    added_level_up_delay_already = false;
                    dust_manip_state = DustManipState::FoundPosition;
                    curr_battlegroup = battlegroup_order[battlegroup_order_pos];
                    set_new_search_config(main_context, &mut context, curr_battlegroup.get_dust_config());
                    selected_screenshot_texture = None;
                } else {
                    println!("Match count = {}, data = {}", search_result.match_count, search_result.single_matched_position);
                }
            }
        }

        // Check for any incoming hotkeys
        for hotkey_id in main_context.hotkey_receiver.try_iter() {
            match hotkey_id {
                0 => {
                    // Screenshots - start raise window timer
                    let text_time = if leveled_up { 
                        added_level_up_delay_already = true;
                        context.search_config.text_length_lvup as f32 * (1000.0 / 30.0) 
                    } else {
                        context.search_config.text_length as f32 * (1000.0 / 30.0) 
                    };
                    raise_window_time = Some(sdl3::timer::ticks() + EXTRA_RAISE_DELAY_MS as u64 + text_time as u64);
                }
                1 => {
                    // Increase attack counter
                    num_attacks += 1;

                    // Reset if you go too high accidentally
                    if num_attacks >= 5 {
                        num_attacks = 1;
                    }
                }
                2 => {
                    // Cycle actual random encounter
                    curr_battlegroup = curr_encounterer.cycle_random_battlegroups(curr_battlegroup);
                    set_new_search_config(main_context, &mut context, curr_battlegroup.get_dust_config());
                    if selected_screenshot < screenshots.len() {
                        drop(selected_screenshot_texture);
                        selected_screenshot_texture = Some(get_new_screenshot(&main_context.texture_creator, &mut screenshots[selected_screenshot], context.search_config.view_rect, context.search_mode));
                    }
                }
                3 => {
                    // Level up toggle
                    leveled_up = !leveled_up;

                    // Add extra time to unfocus timer, if one is currently active
                    if leveled_up && !added_level_up_delay_already {
                        if let Some(current_raise_window_time) = raise_window_time {
                            added_level_up_delay_already = true;
                            
                            let time_leveled_up = context.search_config.text_length_lvup as f32 * (1000.0 / 30.0);
                            let time_not_leveled_up = context.search_config.text_length as f32 * (1000.0 / 30.0);
                            let difference = (time_leveled_up - time_not_leveled_up) as u64;
                            raise_window_time = Some(current_raise_window_time + difference);
                        }
                    }
                }
                _ => {}
            }
        }

        // Perform search if queued
        if queued_search && !waiting_for_search_result {
            queued_search = false;

            let mut initial_particles: Vec<PointU32> = Vec::with_capacity(32);
            let mut initial_particles_last_frame_count = 0;
            let mut initial_particles_second_last_frame_count = 0;

            let mut new_search_anim = context.search_config.dust_data.create_animation();
            new_search_anim.compute_frame_rng_offsets();
            for particle in new_search_anim.get_frames().last().unwrap().iter() {
                initial_particles.push(PointU32::new(particle.get_x() as i16 - (num_attacks as i16), particle.get_y() as i16));
                initial_particles_last_frame_count += 1;
            }
            match context.search_mode {
                DustSearchMode::SecondToLastFrame | DustSearchMode::SecondToLastFrameEarly => {
                    for particle in new_search_anim.get_frames().get(new_search_anim.get_frame_count() - 2).unwrap().iter() {
                        initial_particles.push(PointU32::new(particle.get_x() as i16 - (num_attacks as i16), particle.get_y() as i16));
                        initial_particles_second_last_frame_count += 1;
                    }
                }
                DustSearchMode::LastFrame | DustSearchMode::LastFrameEarly => {}
            }

            let matching_particles: Vec<PointU32> = placed_particles.iter().map(|p| PointU32::new(p.x as i16, p.y as i16)).collect();

            /*
            if let Some(debug_anim) = &debug_anim {
                for particle in debug_anim.get_frames().last().unwrap().iter() {
                    println!("Actual matching particle at ({}, {}) rounded from ({}, {})",
                        f32::round(particle.get_x() - num_attacks as f32 - (1.0 / 512.0)) as i16, f32::round(particle.get_y() - (1.0 / 512.0)) as i16, particle.get_x() - num_attacks as f32, particle.get_y());
                }
                assert!(new_search_anim.get_frame_rng_offset(new_search_anim.get_frame_count() - 1) == debug_anim.get_frame_rng_offset(debug_anim.get_frame_count() - 1));
                match context.search_mode {
                    DustSearchMode::SecondToLastFrame | DustSearchMode::SecondToLastFrameEarly => {
                        for particle in debug_anim.get_frames().get(debug_anim.get_frame_count() - 2).unwrap().iter() {
                            println!("Actual matching particle at ({}, {}) rounded from ({}, {})",
                                f32::round(particle.get_x() - num_attacks as f32 - (1.0 / 512.0)) as i16, f32::round(particle.get_y() - (1.0 / 512.0)) as i16, particle.get_x() - num_attacks as f32, particle.get_y());
                        }

                        assert!(new_search_anim.get_frame_rng_offset(new_search_anim.get_frame_count() - 2) == debug_anim.get_frame_rng_offset(debug_anim.get_frame_count() - 2));
                        println!("second to last frame rng offset is {}", new_search_anim.get_frame_rng_offset(new_search_anim.get_frame_count() - 2) as u32);
                    }
                    DustSearchMode::LastFrame | DustSearchMode::LastFrameEarly => {}
                }
            }
            for p in &matching_particles {
                println!("Matching particle is at ({}, {})", p.get_x(), p.get_y());
            }
            */

            let frame_end_offset: usize = match context.search_mode {
                DustSearchMode::LastFrame | DustSearchMode::LastFrameEarly => 1,
                DustSearchMode::SecondToLastFrame | DustSearchMode::SecondToLastFrameEarly => 2
            };
            let initial_rng_skip_amount: u32 = 2 * (new_search_anim.get_frame_count() - frame_end_offset) as u32;

            if frame_end_offset == 2 {
                //println!("RNG skip amount between last two frames = {}", initial_rng_skip_amount);
                //println!("expected second to last RNG frame offset = {}", new_search_anim.get_frame_rng_offset(new_search_anim.get_frame_count() - 2));
                //println!("actual second to last RNG frame offset = {}", new_search_anim.get_frame_rng_offset(new_search_anim.get_frame_count() - 1) as u32 - ((initial_particles_second_last_frame_count * 2) + initial_rng_skip_amount));
                assert!(new_search_anim.get_frame_rng_offset(new_search_anim.get_frame_count() - 1) as u32 - ((initial_particles_second_last_frame_count * 2) + initial_rng_skip_amount) ==
                        new_search_anim.get_frame_rng_offset(new_search_anim.get_frame_count() - 2) as u32);
            }

            *compute_parameters.lock().unwrap() = DustSearchParameters {
                search_range: (num_to_compute - new_search_anim.get_total_rng_calls() - 10_000) as u32,
                last_frame_rng_offset: new_search_anim.get_frame_rng_offset(new_search_anim.get_frame_count() - 1) as u32,
                matching_particles,
                initial_particles: initial_particles.clone(),
                last_frame_particle_count: initial_particles_last_frame_count,
                second_last_frame_particle_count: initial_particles_second_last_frame_count,
                initial_rng_skip_amount,
                search_mode: context.search_mode
            };

            search_anim = Some(new_search_anim);
            waiting_for_search_result = true;
            compute_perform_search_signal.store(true, Ordering::Relaxed);
            compute_join_handle.thread().unpark();
        }

        // Check for incoming screenshots from the server
        let mut local_screenshot_data = main_context.screenshot_data.lock().unwrap();
        if local_screenshot_data.len() > 1 {
            // Switch to the selecting frame state
            dust_manip_state = DustManipState::SelectingFrame;
            selected_screenshot = 0;
            placed_particles.clear();

            // Clear old screenshot data, and any old frame pair textures
            screenshots.clear();
            selected_screenshot_texture = None;
            dust_search_frame_pairs.clear();

            // Copy over screenshot data into our local vector, clearing out the shared vector with the server
            screenshots.append(&mut local_screenshot_data);

            // Figure out positioning of the frame pair rectangles (in world space)
            let frame_pairs_x = (WORLD_WIDTH as i32 / 2) - (((screenshots.len() - 1) as u32 * dust_search_frame_pair_image.width()) as f32 / 2.0) as i32;
            let frame_pairs_y = (WORLD_HEIGHT as i32 / 2) - (dust_search_frame_pair_image.height() / 2) as i32;
            let mut frame_pair_rect = Rect::new(frame_pairs_x, frame_pairs_y, dust_search_frame_pair_image.width(), dust_search_frame_pair_image.height());

            // Create frame pairs
            let mut output_image: Vec<u8> = Vec::with_capacity(dust_search_frame_pair_image_data.len());
            for i in 1..screenshots.len() {
                let first_screenshot_data = &screenshots[i - 1];
                let second_screenshot_data = &screenshots[i];

                // Create image using two pixels/regions from each screenshot
                frame_images::make_four_pixel(&dust_search_frame_pair_image_data, &mut output_image, 
                                              first_screenshot_data, second_screenshot_data, 
                                              &context.search_config.four_pixel_config);
                let surface = Surface::from_data(&mut output_image, 
                                                              dust_search_frame_pair_image.width(), dust_search_frame_pair_image.height(), dust_search_frame_pair_image.width() * 4, 
                                                              PixelFormat::from(SDL_PIXELFORMAT_RGBA32.0 as i64)).unwrap();
                let mut texture = Texture::from_surface(&surface, &main_context.texture_creator).unwrap();
                texture.set_scale_mode(ScaleMode::Nearest);
                drop(surface);

                // Add to list
                dust_search_frame_pairs.push(DustFramePair {
                    texture,
                    rect: frame_pair_rect,
                    hovered: false,
                    first_frame_index: i - 1
                });

                // Move to the next frame's X coordinate, and clear output image contents
                frame_pair_rect.x += frame_pair_rect.w;
                output_image.clear();
            }
        }
        drop(local_screenshot_data);

        // Check whether connected
        let is_connected = main_context.server_connected.load(Ordering::Relaxed);
        if is_connected != last_server_connected {
            last_server_connected = is_connected;

            // Update newly-connected plugin with latest info
            if is_connected {
                _ = main_context.message_to_send_sender.send(MessageToSend::new_screenshot_mode(false));
                context.update_screenshot_delay_time(main_context);
            }
        }

        // Draw different contents depending on the current state
        match dust_manip_state {
            DustManipState::Waiting => {
                // Draw text for whether connected or not
                _ = program_common::draw_connected_text(main_context, &screen_space, is_connected);
            },
            DustManipState::SelectingFrame => {
                // Draw frame pair images
                for pair in dust_search_frame_pairs.iter() {
                    // Get correct screen space rectangle
                    let transformed_rect = screen_space.rect_world_to_screen(pair.rect);

                    // Draw the texture
                    _ = main_context.canvas.copy(&pair.texture, rect_from_texture(&pair.texture), transformed_rect);

                    // If currently hovered, draw a rectangle around the texture to indicate that
                    if pair.hovered {
                        main_context.canvas.set_draw_color(Color::RGBA(255, 255, 255, 64));
                        _ = main_context.canvas.draw_rect(rect_to_frect(transformed_rect));
                    }
                }
            },
            DustManipState::PlacingParticles => {
                // Draw inside of the world texture
                _ = main_context.canvas.with_texture_canvas(&mut world_texture, |texture_canvas| {
                    // Clear the world texture
                    texture_canvas.set_draw_color(Color::RGBA(0, 0, 0, 255));
                    texture_canvas.clear();

                    // Draw the current screenshot
                    if let Some(selected_screenshot_texture) = &selected_screenshot_texture {
                        let view_rect = context.search_config.view_rect;
                        _ = texture_canvas.copy(&selected_screenshot_texture, 
                            Rect::new(0, 0, selected_screenshot_texture.width(), selected_screenshot_texture.height()), 
                            Rect::new(view_rect.x, view_rect.y, view_rect.w as u32, view_rect.h as u32));
                    }

                    // Draw the debug animation, if enabled
                    if show_anim {
                        texture_canvas.set_draw_color(Color::RGBA(0, 0, 255, 128));
                        if let Some(debug_anim) = &debug_anim {
                            debug_anim.draw(texture_canvas, -num_attacks as f32, 0.0, false);
                        }
                    }

                    // Draw placed/placing particles
                    texture_canvas.set_draw_color(Color::RGBA(255, 0, 0, 255));
                    if let Some(placing_particle) = &placing_particle {
                        placing_particle.draw(texture_canvas);
                    }
                    texture_canvas.set_draw_color(Color::RGBA(128, 0, 0, 255));
                    for placed_particle in &placed_particles {
                        placed_particle.draw(texture_canvas);
                    }
                    texture_canvas.set_draw_color(Color::RGBA(255, 0, 0, 64));
                    if let Some(hovering_particle) = &hovering_particle {
                        hovering_particle.draw(texture_canvas);
                    }
                }).expect("Failed to draw to texture canvas");

                // Copy the world texture to the canvas
                _ = main_context.canvas.copy(&world_texture, context.search_config.view_rect, screen_space.irect());
            },
            DustManipState::FoundPosition => {
                // Show the manip instruction text
                if let Some(text_to_show) = &dust_manip_string {
                    _ = main_context.font.draw_text(
                        main_context,
                        &text_to_show[0..],
                        screen_space.center_x(), screen_space.center_y(),
                        0.5, 0.5,
                        200,
                        screen_space.scale() * 2.0,
                        Color::RGB(255, 255, 255)).unwrap();
                }
            }
        }

        // Handle raising this program's window after the given time (and warping the mouse to the center of it)
        if let Some(curr_raise_window_time) = raise_window_time {
            if sdl3::timer::ticks() >= curr_raise_window_time {
                raise_window_time = None;
                let window = main_context.canvas.window_mut();
                if window.raise() {
                    main_context.sdl_context.mouse().warp_mouse_in_window(window, window.size().0 as f32 / 2.0, window.size().1 as f32 / 2.0);
                }
            }
        }

        // Track the end of the buffer time, before which this program window is unfocusable
        if let Some(curr_raise_window_buffer_time) = focus_window_buffer_time {
            if sdl3::timer::ticks() >= curr_raise_window_buffer_time {
                focus_window_buffer_time = None;

                window_set_focusable(main_context.canvas.window_mut(), true);
            }
        }

        // Draw current attack counter
        _ = main_context.font.draw_text(
            main_context, 
            &format!("Attack counter: {}", num_attacks), 
            screen_space.x_world_to_screen(16.0), screen_space.y_world_to_screen(16.0),
            0.0, 0.0,
            0, 
            screen_space.scale(), 
            Color::RGB(128, 128, 128));

        // Draw current battlegroup
        main_context.font.set_alignment(TTF_HORIZONTAL_ALIGN_RIGHT);
        _ = main_context.font.draw_text(
            main_context, 
            &format!("Battlegroup: {}", curr_battlegroup.get_name()), 
            screen_space.x_world_to_screen(WORLD_WIDTH as f32 - 16.0), screen_space.y_world_to_screen(16.0),
            1.0, 0.0,
            120, 
            screen_space.scale(), 
            Color::RGB(128, 128, 128));
        main_context.font.set_alignment(TTF_HORIZONTAL_ALIGN_LEFT);

        // Draw leveled up text
        if leveled_up {
            _ = main_context.font.draw_text(
                main_context, 
                "Leveled up", 
                screen_space.center_x(), screen_space.y_world_to_screen(16.0),
                0.5, 0.0,
                0, 
                screen_space.scale(), 
                Color::RGB(128, 128, 128));
        }

        // Present latest canvas
        main_context.canvas.present();

        // Sleep until next frame
        frame_timer.end_and_sleep();
    }

    // Return to other programs
    SubProgram::ProgramSelector
}
