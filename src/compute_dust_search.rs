use std::{sync::{atomic::{AtomicBool, Ordering}, Arc, Mutex}, thread, time::Instant};

use const_format::formatcp;
use sdl3::sys::gpu::{SDL_GPUComputePipeline, SDL_GPUDevice, SDL_GPUFence};

use crate::{compute_shaders::{self, ComputePipelineInfo, GPUBufferInfo, PointU32}, dust::SPARE_RNG_LENGTH, rng::PrecomputedRNG};

struct DustComputeData {
    gpu_device: *mut SDL_GPUDevice,
    pipeline_last_frame: *mut SDL_GPUComputePipeline,
    pipeline_last_frame_early: *mut SDL_GPUComputePipeline,
    pipeline_second_last_frame: *mut SDL_GPUComputePipeline,
    pipeline_second_last_frame_early: *mut SDL_GPUComputePipeline,
    pipeline_spare: *mut SDL_GPUComputePipeline,
    return_val_buffer: GPUBufferInfo,
    rng_buffer: GPUBufferInfo,
    rng_length: usize,
    match_positions_buffer: GPUBufferInfo,
    initial_particles_buffer: GPUBufferInfo,
    preload_fence: *mut SDL_GPUFence
}

pub struct DustSearchResult {
    pub match_count: u32,
    pub single_matched_position: u32,
    pub spare_frame_index: u32
}

pub const MAX_INITIAL_PARTICLES: usize = 128;
pub const MAX_MATCHING_PARTICLES: usize = 32;

fn preload(prng: &PrecomputedRNG) -> Result<DustComputeData, &'static str> {
    // Create device and pipelines based on shaders
    let device = compute_shaders::create_gpu_device()?;
    let pipeline_last_frame = compute_shaders::create_compute_pipeline(device, &ComputePipelineInfo {
        shader_name: "dust_last_frame.comp",
        num_readonly_storage_buffers: 3,
        num_readwrite_storage_buffers: 1,
        num_uniform_buffers: 1,
        threadcount_x: 64,
        ..Default::default()
    })?;
    let pipeline_last_frame_early = compute_shaders::create_compute_pipeline(device, &ComputePipelineInfo {
        shader_name: "dust_last_frame_early.comp",
        num_readonly_storage_buffers: 3,
        num_readwrite_storage_buffers: 1,
        num_uniform_buffers: 1,
        threadcount_x: 64,
        ..Default::default()
    })?;
    let pipeline_second_last_frame = compute_shaders::create_compute_pipeline(device, &ComputePipelineInfo {
        shader_name: "dust_second_last_frame.comp",
        num_readonly_storage_buffers: 3,
        num_readwrite_storage_buffers: 1,
        num_uniform_buffers: 1,
        threadcount_x: 64,
        ..Default::default()
    })?;
    let pipeline_second_last_frame_early = compute_shaders::create_compute_pipeline(device, &ComputePipelineInfo {
        shader_name: "dust_second_last_frame_early.comp",
        num_readonly_storage_buffers: 3,
        num_readwrite_storage_buffers: 1,
        num_uniform_buffers: 1,
        threadcount_x: 64,
        ..Default::default()
    })?;
    let pipeline_spare = compute_shaders::create_compute_pipeline(device, &ComputePipelineInfo {
        shader_name: "dust_spare.comp",
        num_readonly_storage_buffers: 2,
        num_readwrite_storage_buffers: 1,
        num_uniform_buffers: 1,
        threadcount_x: 64,
        ..Default::default()
    })?;

    // Create GPU buffers
    let return_val_buffer = compute_shaders::create_gpu_buffer(device, 16, false, true)?;
    let rng_length= prng.raw().len();
    let rng_buffer = compute_shaders::create_gpu_buffer(device, rng_length * 4, true, false)?;
    let match_positions_buffer = compute_shaders::create_gpu_buffer(device, MAX_MATCHING_PARTICLES * 4, true, false)?;
    let initial_particles_buffer = compute_shaders::create_gpu_buffer(device, MAX_INITIAL_PARTICLES * 4, true, false)?;

    // Copy RNG data into its GPU buffer ahead of time
    let command_buffer = compute_shaders::begin_command_buffer(device)?;
    let copy_pass = compute_shaders::begin_copy_pass(command_buffer)?;
    compute_shaders::upload_to_gpu_buffer_u32(device, copy_pass, &rng_buffer, &prng.raw())?;
    compute_shaders::end_copy_pass(copy_pass);
    let preload_fence = compute_shaders::end_command_buffer_and_get_fence(command_buffer)?;

    Ok(DustComputeData { 
        gpu_device: device,
        pipeline_last_frame,
        pipeline_last_frame_early,
        pipeline_second_last_frame,
        pipeline_second_last_frame_early,
        pipeline_spare,
        return_val_buffer,
        rng_buffer,
        rng_length,
        match_positions_buffer,
        initial_particles_buffer,
        preload_fence
    })
}

