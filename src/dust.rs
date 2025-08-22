use std::usize;

use sdl3::{pixels::Color, rect::Rect, render::{Canvas, FRect}, video::Window};

use crate::{compute_dust_search::DustSearchMode, frame_images::FourPixelConfig, rng::PrecomputedRNG};

pub struct DustData {
    pub data: &'static str,
    pub wide: bool,
    pub search_mode: DustSearchMode,
    pub x: f32,
    pub y: f32,
}
impl DustData {
    pub const fn new(data: &'static str, wide: bool, search_mode: DustSearchMode) -> Self {
        DustData {
            data,
            wide,
            search_mode,
            x: 0.0,
            y: 0.0
        }
    }
    pub fn to_search_config(&self, x: f32, y: f32, 
                            text_length: usize, text_length_lvup: usize, view_rect: Rect, four_pixel_config: FourPixelConfig) -> DustSearchConfig {
        DustSearchConfig {
            dust_data: DustData {
                data: self.data,
                wide: self.wide,
                search_mode: self.search_mode,
                x,
                y
            },
            text_length,
            text_length_lvup,
            view_rect,
            four_pixel_config
        }
    }
}

pub struct DustSearchConfig {
    pub dust_data: DustData,
    pub text_length: usize,
    pub text_length_lvup: usize,
    pub view_rect: Rect,
    pub four_pixel_config: FourPixelConfig
}

#[derive(Clone)]
pub struct DustAnimation {
    particle_frames: Vec<Vec<DustParticle>>,
    particle_frame_rng_offsets: Vec<usize>,
    frame_index: u32,
    start_process_index: usize
}

#[derive(Clone)]
pub struct DustParticle {
    x: f32,
    y: f32,
    hspeed: f32,
    vspeed: f32,
    gravity: f32,
    xscale: f32,
}
impl DustParticle {
    pub fn get_x(&self) -> f32 {
        self.x
    }
    pub fn get_y(&self) -> f32 {
        self.y
    }
}

const DUST_END_LINE: u8 = '}' as u8;
const DUST_END: u8 = '~' as u8;

impl DustData {
    pub fn create_animation(&self) -> DustAnimation {
        // Create animation struct
        let mut anim: DustAnimation = DustAnimation {
            particle_frames: Vec::with_capacity(32),
            particle_frame_rng_offsets: Vec::with_capacity(32),
            frame_index: 0,
            start_process_index: 0
        };

        // Create enumerator over data
        let mut data_enum = self.data.bytes().enumerate();

        // Read data, constructing spawn frames and particles within those frames.
        // Start at the top.
        let mut curr_y = 0;
        'read_loop: loop {
            // Each loop is a single frame
            let mut frame: Vec<DustParticle> = Vec::with_capacity(4);

            // Process 4 rows/lines per each frame
            for _ in 0..4 {
                // Process a single row - start from the left
                let mut curr_x: u32 = 0;

                // Read characters until the end of line, or end of overall data
                let mut curr_char: u8 = 0;
                while curr_char != DUST_END_LINE && curr_char != DUST_END {
                    // Read next character from enumerator
                    curr_char = data_enum.next().unwrap().1;

                    // Process character
                    if curr_char >= 86 && curr_char <= 121 {
                        // Empty/skip character
                        curr_x += ((curr_char - 85) as u32) * 2;
                    } else if curr_char >= 39 && curr_char <= 82 {
                        // Particle character
                        if self.wide {
                            // Wide mode - collapses multiple particles into one wider particle
                            frame.push(DustParticle {
                                x: self.x + (curr_x as f32),
                                y: self.y + (curr_y as f32),
                                xscale: ((curr_char as f32) - 40.0) * 2.0,
                                hspeed: 0.0,
                                vspeed: 0.0,
                                gravity: 0.0,
                            });
                            curr_x += ((curr_char - 40) as u32) * 2;
                        } else {
                            // Non-wide mode - every particle is the same size
                            for _ in 0..(curr_char - 40) {
                                frame.push(DustParticle {
                                    x: self.x + (curr_x as f32),
                                    y: self.y + (curr_y as f32) + 2.0, // Strange offset...
                                    xscale: 2.0,
                                    hspeed: 0.0,
                                    vspeed: 0.0,
                                    gravity: 0.0,
                                });
                                curr_x += 2;
                            }
                        }
                    }
                }
                
                // If the end of the data has been reached, stop here
                if curr_char == DUST_END {
                    // Push final frame to the animation
                    if frame.len() > 0 {
                        anim.particle_frames.push(frame);
                    }
                    break 'read_loop;
                }

                // Advance down to the next row
                curr_y += 2;
            }

            // Push this current frame to the animation
            if frame.len() > 0 {
                anim.particle_frames.push(frame);
            }
        }

        // Return final animation
        anim
    }
}

const DUST_COLORS: [u8; 12] = [255, 251, 251, 251, 251, 251, 251, 219, 182, 146, 109, 73];
const ROUNDING_OFFSET: f32 = 1.0 / 512.0; // Rounding offset as used by Direct3D

impl DustAnimation {
    pub fn draw(&self, canvas: &mut Canvas<Window>, x: f32, y: f32, change_color: bool) {
        let mut iter_frame_index: u32 = self.start_process_index as u32;
        for frame in self.particle_frames.iter().skip(self.start_process_index) {
            // Skip frames that are done animating
            let frame_image_index: i32 = i32::max((self.frame_index as i32) - (iter_frame_index as i32), 0);
            if frame_image_index >= 12 {
                iter_frame_index += 1;
                continue;
            }

            // Fade out depending on time and frame
            if change_color {
                let frame_color = DUST_COLORS[frame_image_index as usize];
                canvas.set_draw_color(Color::RGB(frame_color, frame_color, frame_color));
            }

            // Draw rectangles
            for particle in frame.iter() {
                _ = canvas.draw_rect(FRect::new(f32::round(particle.x + x - ROUNDING_OFFSET), f32::round(particle.y + y - ROUNDING_OFFSET), particle.xscale, 2.0));
            }

            iter_frame_index += 1;
        }
    }

