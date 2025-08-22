use std::{sync::{atomic::{AtomicBool, Ordering}, Arc, Mutex}, thread, time::Instant};

use sdl3::sys::gpu::{SDL_GPUComputePipeline, SDL_GPUDevice, SDL_GPUFence};

use crate::{compute_shaders::{self, ComputePipelineInfo, GPUBufferInfo, PointU32}, rng::PrecomputedRNG};

struct SnowballComputeData {
    gpu_device: *mut SDL_GPUDevice,
    pipeline: *mut SDL_GPUComputePipeline,
    return_val_buffer: GPUBufferInfo,
    rng_buffer: GPUBufferInfo,
    match_positions_buffer: GPUBufferInfo,
    preload_fence: *mut SDL_GPUFence
}

pub struct SnowballSearchResult {
    pub match_count: u32,
    pub single_matched_position: u32
}

fn preload(prng: &PrecomputedRNG) -> Result<SnowballComputeData, &'static str> {
    // Create device and pipelines based on shaders
    let device = compute_shaders::create_gpu_device()?;
    let pipeline = compute_shaders::create_compute_pipeline(device, &ComputePipelineInfo {
        shader_name: "snowballs.comp",
        num_readonly_storage_buffers: 2,
        num_readwrite_storage_buffers: 1,
        num_uniform_buffers: 1,
        threadcount_x: 64,
        ..Default::default()
    })?;

    // Create GPU buffers
    let return_val_buffer = compute_shaders::create_gpu_buffer(device, 16, false, true)?;
    let rng_buffer = compute_shaders::create_gpu_buffer(device, prng.raw().len() * 4, true, false)?;
    let match_positions_buffer = compute_shaders::create_gpu_buffer(device, 32 * 4, true, false)?;

    // Copy RNG data into its GPU buffer ahead of time
    let command_buffer = compute_shaders::begin_command_buffer(device)?;
    let copy_pass = compute_shaders::begin_copy_pass(command_buffer)?;
    compute_shaders::upload_to_gpu_buffer_u32(device, copy_pass, &rng_buffer, &prng.raw())?;
    compute_shaders::end_copy_pass(copy_pass);
    let preload_fence = compute_shaders::end_command_buffer_and_get_fence(command_buffer)?;

    Ok(SnowballComputeData { 
        gpu_device: device,
        pipeline,
        return_val_buffer,
        rng_buffer,
        match_positions_buffer,
        preload_fence
    })
}

fn points_to_bytes(points: &[PointU32]) -> &[u8] {
    let len = points.len().checked_mul(4).unwrap();
    let ptr: *const u8 = points.as_ptr().cast();
    unsafe { std::slice::from_raw_parts(ptr, len) }
}

fn search(data: &mut SnowballComputeData, params: &SnowballSearchParameters) -> Result<SnowballSearchResult, &'static str> {
    // If there's a preload fence to wait for and/or release, do so
    if !data.preload_fence.is_null() {
        let success = compute_shaders::wait_for_and_release_fences(data.gpu_device, &[data.preload_fence]);
        data.preload_fence = std::ptr::null_mut() as *mut SDL_GPUFence;

        if !success {
            return Err("Preload command buffer was unsuccessful");
        }
    }

    // Verify number of points are valid
    if params.matching_snowballs.len() == 0 {
        return Err("No match points");
    }
    if params.matching_snowballs.len() > 32 {
        return Err("Too many match points (max is 32)");
    }

    // Create command buffer for all operations
    let command_buffer = compute_shaders::begin_command_buffer(data.gpu_device)?;

    // Copy search data to GPU buffers
    let copy_pass = compute_shaders::begin_copy_pass(command_buffer)?;
    compute_shaders::upload_to_gpu_buffer(data.gpu_device, copy_pass, &data.match_positions_buffer, points_to_bytes(&params.matching_snowballs))?;
    compute_shaders::upload_to_gpu_buffer(data.gpu_device, copy_pass, &data.return_val_buffer, &[0; 8])?;
    compute_shaders::end_copy_pass(copy_pass);

    // Push uniform data
    let uniform_data = [
        u32::to_ne_bytes(params.matching_snowballs.len() as u32), 
    ].concat();
    compute_shaders::push_uniform_data(command_buffer, 0, &uniform_data);

    // Run main search operation
    let writeable_buffer_storage = [data.return_val_buffer.raw()];
    let all_buffer_storage_ordered = [data.rng_buffer.raw(), data.match_positions_buffer.raw()];
    compute_shaders::perform_buffer_compute(command_buffer, data.pipeline, &writeable_buffer_storage, &all_buffer_storage_ordered, params.search_range / 64, 1, 1)?;

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
    Ok(SnowballSearchResult {
        match_count: u32::from_ne_bytes(output_buffer[0..4].try_into().unwrap()),
        single_matched_position: u32::from_ne_bytes(output_buffer[4..8].try_into().unwrap())
    })
}

// TODO: function to replace RNG sequence with a different one (e.g. to reduce search range)

fn unload(data: &SnowballComputeData) {
    compute_shaders::free_gpu_buffer(data.gpu_device, &data.match_positions_buffer);
    compute_shaders::free_gpu_buffer(data.gpu_device, &data.rng_buffer);
    compute_shaders::free_gpu_buffer(data.gpu_device, &data.return_val_buffer);
    compute_shaders::free_compute_pipeline(data.gpu_device, data.pipeline);
    compute_shaders::free_gpu_device(data.gpu_device);
}

pub struct SnowballSearchParameters {
    pub search_range: u32,
    pub matching_snowballs: Vec<PointU32>
}

pub fn thread_func(end_thread: Arc<AtomicBool>, perform_search: Arc<AtomicBool>,
                   prng: Arc<Mutex<PrecomputedRNG>>, parameters: Arc<Mutex<SnowballSearchParameters>>,
                   output: Arc<Mutex<SnowballSearchResult>>) {
    println!("Snowball compute thread started");
    let mut snowball_data = preload(&prng.lock().unwrap()).expect("Failed to preload");

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
        let search_result = match search(&mut snowball_data, &params) {
            Ok(result) => result,
            Err(e) => {
                *output.lock().unwrap() = SnowballSearchResult {
                    match_count: 0,
                    single_matched_position: 0
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

    unload(&snowball_data);
    println!("Snowball compute thread ended");
}