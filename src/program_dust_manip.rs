extern crate sdl3;

use compute_dust_search::{DustSearchMode, KillDustSearchParameters, DustSearchResult};
use compute_shaders::PointU32;
use defer_rs::defer;
use encounter_data::{Battlegroup, Encounterer};
use manip_data::MANIP_SETUPS_CORE;
use sdl3::mouse::MouseButton;
use sdl3::pixels::PixelFormat;
use sdl3::rect::{Point, Rect};
use sdl3::render::{Canvas, FRect, ScaleMode, Texture, TextureCreator};
use sdl3::surface::Surface;
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

use crate::compute_dust_search::{DustSearchParameters, SpareDustSearchParameters};
use crate::dust::{DustData, SPARE_RNG_LENGTH};
use crate::windowing::window_set_focusable;
use crate::program_common::{rect_from_texture, rect_to_frect, window_to_world, FrameTimer, ScreenSpace};
use crate::rng::{self, LinearRNG};
use crate::server::{self, MessageToSend};
use crate::{MainContext, SubProgram, compute_dust_search, compute_shaders, dust, encounter_data, frame_images, manip_data, program_common, windowing};

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
    pub fn draw_spare(&self, texture_canvas: &mut Canvas<Window>) {
        _ = texture_canvas.draw_rect(FRect {
            x: (self.x - 5) as f32,
            y: (self.y - 5) as f32,
            w: 10.0,
            h: 10.0
        });
    }
}

struct DustFramePreview<'a> {
    pub texture: Texture<'a>,
    pub texture_secondary: Option<Texture<'a>>,
    pub rect: Rect,
    pub rect_secondary: Option<Rect>,
    pub frame_index: usize,
    pub hovered: bool,
    pub hovered_secondary: bool,
    pub selected: bool,
    pub selected_secondary: bool
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
    pub fn new(main_context: &MainContext, search_config: DustSearchConfig) -> Self {
        let mut res = DustManipContext {
            search_mode: match &search_config.dust_data {
                DustData::KillDustData(data) => {
                    data.search_mode
                },
                DustData::SpareDustData(_) => {
                    DustSearchMode::Spare
                }
            },
            search_config
        };
        res.update_screenshot_delay_time(main_context);
        res
    }
    pub fn update_screenshot_delay_time(&mut self, main_context: &MainContext) {
        // Instantiate the dust animation so we can figure out how many frames there are (for timing purposes)
        let animation: DustAnimation = self.search_config.dust_data.create_animation();
        let mut time_in_frames;
        match animation {
            DustAnimation::KillDustAnimation(anim) => {
                let num_frames_early = match self.search_mode {
                    DustSearchMode::LastFrame | DustSearchMode::LastFrameEarly => 1,
                    DustSearchMode::SecondToLastFrame | DustSearchMode::SecondToLastFrameEarly => 2,
                    _ => panic!()
                };
                time_in_frames = anim.get_length() as i32 - num_frames_early as i32 - main_context.config.dust_screenshot_start_early_frames;
            },
            DustAnimation::SpareDustAnimation(_) => {
                time_in_frames = main_context.config.dust_screenshot_spare_time_frames;
            }
        }
        if time_in_frames < 0 {
            time_in_frames = 0;
        }

        // Send signal to client for the new delay time
        let new_delay_time = (time_in_frames as f32 * (1000.0 / 30.0)) as u32;
        _ = main_context.message_to_send_sender.send(MessageToSend::new_screenshot_start_delay(new_delay_time));
    }
}

const EXTRA_RAISE_DELAY_MS: u32 = 450;