fn points_to_bytes(points: &[PointU32]) -> &[u8] {
    let len = points.len().checked_mul(4).unwrap();
    let ptr: *const u8 = points.as_ptr().cast();
    unsafe { std::slice::from_raw_parts(ptr, len) }
}

fn search(data: &mut DustComputeData, params: &DustSearchParameters) -> Result<DustSearchResult, &'static str> {
    // If there's a preload fence to wait for and/or release, do so
    if !data.preload_fence.is_null() {
        let success = compute_shaders::wait_for_and_release_fences(data.gpu_device, &[data.preload_fence]);
        data.preload_fence = std::ptr::null_mut() as *mut SDL_GPUFence;

        if !success {
            return Err("Preload command buffer was unsuccessful");
        }
    }

    match &params {
        DustSearchParameters::None => {
            return Err("Missing search parameters");
        },
        DustSearchParameters::KillDustSearchParameters(params) => {
            // Verify parameters
            if params.matching_particles.len() == 0 {
                return Err("No match points");
            }
            if params.matching_particles.len() > MAX_MATCHING_PARTICLES {
                return Err(formatcp!("Too many match points, max is {}", MAX_MATCHING_PARTICLES));
            }
            if params.initial_particles.len() == 0 {
                return Err("No initial particles");
            }
            if params.initial_particles.len() > MAX_INITIAL_PARTICLES {
                return Err(formatcp!("Too many initial particles (max is {})", MAX_INITIAL_PARTICLES));
            }
            if (params.last_frame_particle_count + params.second_last_frame_particle_count) as usize != params.initial_particles.len() {
                return Err("Frame particle counts don't sum to total");
            }
            if data.rng_length < 10_000 {
                return Err("RNG length is too small");
            }
            if params.search_range as usize > (data.rng_length - 10_000) {
                return Err("Search range is too large");
            }

            // Create command buffer for all operations
            let command_buffer = compute_shaders::begin_command_buffer(data.gpu_device)?;

            // Copy search data to GPU buffers
            let copy_pass = compute_shaders::begin_copy_pass(command_buffer)?;
            compute_shaders::upload_to_gpu_buffer(data.gpu_device, copy_pass, &data.match_positions_buffer, points_to_bytes(&params.matching_particles))?;
            compute_shaders::upload_to_gpu_buffer(data.gpu_device, copy_pass, &data.initial_particles_buffer, points_to_bytes(&params.initial_particles))?;
            compute_shaders::upload_to_gpu_buffer(data.gpu_device, copy_pass, &data.return_val_buffer, &[0; 8])?;
            compute_shaders::end_copy_pass(copy_pass);

            // Push uniform data
            let uniform_data = match params.search_mode {
                DustSearchMode::LastFrame | DustSearchMode::LastFrameEarly => [
                    u32::to_ne_bytes(params.last_frame_particle_count), 
                    u32::to_ne_bytes(params.matching_particles.len() as u32), 
                    u32::to_ne_bytes(params.last_frame_rng_offset)
                ].concat(),
                DustSearchMode::SecondToLastFrame | DustSearchMode::SecondToLastFrameEarly => [
                    u32::to_ne_bytes(params.last_frame_particle_count), 
                    u32::to_ne_bytes(params.second_last_frame_particle_count), 
                    u32::to_ne_bytes(params.matching_particles.len() as u32), 
                    u32::to_ne_bytes(params.last_frame_rng_offset),
                    u32::to_ne_bytes(params.initial_rng_skip_amount)
                ].concat(),
                DustSearchMode::Spare => panic!()
            };
            compute_shaders::push_uniform_data(command_buffer, 0, &uniform_data);

            // Run main search operation
            let pipeline = match params.search_mode {
                DustSearchMode::LastFrame => data.pipeline_last_frame,
                DustSearchMode::LastFrameEarly => data.pipeline_last_frame_early,
                DustSearchMode::SecondToLastFrame => data.pipeline_second_last_frame,
                DustSearchMode::SecondToLastFrameEarly => data.pipeline_second_last_frame_early,
                DustSearchMode::Spare => panic!()
            };
            let writeable_buffer_storage = [data.return_val_buffer.raw()];
            let all_buffer_storage_ordered = [data.rng_buffer.raw(), data.initial_particles_buffer.raw(), data.match_positions_buffer.raw()];
            compute_shaders::perform_buffer_compute(command_buffer, pipeline, &writeable_buffer_storage, &all_buffer_storage_ordered, params.search_range / 64, 1, 1)?;

            // Start download of data from return value buffer
            let copy_pass = compute_shaders::begin_copy_pass(command_buffer)?;
            let download_transfer_buffer = compute_shaders::queue_download_from_gpu_buffer(data.gpu_device, copy_pass, &data.return_val_buffer)?;
            compute_shaders::end_copy_pass(copy_pass);

            // End command buffer and wait for it to finish
            let success = compute_shaders::end_command_buffer_and_wait_for_fence(data.gpu_device, command_buffer);
            if !success {
                return Err("Search command buffer was unsuccessful");
            }

            // Finish download of data from return value buffer
            let mut output_buffer: Vec<u8> = vec![0; 8];
            compute_shaders::finish_download_from_gpu_buffer(data.gpu_device, &data.return_val_buffer, download_transfer_buffer, &mut output_buffer)?;

            // Interpret final data
            Ok(DustSearchResult {
                match_count: u32::from_ne_bytes(output_buffer[0..4].try_into().unwrap()),
                single_matched_position: u32::from_ne_bytes(output_buffer[4..8].try_into().unwrap()),
                spare_frame_index: 0
            })
        },
        DustSearchParameters::SpareDustSearchParameters(params) => {
            // Verify number of points are valid
            if params.matching_particles.len() == 0 {
                return Err("No match points");
            }
            if params.matching_particles.len() > MAX_MATCHING_PARTICLES {
                return Err(formatcp!("Too many match points, max is {}", MAX_MATCHING_PARTICLES));
            }

            // Verify and correct search range
            let mut search_range = params.search_range;
            if search_range < SPARE_RNG_LENGTH as u32 {
                return Err("Search range is too small");
            }
            if data.rng_length < SPARE_RNG_LENGTH {
                return Err("RNG length is too small");
            }
            if search_range as usize > (data.rng_length - SPARE_RNG_LENGTH) {
                search_range = (data.rng_length - SPARE_RNG_LENGTH) as u32;
            }

            // Create command buffer for all operations
            let command_buffer = compute_shaders::begin_command_buffer(data.gpu_device)?;

            // Copy search data to GPU buffers
            let copy_pass = compute_shaders::begin_copy_pass(command_buffer)?;
            compute_shaders::upload_to_gpu_buffer(data.gpu_device, copy_pass, &data.match_positions_buffer, points_to_bytes(&params.matching_particles))?;
            compute_shaders::upload_to_gpu_buffer(data.gpu_device, copy_pass, &data.return_val_buffer, &[0; 12])?;
            compute_shaders::end_copy_pass(copy_pass);

            // Push uniform data
            let uniform_data = [
                u32::to_ne_bytes(params.anim_position.value()), 
                u32::to_ne_bytes(params.anim_size.value()), 
                u32::to_ne_bytes(params.matching_particles.len() as u32), 
            ].concat();
            compute_shaders::push_uniform_data(command_buffer, 0, &uniform_data);

            // Run main search operation
            let writeable_buffer_storage = [data.return_val_buffer.raw()];
            let all_buffer_storage_ordered = [data.rng_buffer.raw(), data.match_positions_buffer.raw()];
            compute_shaders::perform_buffer_compute(command_buffer, data.pipeline_spare, &writeable_buffer_storage, &all_buffer_storage_ordered, search_range / 64, 1, 1)?;

            // Start download of data from return value buffer
            let copy_pass = compute_shaders::begin_copy_pass(command_buffer)?;
            let download_transfer_buffer = compute_shaders::queue_download_from_gpu_buffer(data.gpu_device, copy_pass, &data.return_val_buffer)?;
            compute_shaders::end_copy_pass(copy_pass);

            // End command buffer and wait for it to finish
            let success = compute_shaders::end_command_buffer_and_wait_for_fence(data.gpu_device, command_buffer);
            if !success {
                return Err("Search command buffer was unsuccessful");
            }

            // Finish download of data from return value buffer
            let mut output_buffer: Vec<u8> = vec![0; 12];
            compute_shaders::finish_download_from_gpu_buffer(data.gpu_device, &data.return_val_buffer, download_transfer_buffer, &mut output_buffer)?;

            // Interpret final data
            Ok(DustSearchResult {
                match_count: u32::from_ne_bytes(output_buffer[0..4].try_into().unwrap()),
                single_matched_position: u32::from_ne_bytes(output_buffer[4..8].try_into().unwrap()),
                spare_frame_index: u32::from_ne_bytes(output_buffer[8..12].try_into().unwrap())
            })
        }
    }
}