    pub fn compute_frame_rng_offsets(&mut self) {
        let mut rng_skip_amount: usize = 0;
        let mut rng_position = 0;
        for frame in self.particle_frames.iter() {
            // Track frame RNG offsets
            self.particle_frame_rng_offsets.push(rng_position);

            // Count RNG for particles
            rng_position += frame.len() * 2;
            
            // Count RNG from "you won" text
            rng_position += rng_skip_amount;
            rng_skip_amount += 2;
        }
    }

    pub fn start_animating(&mut self, rng: &PrecomputedRNG, mut rng_position: usize) {
        let mut rng_skip_amount: usize = 0;
        let rng_start_position = rng_position;

        //let mut frame_index = 0;
        //let num_frames = self.particle_frames.len();

        for frame in self.particle_frames.iter_mut() {
            // Track frame RNG offsets
            self.particle_frame_rng_offsets.push(rng_position - rng_start_position);

            // Get RNG for particles
            //let mut particle_index = 0;
            for particle in frame.iter_mut() {
                particle.gravity = (rng.get_f64(0.5, rng_position) as f32) + 0.2;
                particle.hspeed = (rng.get_f64(4.0, rng_position + 1) as f32) - 2.0;
                rng_position += 2;

                //particle_index += 1;
            }            
            
            // Skip RNG from "you won" text
            rng_position += rng_skip_amount;
            rng_skip_amount += 2;

            //frame_index += 1;
        }
    }

    pub fn update(&mut self) {
        let curr_frame_index = self.frame_index;

        let mut iter_frame_index: u32 = self.start_process_index as u32;
        for frame in self.particle_frames.iter_mut().skip(self.start_process_index) {
            // Skip frames that haven't started animating yet
            if iter_frame_index > curr_frame_index {
                iter_frame_index += 1;
                continue;
            }

            // Skip frames that are done animating
            let frame_image_index: i32 = (curr_frame_index as i32) - (iter_frame_index as i32);
            if frame_image_index >= 12 {
                iter_frame_index += 1;
                continue;
            }

            // Apply physics
            for particle in frame.iter_mut() {
                particle.vspeed -= particle.gravity;
                particle.x += particle.hspeed;
                particle.y += particle.vspeed;
            }

            iter_frame_index += 1;
        }

        self.frame_index += 1;
    }

    pub fn set_start_process_frame(&mut self, start_process_frame_index: usize) {
        self.start_process_index = start_process_frame_index;
    }

    pub fn is_finished(&self) -> bool {
        (self.frame_index as i32) - ((self.particle_frames.len() - 1) as i32) >= 12
    }

    pub fn get_frame_count(&self) -> usize {
        self.particle_frames.len()
    }

    pub fn get_length(&self) -> usize {
        self.particle_frames.len() + 11
    }

    pub fn get_frames(&self) -> &Vec<Vec<DustParticle>> {
        &self.particle_frames
    }

    pub fn get_frame_rng_offset(&self, index: usize) -> usize {
        self.particle_frame_rng_offsets[index]
    }

    pub fn is_close_match(&self, points: &Vec<(f32, f32)>) -> bool {
        let curr_frame_index = self.frame_index;

        'point_loop: for point in points.iter() {
            let mut iter_frame_index: u32 = self.start_process_index as u32;
            for frame in self.particle_frames.iter().skip(self.start_process_index) {
                // Skip frames that haven't started animating yet
                if iter_frame_index > curr_frame_index {
                    iter_frame_index += 1;
                    continue;
                }
    
                // Skip frames that are done animating
                let frame_image_index: i32 = (curr_frame_index as i32) - (iter_frame_index as i32);
                if frame_image_index >= 12 {
                    iter_frame_index += 1;
                    continue;
                }
                
                for particle in frame.iter() {
                    if f32::abs(point.0 - particle.x) <= 0.5 && f32::abs(point.1 - particle.y) <= 0.5 {
                        continue 'point_loop;
                    }
                    //if f32::sqrt(((point.0 - particle.x) * (point.0 - particle.x)) + ((point.1 - particle.y) * (point.1 - particle.y))) <= 2.0 {
                    //    continue 'point_loop;
                    //}
                }

                iter_frame_index += 1;
            }

            // Nothing matched for this point, abort.
            return false;
        }
        true
    }

    pub fn get_total_rng_calls(&self) -> usize {
        let mut count: usize = 0;
        let mut rng_skip_amount: usize = 0;
        for frame in self.particle_frames.iter() {
            // Count particles
            count += frame.len() * 2;
            
            // Count RNG from "you won" text
            count += rng_skip_amount;
            rng_skip_amount += 2;
        }
        count
    }

    pub fn get_after_battle_rng_calls(&self, text_length: usize) -> usize {
        let mut count: usize = 0;
        let mut rng_skip_amount: usize = 0;
        for frame in self.particle_frames.iter() {
            // Count particles
            count += frame.len() * 2;
            
            // Count RNG from "you won" text
            count += rng_skip_amount;
            rng_skip_amount += 2;
        }

        // Count RNG from all remaining "you won" text
        let remaining_frames = text_length - self.particle_frames.len();
        let mut i = 0;
        while i < remaining_frames {
            count += rng_skip_amount;
            i += 1;
            if i < remaining_frames - 1 { // Last frame doesn't add any extra characters
                rng_skip_amount += 2;
            }
        }

        count
    }
}