fn get_new_screenshot<'a>(texture_creator: &'a TextureCreator<WindowContext>, screenshot_data: &mut ScreenshotData, world_view: Rect, search_mode: DustSearchMode, brighten: bool) -> Texture<'a> {
    // Preprocess image
    let mut cleaned_data: Vec<u8> = Vec::with_capacity((world_view.w * world_view.h * 4) as usize);
    if search_mode != DustSearchMode::Spare {
        frame_images::clear_unwanted_pixels_dust(&mut cleaned_data, &screenshot_data, world_view, match search_mode {
            DustSearchMode::LastFrame | DustSearchMode::SecondToLastFrame => false,
            DustSearchMode::LastFrameEarly | DustSearchMode::SecondToLastFrameEarly => true,
            _ => panic!()
        }, brighten);
    } else {
        frame_images::copy_pixels(&mut cleaned_data, &screenshot_data, world_view);
    }
    
    // Create texture
    let surface = Surface::from_data(&mut cleaned_data, 
        world_view.w as u32, world_view.h as u32, world_view.w as u32 * 4, PixelFormat::RGBA32).unwrap();
    let mut texture = Texture::from_surface(&surface, &texture_creator).unwrap();
    texture.set_scale_mode(ScaleMode::Nearest);
    texture
}

fn set_new_search_config(main_context: &MainContext, context: &mut DustManipContext, new_config: DustSearchConfig) {
    // Set the new config
    context.search_config = new_config;

    // Update the current mode
    context.search_mode = match &context.search_config.dust_data {
        dust::DustData::KillDustData(data) => {
            data.search_mode
        },
        dust::DustData::SpareDustData(_) => {
            DustSearchMode::Spare
        }
    };

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
    let brighten = true;
    let mut context = DustManipContext::new(&main_context, search_config);
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
    let compute_parameters = Arc::new(Mutex::new(DustSearchParameters::None));
    let compute_parameters_thread = compute_parameters.clone();
    let compute_result = Arc::new(Mutex::new(DustSearchResult { match_count: 0, single_matched_position: 0, spare_frame_index: 0 }));
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
    //let mut dust_search_frame_pairs: Vec<DustFramePreview> = Vec::with_capacity(100);
    //let dust_search_frame_pair_image = 
    //    ImageReader::open(util::get_exe_directory().join("./assets/dust_search_frame_pair.png"))
    //        .expect("Failed to open image").decode().expect("Failed to decode image");
    //let dust_search_frame_pair_image_data = dust_search_frame_pair_image.to_rgba8().to_vec();

    // Initialize frame previews
    let mut dust_search_frame_previews: Vec<DustFramePreview> = Vec::with_capacity(100);

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
                    return SubProgram::None
                },
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                Event::KeyDown { keycode: Some(Keycode::Space), .. } => {
                    let mut new_debug_anim = context.search_config.dust_data.create_animation();
                    let test_rng_position = 0;
                    
                    let mut total_num_updates: usize;
                    match &mut new_debug_anim {
                        DustAnimation::KillDustAnimation(anim) => {
                            let frame_end_offset: usize = match context.search_mode {
                                DustSearchMode::LastFrame | DustSearchMode::LastFrameEarly => 1,
                                DustSearchMode::SecondToLastFrame | DustSearchMode::SecondToLastFrameEarly => 2,
                                _ => panic!()
                            };
                            let early_delay_offset: usize = match context.search_mode {
                                DustSearchMode::LastFrame | DustSearchMode::SecondToLastFrame => 0,
                                DustSearchMode::LastFrameEarly | DustSearchMode::SecondToLastFrameEarly => 1,
                                _ => panic!()
                            };

                            anim.set_start_process_frame(match context.search_mode {
                                DustSearchMode::LastFrame | DustSearchMode::LastFrameEarly => anim.get_frame_count() - 1,
                                DustSearchMode::SecondToLastFrame | DustSearchMode::SecondToLastFrameEarly => anim.get_frame_count() - 2,
                                _ => panic!()
                            });
                            
                            total_num_updates = (anim.get_length() - frame_end_offset) - early_delay_offset;
                        },
                        DustAnimation::SpareDustAnimation(_) => {
                            total_num_updates = 0;
                        }
                    }

                    new_debug_anim.start_animating(&prng, test_rng_position);
                    let mut num_updates = 0;
                    while num_updates < total_num_updates {
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
                                context.search_config.view_rect, context.search_mode, brighten));
                        }
                    }
                },
                Event::KeyDown { keycode: Some(Keycode::Right), .. } => {
                    if dust_manip_state == DustManipState::PlacingParticles {
                        if screenshots.len() > 0 {
                            selected_screenshot = (selected_screenshot + 1) % screenshots.len();
                            drop(selected_screenshot_texture);
                            selected_screenshot_texture = Some(get_new_screenshot(&main_context.texture_creator, &mut screenshots[selected_screenshot],
                                context.search_config.view_rect, context.search_mode, brighten));
                        }
                    }
                },
                Event::MouseButtonDown { mouse_btn: MouseButton::Left, x, y, .. } => {
                    match dust_manip_state {
                        DustManipState::SelectingFrame => {
                            let (selector_x, selector_y) = window_to_world(x, y, Rect::new(0, 0, WORLD_WIDTH, WORLD_HEIGHT), screen_space.rect());
                            let p = Point::new(selector_x, selector_y);
                            if context.search_mode == DustSearchMode::Spare {
                                for preview in dust_search_frame_previews.iter_mut() {
                                    if preview.rect.contains_point(p) {
                                        preview.selected = preview.selected_secondary || !preview.selected;
                                    }
                                    
                                    if let Some(rect_secondary) = &preview.rect_secondary {
                                        if rect_secondary.contains_point(p) {
                                            preview.selected_secondary = !preview.selected_secondary;
                                            preview.selected = preview.selected_secondary;
                                        }
                                    }
                                }
                            } else {
                                for preview in dust_search_frame_previews.iter_mut() {
                                    if preview.rect.contains_point(p) {
                                        selected_screenshot = preview.frame_index;
                                        drop(selected_screenshot_texture);
                                        context.search_mode = context.search_mode.to_normal();
                                        selected_screenshot_texture = Some(get_new_screenshot(&main_context.texture_creator, &mut screenshots[selected_screenshot], context.search_config.view_rect, context.search_mode, brighten));
                                        dust_manip_state = DustManipState::PlacingParticles;
                                        break;
                                    }
                                }
                            }
                        },
                        DustManipState::PlacingParticles => {
                            let (world_x, world_y) = window_to_world(x, y, context.search_config.view_rect, screen_space.rect());
                            let offset = if context.search_mode == DustSearchMode::Spare { 0 } else { 1 };
                            placing_particle = Some(PlacedDustParticle {
                                x: world_x - offset,
                                y: world_y - offset,
                                xscale: 1
                            });
                        }
                        _ => {}
                    }
                },
                Event::MouseButtonUp { mouse_btn: MouseButton::Left, x, y, .. } => {
                    if dust_manip_state == DustManipState::SelectingFrame {
                        if context.search_mode == DustSearchMode::Spare {
                            let mut frame_index = usize::MAX;
                            for preview in &dust_search_frame_previews {
                                if preview.selected_secondary {
                                    frame_index = preview.frame_index;
                                }
                            }
                            if frame_index != usize::MAX {
                                selected_screenshot = frame_index;
                                drop(selected_screenshot_texture);
                                selected_screenshot_texture = Some(get_new_screenshot(&main_context.texture_creator, &mut screenshots[selected_screenshot], context.search_config.view_rect, context.search_mode, brighten));
                                dust_manip_state = DustManipState::PlacingParticles;
                            }
                        }
                    } else {
                        if let Some(ref mut placing_particle) = placing_particle {
                            // TODO: maybe allow for changing xscale somehow...
                            let (world_x, world_y) = window_to_world(x, y, context.search_config.view_rect, screen_space.rect());
                            let offset = if context.search_mode == DustSearchMode::Spare { 0 } else { 1 };
                            (placing_particle.x, placing_particle.y) = (world_x - offset, world_y - offset);
                            placed_particles.push(placing_particle.clone());

                            if placed_particles.len() >= num_to_click {
                                // Start searching now!
                                queued_search = true;
                            }
                        }
                        placing_particle = None;
                    }
                },
                Event::MouseMotion { mousestate, x, y, .. } => {
                    match dust_manip_state {
                        DustManipState::SelectingFrame => {
                            let (selector_x, selector_y) = window_to_world(x, y, Rect::new(0, 0, WORLD_WIDTH, WORLD_HEIGHT), screen_space.rect());
                            let p = Point::new(selector_x, selector_y);
                            for preview in dust_search_frame_previews.iter_mut() {
                                let was_hovered = preview.hovered;
                                preview.hovered = preview.rect.contains_point(p);

                                if let Some(rect_secondary) = &preview.rect_secondary {
                                    let was_hovered_secondary = preview.hovered_secondary;
                                    preview.hovered_secondary = rect_secondary.contains_point(p);

                                    if mousestate.left() {
                                        if !was_hovered && preview.hovered {
                                            preview.selected = preview.selected_secondary || !preview.selected;
                                        }
                                        if !was_hovered_secondary && preview.hovered_secondary {
                                            preview.selected_secondary = !preview.selected_secondary;
                                            preview.selected = preview.selected_secondary;
                                        }
                                    }
                                }
                            }
                        },
                        DustManipState::PlacingParticles => {
                            let (world_x, world_y) = window_to_world(x, y, context.search_config.view_rect, screen_space.rect());
                            let offset = if context.search_mode == DustSearchMode::Spare { 0 } else { 1 };
                            hovering_particle = Some(PlacedDustParticle { 
                                x: world_x - offset, 
                                y: world_y - offset, 
                                xscale: 1 
                            });
                            if !mousestate.left() {
                                continue;
                            }
                            if let Some(ref mut placing_particle) = placing_particle {
                                // TODO: maybe allow for changing xscale somehow...
                                (placing_particle.x, placing_particle.y) = (world_x - offset, world_y - offset);
                            }
                        },
                        _ => {}
                    }
                },
                Event::MouseButtonDown { mouse_btn: MouseButton::Right, x, y, .. } => {
                    match dust_manip_state {
                        DustManipState::SelectingFrame => {
                            if context.search_mode == DustSearchMode::Spare {
                                for preview in dust_search_frame_previews.iter_mut() {
                                    preview.selected = false;
                                    preview.selected_secondary = false;
                                }
                            } else {
                                let (selector_x, selector_y) = window_to_world(x, y, Rect::new(0, 0, WORLD_WIDTH, WORLD_HEIGHT), screen_space.rect());
                                for preview in dust_search_frame_previews.iter_mut() {
                                    if preview.rect.contains_point(Point::new(selector_x, selector_y)) {
                                        selected_screenshot = preview.frame_index;
                                        drop(selected_screenshot_texture);
                                        context.search_mode = context.search_mode.to_early();
                                        selected_screenshot_texture = Some(get_new_screenshot(&main_context.texture_creator, &mut screenshots[selected_screenshot], context.search_config.view_rect, context.search_mode, brighten));
                                        dust_manip_state = DustManipState::PlacingParticles;
                                        break;
                                    }
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
                        selected_screenshot_texture = Some(get_new_screenshot(&main_context.texture_creator, &mut screenshots[selected_screenshot], context.search_config.view_rect, context.search_mode, brighten));
                    } else if dust_manip_state == DustManipState::PlacingParticles {
                        dust_manip_state = DustManipState::SelectingFrame;
                        placed_particles.clear();
                        placing_particle = None;
                        selected_screenshot_texture = None;
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
                    let predicted_pos = match search_anim {
                        DustAnimation::KillDustAnimation(anim) => {
                            let predicted_pos = search_result.single_matched_position as usize + anim.get_after_battle_rng_calls(match leveled_up {
                                true => context.search_config.text_length_lvup,
                                false => context.search_config.text_length
                            });
                            if battlegroup_order_pos < battlegroup_order.len() - 1 {
                                battlegroup_order_pos += 1;
                            }
                            println!("Matched position is {}", search_result.single_matched_position);

                            predicted_pos
                        },
                        DustAnimation::SpareDustAnimation(_) => {
                            println!("Matched position is {}, frame index {}", search_result.single_matched_position, search_result.spare_frame_index);

                            // Try to predict position...
                            let mut predicted_pos = 
                                search_result.single_matched_position +
                                SPARE_RNG_LENGTH as u32;

                            // Calculate unskipped and skipped frame counts
                            let mut unskipped_frame_count = 0;
                            let mut skipped_frame_count = 0;
                            for preview in &dust_search_frame_previews {
                                if preview.selected_secondary {
                                    skipped_frame_count += 1;
                                } else if preview.selected {
                                    unskipped_frame_count += 1;
                                }
                            }

                            // If frame counts don't add up, we have a problem...
                            let total_unchecked_frame_count = unskipped_frame_count + skipped_frame_count;
                            let total_expected_frame_count = search_result.spare_frame_index + 1;
                            if total_unchecked_frame_count != total_expected_frame_count {
                                println!("Mismatch of frame counts! Difference = {}", total_unchecked_frame_count as i32 - total_expected_frame_count as i32);
                                if total_unchecked_frame_count > total_expected_frame_count {
                                    // Assume extra frames at the end, if possible...
                                    // If not possible, assume extra frames at the beginning...
                                    let adjustment = total_unchecked_frame_count - total_expected_frame_count;
                                    if skipped_frame_count > adjustment {
                                        println!("Assuming {} extra skipped frames, removing", adjustment);
                                        skipped_frame_count -= adjustment;
                                    } else if unskipped_frame_count > adjustment {
                                        println!("Assuming {} extra unskipped frames, removing", adjustment);
                                        unskipped_frame_count -= adjustment;
                                    } else {
                                        println!("Assuming some combination of skipped and unskipped... Removing skipped first");
                                        let mut remaining_adjustment = adjustment;
                                        while skipped_frame_count > 1 && remaining_adjustment > 0 {
                                            skipped_frame_count -= 1;
                                            remaining_adjustment -= 1;
                                        }
                                        while unskipped_frame_count > 1 && remaining_adjustment > 0 {
                                            unskipped_frame_count -= 1;
                                            remaining_adjustment -= 1;
                                        }
                                    }
                                } else {
                                    // Assume missing frames at the beginning
                                    let adjustment = total_expected_frame_count - total_unchecked_frame_count;
                                    println!("Assuming {} missing unskipped frames, adding", adjustment);
                                    unskipped_frame_count += adjustment;
                                }
                            }

                            // Add unskipped text RNG
                            for i in 0..unskipped_frame_count {
                                predicted_pos += i * 2;
                            }
                            
                            // Add skipped text RNG
                            predicted_pos += skipped_frame_count * 2 * match leveled_up {
                                true => (context.search_config.text_length_lvup - 2) as u32,
                                false => (context.search_config.text_length - 2) as u32
                            };

                            predicted_pos as usize
                        }
                    };
                    
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
                    println!("[{}] Current destined encounter: {}", curr_encounterer.get_name(), curr_encounterer.get_battlegroup_at_pos(&prng, predicted_pos + 1).get_name());
                    println!("[{}] Room change destined encounter: {}", curr_encounterer.get_name(), curr_encounterer.get_battlegroup_at_pos(&prng, predicted_pos + 2).get_name());
                    let kills_before_battle = 0;
                    let new_kill_count = kills_before_battle + context.search_config.kill_count;
                    let step_count = curr_encounterer.get_step_count_room_start(&prng, predicted_pos, new_kill_count);
                    println!("[{}] Room change step count: {}", curr_encounterer.get_name(), step_count);
                    println!("[{}] Room change step count (in seconds): {}", curr_encounterer.get_name(), step_count / 30.0);

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
                        selected_screenshot_texture = Some(get_new_screenshot(&main_context.texture_creator, &mut screenshots[selected_screenshot], context.search_config.view_rect, context.search_mode, brighten));
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
                4 => {
                    // Reset run
                    main_context.run_context.reset();
                    return main_context.config.reset_return_to;
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

            let slide_amount: i16 = if context.search_config.slides_left { num_attacks as i16 } else { 0 };

            let mut new_search_anim = context.search_config.dust_data.create_animation();
            match &mut new_search_anim {
                DustAnimation::KillDustAnimation(anim) => {
                    anim.compute_frame_rng_offsets();
                    for particle in anim.get_frames().last().unwrap().iter() {
                        initial_particles.push(PointU32::new(particle.get_x() as i16 - slide_amount, particle.get_y() as i16));
                        initial_particles_last_frame_count += 1;
                    }
                    match context.search_mode {
                        DustSearchMode::SecondToLastFrame | DustSearchMode::SecondToLastFrameEarly => {
                            for particle in anim.get_frames().get(anim.get_frame_count() - 2).unwrap().iter() {
                                initial_particles.push(PointU32::new(particle.get_x() as i16 - slide_amount, particle.get_y() as i16));
                                initial_particles_second_last_frame_count += 1;
                            }
                        }
                        DustSearchMode::LastFrame | DustSearchMode::LastFrameEarly => {},
                        _ => panic!()
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
                        DustSearchMode::SecondToLastFrame | DustSearchMode::SecondToLastFrameEarly => 2,
                        _ => panic!()
                    };
                    let initial_rng_skip_amount: u32 = 2 * (anim.get_frame_count() - frame_end_offset) as u32;

                    if frame_end_offset == 2 {
                        //println!("RNG skip amount between last two frames = {}", initial_rng_skip_amount);
                        //println!("expected second to last RNG frame offset = {}", new_search_anim.get_frame_rng_offset(new_search_anim.get_frame_count() - 2));
                        //println!("actual second to last RNG frame offset = {}", new_search_anim.get_frame_rng_offset(new_search_anim.get_frame_count() - 1) as u32 - ((initial_particles_second_last_frame_count * 2) + initial_rng_skip_amount));
                        assert!(anim.get_frame_rng_offset(anim.get_frame_count() - 1) as u32 - ((initial_particles_second_last_frame_count * 2) + initial_rng_skip_amount) ==
                                anim.get_frame_rng_offset(anim.get_frame_count() - 2) as u32);
                    }

                    *compute_parameters.lock().unwrap() = DustSearchParameters::KillDustSearchParameters(KillDustSearchParameters {
                        search_range: (num_to_compute - anim.get_total_rng_calls() - 10_000) as u32,
                        last_frame_rng_offset: anim.get_frame_rng_offset(anim.get_frame_count() - 1) as u32,
                        matching_particles,
                        initial_particles: initial_particles.clone(),
                        last_frame_particle_count: initial_particles_last_frame_count,
                        second_last_frame_particle_count: initial_particles_second_last_frame_count,
                        initial_rng_skip_amount,
                        search_mode: context.search_mode
                    });
                },
                DustAnimation::SpareDustAnimation(anim) => {
                    // if let Some(DustAnimation::SpareDustAnimation(debug_anim)) = &debug_anim {
                    //     for particle in &debug_anim.particles {
                    //         println!("Actual particle at ({}, {})", particle.x, particle.y);
                    //     }
                    // }

                    let matching_particles: Vec<PointU32> = placed_particles.iter().map(|p| PointU32::new(p.x as i16, p.y as i16)).collect();
                    *compute_parameters.lock().unwrap() = DustSearchParameters::SpareDustSearchParameters(SpareDustSearchParameters {
                        search_range: num_to_compute as u32,
                        anim_position: PointU32::new(anim.dust_data.x as i16, anim.dust_data.y as i16),
                        anim_size: PointU32::new(anim.dust_data.sprite_width as i16, anim.dust_data.sprite_height as i16),
                        matching_particles
                    });
                }
            }

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
            dust_search_frame_previews.clear();

            // Copy over screenshot data into our local vector, clearing out the shared vector with the server
            screenshots.append(&mut local_screenshot_data);

            // Display previews differently based on whether sparing or killing
            match context.search_mode {
                DustSearchMode::Spare => {
                    // Figure out positioning of the frame preview rectangles (in world space)
                    let preview_rect = context.search_config.view_rect;
                    let preview_secondary_rect = Rect::new(41, 304, 39, 29);
                    let frame_preview_scale = main_context.config.dust_screenshot_preview_scale;
                    let frame_preview_width = ((preview_rect.w as f32) * frame_preview_scale) as u32;
                    let frame_preview_single_height = ((preview_rect.h as f32) * frame_preview_scale) as u32;
                    let frame_preview_total_height = frame_preview_single_height * 2;
                    let frame_preview_x = (WORLD_WIDTH as i32 / 2) - (((screenshots.len() - 1) as u32 * frame_preview_width) as f32 / 2.0) as i32;
                    let frame_preview_y = (WORLD_HEIGHT as i32 / 2) - (frame_preview_total_height / 2) as i32;
                    let frame_preview_secondary_y = frame_preview_y + frame_preview_single_height as i32;
                    let mut frame_preview_rect = Rect::new(frame_preview_x, frame_preview_y, frame_preview_width, frame_preview_single_height);
                    let mut frame_preview_secondary_rect = Rect::new(frame_preview_x, frame_preview_secondary_y, frame_preview_width, frame_preview_single_height);

                    // Create frame previews
                    let mut output_image: Vec<u8> = Vec::with_capacity(preview_rect.w as usize * preview_rect.h as usize * 4);
                    let mut output_secondary_image: Vec<u8> = Vec::with_capacity(preview_rect.w as usize * preview_rect.h as usize * 4);
                    let mut i = 0;
                    for screenshot_data in &screenshots {
                        // Create preview images from screenshot
                        frame_images::copy_pixels(&mut output_image, screenshot_data, preview_rect);
                        frame_images::copy_pixels(&mut output_secondary_image, screenshot_data, preview_secondary_rect);
                        let surface = Surface::from_data(&mut output_image, 
                            preview_rect.w as u32, preview_rect.h as u32, preview_rect.w as u32 * 4, PixelFormat::RGBA32).unwrap();
                        let mut texture = Texture::from_surface(&surface, &main_context.texture_creator).unwrap();
                        texture.set_scale_mode(ScaleMode::Nearest);
                        drop(surface);
                        let surface_secondary = Surface::from_data(&mut output_secondary_image, 
                            preview_secondary_rect.w as u32, preview_secondary_rect.h as u32, preview_secondary_rect.w as u32 * 4, PixelFormat::RGBA32).unwrap();
                        let mut texture_secondary = Texture::from_surface(&surface_secondary, &main_context.texture_creator).unwrap();
                        texture_secondary.set_scale_mode(ScaleMode::Nearest);
                        drop(surface_secondary);

                        // Add to list
                        dust_search_frame_previews.push(DustFramePreview {
                            texture,
                            texture_secondary: Some(texture_secondary),
                            rect: frame_preview_rect,
                            rect_secondary: Some(frame_preview_secondary_rect),
                            frame_index: i,
                            hovered: false,
                            hovered_secondary: false,
                            selected: false,
                            selected_secondary: false
                        });

                        // Move to the next frame's X coordinate, and clear output image contents
                        frame_preview_rect.x += frame_preview_rect.w;
                        frame_preview_secondary_rect.x += frame_preview_secondary_rect.w;
                        output_image.clear();
                        output_secondary_image.clear();

                        i += 1;
                    }
                },
                _ => {
                    // Figure out positioning of the frame preview rectangles (in world space)
                    let preview_rect = context.search_config.view_rect;
                    let frame_preview_scale = main_context.config.dust_screenshot_preview_scale;
                    let frame_preview_width = ((preview_rect.w as f32) * frame_preview_scale) as u32;
                    let frame_preview_height = ((preview_rect.h as f32) * frame_preview_scale) as u32;
                    let frame_preview_x = (WORLD_WIDTH as i32 / 2) - (((screenshots.len() - 1) as u32 * frame_preview_width) as f32 / 2.0) as i32;
                    let frame_preview_y = (WORLD_HEIGHT as i32 / 2) - (frame_preview_height / 2) as i32;
                    let mut frame_preview_rect = Rect::new(frame_preview_x, frame_preview_y, frame_preview_width, frame_preview_height);

                    // Create frame previews
                    let mut output_image: Vec<u8> = Vec::with_capacity(preview_rect.w as usize * preview_rect.h as usize * 4);
                    let mut i = 0;
                    for screenshot_data in &screenshots {
                        // Create preview image from screenshot
                        frame_images::copy_pixels(&mut output_image, screenshot_data, preview_rect);
                        let surface = Surface::from_data(&mut output_image, 
                            preview_rect.w as u32, preview_rect.h as u32, preview_rect.w as u32 * 4, PixelFormat::RGBA32).unwrap();
                        let mut texture = Texture::from_surface(&surface, &main_context.texture_creator).unwrap();
                        texture.set_scale_mode(ScaleMode::Nearest);
                        drop(surface);

                        // Add to list
                        dust_search_frame_previews.push(DustFramePreview {
                            texture,
                            texture_secondary: None,
                            rect: frame_preview_rect,
                            rect_secondary: None,
                            frame_index: i,
                            hovered: false,
                            hovered_secondary: false,
                            selected: false,
                            selected_secondary: false
                        });

                        // Move to the next frame's X coordinate, and clear output image contents
                        frame_preview_rect.x += frame_preview_rect.w;
                        output_image.clear();

                        i += 1;
                    }
                }
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
                // Draw frame preview images
                for preview in dust_search_frame_previews.iter() {
                    if let Some(texture_secondary) = &preview.texture_secondary {
                        // Get correct screen space rectangle
                        let transformed_rect = screen_space.rect_world_to_screen(preview.rect);
                        let transformed_secondary_rect = screen_space.rect_world_to_screen(preview.rect_secondary.unwrap());

                        // Draw the texture
                        _ = main_context.canvas.copy(&preview.texture, rect_from_texture(&preview.texture), transformed_rect);
                        _ = main_context.canvas.copy(texture_secondary, rect_from_texture(texture_secondary), transformed_secondary_rect);

                        // If currently hovered, draw a rectangle around the texture to indicate that
                        if preview.hovered {
                            main_context.canvas.set_draw_color(Color::RGBA(255, 255, 255, 64));
                            _ = main_context.canvas.draw_rect(rect_to_frect(transformed_rect));
                        }
                        if preview.selected {
                            main_context.canvas.set_draw_color(Color::RGBA(255, 0, 0, 64));
                            _ = main_context.canvas.fill_rect(rect_to_frect(transformed_rect));
                        }
                        if preview.hovered_secondary {
                            main_context.canvas.set_draw_color(Color::RGBA(255, 255, 255, 64));
                            _ = main_context.canvas.draw_rect(rect_to_frect(transformed_secondary_rect));
                        }
                        if preview.selected_secondary {
                            main_context.canvas.set_draw_color(Color::RGBA(255, 0, 0, 64));
                            _ = main_context.canvas.fill_rect(rect_to_frect(transformed_secondary_rect));
                        }
                    } else {
                        // Get correct screen space rectangle
                        let transformed_rect = screen_space.rect_world_to_screen(preview.rect);

                        // Draw the texture
                        _ = main_context.canvas.copy(&preview.texture, rect_from_texture(&preview.texture), transformed_rect);

                        // If currently hovered, draw a rectangle around the texture to indicate that
                        if preview.hovered {
                            main_context.canvas.set_draw_color(Color::RGBA(255, 255, 255, 64));
                            _ = main_context.canvas.draw_rect(rect_to_frect(transformed_rect));
                        }
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
                        match context.search_mode {
                            DustSearchMode::Spare => {
                                placing_particle.draw_spare(texture_canvas);
                            },
                            _ => {
                                placing_particle.draw(texture_canvas);
                            }
                        }
                    }
                    texture_canvas.set_draw_color(Color::RGBA(128, 0, 0, 255));
                    for placed_particle in &placed_particles {
                        match context.search_mode {
                            DustSearchMode::Spare => {
                                placed_particle.draw_spare(texture_canvas);
                            },
                            _ => {
                                placed_particle.draw(texture_canvas);
                            }
                        }
                    }
                    texture_canvas.set_draw_color(Color::RGBA(255, 0, 0, 64));
                    if let Some(hovering_particle) = &hovering_particle {
                        match context.search_mode {
                            DustSearchMode::Spare => {
                                hovering_particle.draw_spare(texture_canvas);
                            },
                            _ => {
                                hovering_particle.draw(texture_canvas);
                            }
                        }
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
                if window.raise() && main_context.config.mouse_warps {
                    main_context.sdl_context.mouse().warp_mouse_in_window(window, window.size().0 as f32 / 2.0, window.size().1 as f32 / 2.0);
                }
            }
        }

        // Track the end of the buffer time, before which this program window is unfocusable
        if let Some(curr_raise_window_buffer_time) = focus_window_buffer_time {
            if sdl3::timer::ticks() >= curr_raise_window_buffer_time {
                focus_window_buffer_time = None;

                if !main_context.config.window_unfocusable_by_default {
                    window_set_focusable(main_context.canvas.window_mut(), true);
                }
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
    if main_context.config.auto_return_to_naming_search { SubProgram::NamingSeedSearch } else { SubProgram::ProgramSelector }
}
