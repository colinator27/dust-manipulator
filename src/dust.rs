use std::usize;

use sdl3::{pixels::Color, rect::Rect, render::{Canvas, FRect}, video::Window};

use crate::{compute_dust_search::DustSearchMode, encounter_data::GML_EPSILON, rng::PrecomputedRNG};

pub struct KillDustData {
    pub data: &'static str,
    pub wide: bool,
    pub search_mode: DustSearchMode,
    pub x: f32,
    pub y: f32
}

#[derive(Clone, Copy)]
pub struct SpareDustData {
    pub sprite_width: f32,
    pub sprite_height: f32,
    pub x: f32,
    pub y: f32
}

pub enum DustData {
    KillDustData(KillDustData),
    SpareDustData(SpareDustData)
}

impl DustData {
    pub const fn new_kill(data: &'static str, wide: bool, search_mode: DustSearchMode) -> Self {
        Self::KillDustData(KillDustData {
            data,
            wide,
            search_mode,
            x: 0.0,
            y: 0.0
        })
    }
    pub const fn new_spare(sprite_width: f32, sprite_height: f32) -> Self {
        Self::SpareDustData(SpareDustData {
            sprite_width,
            sprite_height,
            x: 0.0,
            y: 0.0
        })
    }
    pub fn to_search_config(&self, x: f32, y: f32, 
                            text_length: usize, text_length_lvup: usize, view_rect: Rect, slides_left: bool, kill_count: u32) -> DustSearchConfig {
        match self {
            Self::KillDustData(data) => {
                DustSearchConfig {
                    dust_data: Self::KillDustData(KillDustData {
                        x, y,
                        ..*data
                    }),
                    text_length,
                    text_length_lvup,
                    view_rect,
                    slides_left,
                    kill_count
                }
            },
            Self::SpareDustData(data) => {
                DustSearchConfig {
                    dust_data: Self::SpareDustData(SpareDustData {
                        x, y,
                        ..*data
                    }),
                    text_length,
                    text_length_lvup,
                    view_rect,
                    slides_left,
                    kill_count
                }
            },
        }
    }
}

pub struct DustSearchConfig {
    pub dust_data: DustData,
    pub text_length: usize,
    pub text_length_lvup: usize,
    pub view_rect: Rect,
    pub slides_left: bool,
    pub kill_count: u32
}

#[derive(Clone)]
pub struct KillDustAnimation {
    pub particle_frames: Vec<Vec<KillDustParticle>>,
    pub particle_frame_rng_offsets: Vec<usize>,
    pub frame_index: u32,
    pub start_process_index: usize
}