// TODO: function to replace RNG sequence with a different one (e.g. to reduce search range)

fn unload(data: &DustComputeData) {
    compute_shaders::free_gpu_buffer(data.gpu_device, &data.initial_particles_buffer);
    compute_shaders::free_gpu_buffer(data.gpu_device, &data.match_positions_buffer);
    compute_shaders::free_gpu_buffer(data.gpu_device, &data.rng_buffer);
    compute_shaders::free_gpu_buffer(data.gpu_device, &data.return_val_buffer);
    compute_shaders::free_compute_pipeline(data.gpu_device, data.pipeline_last_frame);
    compute_shaders::free_compute_pipeline(data.gpu_device, data.pipeline_last_frame_early);
    compute_shaders::free_compute_pipeline(data.gpu_device, data.pipeline_second_last_frame);
    compute_shaders::free_compute_pipeline(data.gpu_device, data.pipeline_second_last_frame_early);
    compute_shaders::free_compute_pipeline(data.gpu_device, data.pipeline_spare);
    compute_shaders::free_gpu_device(data.gpu_device);
}

#[derive(Clone, Copy, PartialEq)]
pub enum DustSearchMode {
    LastFrame,
    LastFrameEarly,
    SecondToLastFrame,
    SecondToLastFrameEarly,
    Spare
}