impl KillDustAnimation {
    pub fn compute_frame_rng_offsets(&mut self) {
        let mut rng_skip_amount: usize = 0;
        let mut rng_position = 0;
        self.particle_frame_rng_offsets.clear();
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

    pub fn get_frames(&self) -> &Vec<Vec<KillDustParticle>> {
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

#[derive(Clone)]
pub struct KillDustParticle {
    x: f32,
    y: f32,
    hspeed: f32,
    vspeed: f32,
    gravity: f32,
    xscale: f32,
}
impl KillDustParticle {
    pub fn get_x(&self) -> f32 {
        self.x
    }
    pub fn get_y(&self) -> f32 {
        self.y
    }
}

pub const SPARE_DUST_PARTICLE_COUNT: usize = 14;
pub const SPARE_RNG_LENGTH: usize = (SPARE_DUST_PARTICLE_COUNT * 4) + 38;

#[derive(Clone)]
pub struct SpareDustAnimation {
    pub particles: [SpareDustParticle; SPARE_DUST_PARTICLE_COUNT],
    pub frame_index: u32,
    pub remaining_speed: f32,
    pub dust_data: SpareDustData
}

#[derive(Clone, Copy)]
pub struct SpareDustParticle {
    pub x: f32,
    pub y: f32,
    pub size: f32,
    pub normalized_x_movement: f32,
    pub normalized_y_movement: f32
}

#[derive(Clone)]
pub enum DustAnimation {
    KillDustAnimation(KillDustAnimation),
    SpareDustAnimation(SpareDustAnimation)
}

const DUST_END_LINE: u8 = '}' as u8;
const DUST_END: u8 = '~' as u8;

impl DustData {
    pub fn create_animation(&self) -> DustAnimation {
        match self {
            DustData::KillDustData(dust_data) => {
                // Create animation struct
                let mut anim = KillDustAnimation {
                    particle_frames: Vec::with_capacity(32),
                    particle_frame_rng_offsets: Vec::with_capacity(32),
                    frame_index: 0,
                    start_process_index: 0
                };

                // Create enumerator over data
                let mut data_enum = dust_data.data.bytes().enumerate();

                // Read data, constructing spawn frames and particles within those frames.
                // Start at the top.
                let mut curr_y = 0;
                'read_loop: loop {
                    // Each loop is a single frame
                    let mut frame: Vec<KillDustParticle> = Vec::with_capacity(4);

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
                                if dust_data.wide {
                                    // Wide mode - collapses multiple particles into one wider particle
                                    frame.push(KillDustParticle {
                                        x: dust_data.x + (curr_x as f32),
                                        y: dust_data.y + (curr_y as f32),
                                        xscale: ((curr_char as f32) - 40.0) * 2.0,
                                        hspeed: 0.0,
                                        vspeed: 0.0,
                                        gravity: 0.0,
                                    });
                                    curr_x += ((curr_char - 40) as u32) * 2;
                                } else {
                                    // Non-wide mode - every particle is the same size
                                    for _ in 0..(curr_char - 40) {
                                        frame.push(KillDustParticle {
                                            x: dust_data.x + (curr_x as f32),
                                            y: dust_data.y + (curr_y as f32) + 2.0, // Strange offset...
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
                DustAnimation::KillDustAnimation(anim)
            },
            DustData::SpareDustData(dust_data) => {
                // Create animation struct
                let anim = SpareDustAnimation {
                    particles: [SpareDustParticle{x: 0.0, y: 0.0, size: 0.0, normalized_x_movement: 0.0, normalized_y_movement: 0.0}; SPARE_DUST_PARTICLE_COUNT],
                    frame_index: 0,
                    remaining_speed: 8.0,
                    dust_data: *dust_data
                };

                DustAnimation::SpareDustAnimation(anim)
            }
        }
    }
}

const DUST_COLORS: [u8; 12] = [255, 251, 251, 251, 251, 251, 251, 219, 182, 146, 109, 73];
const ROUNDING_OFFSET: f32 = 1.0 / 512.0; // Rounding offset as used by Direct3D

impl DustAnimation {
    pub fn draw(&self, canvas: &mut Canvas<Window>, x: f32, y: f32, change_color: bool) {
        match self {
            DustAnimation::KillDustAnimation(anim) => {
                let mut iter_frame_index: u32 = anim.start_process_index as u32;
                for frame in anim.particle_frames.iter().skip(anim.start_process_index) {
                    // Skip frames that are done animating
                    let frame_image_index: i32 = i32::max((anim.frame_index as i32) - (iter_frame_index as i32), 0);
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
            },
            DustAnimation::SpareDustAnimation(anim) => {
               for particle in &anim.particles {
                    _ = canvas.draw_rect(FRect::new(f32::round(particle.x + x - ROUNDING_OFFSET), f32::round(particle.y + y - ROUNDING_OFFSET),  particle.size, particle.size));
               }
            }
        }
    }

    pub fn start_animating(&mut self, rng: &PrecomputedRNG, mut rng_position: usize) {
        match self {
            Self::KillDustAnimation(anim) => {
                let mut rng_skip_amount: usize = 0;
                let rng_start_position = rng_position;

                //let mut frame_index = 0;
                //let num_frames = self.particle_frames.len();

                anim.particle_frame_rng_offsets.clear();
                for frame in anim.particle_frames.iter_mut() {
                    // Track frame RNG offsets
                    anim.particle_frame_rng_offsets.push(rng_position - rng_start_position);

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
            },
            Self::SpareDustAnimation(anim) => {
                let x = anim.dust_data.x as f64;
                let y = anim.dust_data.y as f64;
                let sprite_width = anim.dust_data.sprite_width as f64;
                let sprite_height = anim.dust_data.sprite_height as f64;

                // Set initial positions and directions of particles
                let mut i = 0;
                for particle in &mut anim.particles {
                    // Yes, the extra sprite_width here is *not* a typo...
                    particle.y = ((rng.get_f64(sprite_height / 2.0, rng_position + (i * 3)) + (sprite_width / 4.0) + y) - 8.0) as f32;
                    particle.x = ((rng.get_f64(sprite_width / 2.0, rng_position + (i * 3) + 1) + (sprite_width / 4.0) + x) - 8.0) as f32;
                    particle.size = (16.0 * (rng.get_f64(1.0, rng_position + (i * 3) + 2) + 0.7)) as f32;

                    let right_side = ((particle.x as f64 + 8.0) - x) / (sprite_width / 2.0);
                    let top_side = ((particle.y as f64 + 8.0) - y) / (sprite_height / 2.0);
                    let mut direction = -(rng.get_f64(360.0, rng_position + (SPARE_DUST_PARTICLE_COUNT * 3) + 38 + i) as f32);
                    if direction < -0.0 {
                        direction += 360.0;
                    }

                    // Let's just hope the compiler is able to be a bit smarter about this.
                    if right_side <= (0.75 - GML_EPSILON) {
                        direction = -180.0 + 360.0;
                    }
                    if right_side >= (1.25 + GML_EPSILON) {
                        direction = 0.0;
                    }
                    if top_side >= (1.25 + GML_EPSILON) && right_side >= (1.25 + GML_EPSILON) {
                        direction = -45.0 + 360.0;
                    }
                    if top_side >= (1.25 + GML_EPSILON) && right_side >= (0.75 + GML_EPSILON) && right_side <= (1.25 - GML_EPSILON) {
                        direction = -90.0 + 360.0;
                    }
                    if top_side >= (1.25 + GML_EPSILON) && right_side <= (0.75 - GML_EPSILON) {
                        direction = -135.0 + 360.0;
                    }
                    if top_side <= (0.75 - GML_EPSILON) && right_side >= (1.25 + GML_EPSILON) {
                        direction = -315.0 + 360.0;
                    }
                    if top_side <= (0.75 - GML_EPSILON) && right_side >= (0.75 + GML_EPSILON) && right_side <= (1.25 - GML_EPSILON) {
                        direction = -270.0 + 360.0;
                    }
                    if top_side <= (0.75 - GML_EPSILON) && right_side <= (0.75 - GML_EPSILON) {
                        direction = -235.0 + 360.0;
                    }

                    // Note: the trig functions used here are nondeterministic... but should be a decent approximation...
                    let angle_radians = (direction * 3.1415927) / 180.0;
                    particle.normalized_x_movement = f32::cos(angle_radians);
                    particle.normalized_y_movement = -f32::sin(angle_radians);

                    i += 1;
                }
            }
        }
    }

    pub fn update(&mut self) {
        match self {
            Self::KillDustAnimation(anim) => {
                let curr_frame_index = anim.frame_index;

                let mut iter_frame_index: u32 = anim.start_process_index as u32;
                for frame in anim.particle_frames.iter_mut().skip(anim.start_process_index) {
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
                        // Note: this is a simplification of some internal rounding, trigonometry, and floating point error that GM performs.
                        // In rare cases, this can result in off-by-one errors in the final results.
                        particle.vspeed -= particle.gravity;
                        particle.x += particle.hspeed;
                        particle.y += particle.vspeed;
                    }

                    iter_frame_index += 1;
                }

                anim.frame_index += 1;
            },
            Self::SpareDustAnimation(anim) => {
                // Apply friction to speed
                const FRICTION: f32 = 0.8;
                anim.remaining_speed -= FRICTION;
                if anim.remaining_speed < 0.0 {
                    anim.remaining_speed = 0.0;
                }
                let speed = anim.remaining_speed;

                // Move particles
                for particle in &mut anim.particles {
                    // Note: this is a simplification of some internal rounding, trigonometry, and floating point error that GM performs.
                    // In rare cases, this can result in off-by-one errors in the final results.
                    let hspeed = speed * particle.normalized_x_movement;
                    let vspeed = speed * particle.normalized_y_movement;
                    particle.x += hspeed;
                    particle.y += vspeed;
                }

                anim.frame_index += 1;
            }
        }
    }
}