impl DustSearchMode {
    pub fn to_early(&self) -> Self {
        match self {
            DustSearchMode::LastFrame => DustSearchMode::LastFrameEarly,
            DustSearchMode::SecondToLastFrame => DustSearchMode::SecondToLastFrameEarly,
            _ => self.clone()
        }
    }
    pub fn to_normal(&self) -> Self {
        match self {
            DustSearchMode::LastFrameEarly => DustSearchMode::LastFrame,
            DustSearchMode::SecondToLastFrameEarly => DustSearchMode::SecondToLastFrame,
            _ => self.clone()
        }
    }
}

// probably make this an enum called DustSearchParameters using these structs...
pub struct KillDustSearchParameters {
    pub search_mode: DustSearchMode,
    pub search_range: u32,
    pub last_frame_rng_offset: u32,
    pub initial_rng_skip_amount: u32,
    pub matching_particles: Vec<PointU32>,
    pub initial_particles: Vec<PointU32>,
    pub last_frame_particle_count: u32,
    pub second_last_frame_particle_count: u32
}

pub struct SpareDustSearchParameters {
    pub anim_position: PointU32,
    pub anim_size: PointU32,
    pub search_range: u32, // note: needs to have RNG length of animation subtracted (which is constant...)
    pub matching_particles: Vec<PointU32>
}

pub enum DustSearchParameters {
    None,
    KillDustSearchParameters(KillDustSearchParameters),
    SpareDustSearchParameters(SpareDustSearchParameters)
}

pub fn thread_func(end_thread: Arc<AtomicBool>, perform_search: Arc<AtomicBool>,
                   prng: Arc<PrecomputedRNG>, parameters: Arc<Mutex<DustSearchParameters>>,
                   output: Arc<Mutex<DustSearchResult>>) {
    println!("Dust compute thread started");
    let mut dust_last_frame_data = preload(&prng).expect("Failed to preload");

    loop {
        // Wait until an end thread or perform search signal are sent
        let mut end_thread_signal = false;
        let mut perform_search_signal = false;
        loop {
            if end_thread.load(Ordering::Relaxed) {
                end_thread.store(false, Ordering::Relaxed);
                end_thread_signal = true;
                break;
            }
            if perform_search.load(Ordering::Relaxed) {
                perform_search_signal = true;
                break;
            }
            thread::park();
        }
        if end_thread_signal {
            break;
        }
        if !perform_search_signal {
            continue;
        }
        
        let now = Instant::now();

        // Begin search with current parameters
        let params = parameters.lock().unwrap();
        let search_result = match search(&mut dust_last_frame_data, &params) {
            Ok(result) => result,
            Err(e) => {
                *output.lock().unwrap() = DustSearchResult {
                    match_count: 0,
                    single_matched_position: 0,
                    spare_frame_index: 0
                };
                perform_search.store(false, Ordering::Relaxed);
                println!("Error occurred during search: {}", e);
                continue;
            }
        };

        let elapsed = now.elapsed();
        println!("Elapsed: {:.2?}", elapsed);

        // Output results
        *output.lock().unwrap() = search_result;

        // Allow new searches to be performed
        perform_search.store(false, Ordering::Relaxed);
    }

    unload(&dust_last_frame_data);
    println!("Dust last frame compute thread ended");
}