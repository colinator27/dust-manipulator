use std::{sync::{atomic::{AtomicBool, Ordering}, Arc, Mutex}, thread, time::Instant};

use sdl3::sys::gpu::{SDL_GPUComputePipeline, SDL_GPUDevice, SDL_GPUFence};

use crate::compute_shaders::{self, ComputePipelineInfo, GPUBufferInfo};

struct NamingComputeData {
    gpu_device: *mut SDL_GPUDevice,
    pipeline: *mut SDL_GPUComputePipeline,
    return_val_buffer: GPUBufferInfo,
    rng_seeds_buffer: GPUBufferInfo,
    rng_seeds_count: u32,
    preload_fence: *mut SDL_GPUFence
}

pub struct NamingSearchResult {
    pub match_count: u32,
    pub single_matched_seed: u32,
    pub single_matched_position: u32
}

fn preload(unique_seeds: &Vec<u32>) -> Result<NamingComputeData, &'static str> {
    // Create device and pipeline based on shader
    let device = compute_shaders::create_gpu_device()?;
    let pipeline = compute_shaders::create_compute_pipeline(device, &ComputePipelineInfo {
        shader_name: "naming.comp",
        num_readonly_storage_buffers: 1,
        num_readwrite_storage_buffers: 1,
        num_uniform_buffers: 1,
        threadcount_x: 64,
        ..Default::default()
    })?;

    // Create GPU buffers
    let return_val_buffer = compute_shaders::create_gpu_buffer(device, 16, false, true)?;
    let rng_seeds_count = unique_seeds.len() as u32;
    let rng_seeds_buffer = compute_shaders::create_gpu_buffer(device, unique_seeds.len() * 4, true, false)?;

    // Copy RNG seed data into its GPU buffer ahead of time
    let command_buffer = compute_shaders::begin_command_buffer(device)?;
    let copy_pass = compute_shaders::begin_copy_pass(command_buffer)?;
    compute_shaders::upload_to_gpu_buffer_u32(device, copy_pass, &rng_seeds_buffer, &unique_seeds)?;
    compute_shaders::end_copy_pass(copy_pass);
    let preload_fence = compute_shaders::end_command_buffer_and_get_fence(command_buffer)?;

    Ok(NamingComputeData { 
        gpu_device: device,
        pipeline,
        return_val_buffer,
        rng_seeds_buffer,
        rng_seeds_count,
        preload_fence
    })
}

fn search(data: &mut NamingComputeData, params: &NamingSearchParameters) -> Result<NamingSearchResult, &'static str> {
    // If there's a preload fence to wait for and/or release, do so
    if !data.preload_fence.is_null() {
        let success = compute_shaders::wait_for_and_release_fences(data.gpu_device, &[data.preload_fence]);
        data.preload_fence = std::ptr::null_mut() as *mut SDL_GPUFence;

        if !success {
            return Err("Preload command buffer was unsuccessful");
        }
    }

    // Verify number of pixels are valid, and build match integers
    if params.matching_pixels.len() != 104 {
        return Err("Expected 104 matching pixels");
    }
    let mut match1: u32 = 0;
    let mut match2: u32 = 0;
    let mut match3: u32 = 0;
    let mut match4: u32 = 0;
    for i in 0..8 {
        if params.matching_pixels[i] {
            match4 |= 1 << (7 - i);
        }
    }
    for i in 8..40 {
        if params.matching_pixels[i] {
            match3 |= 1 << (31 - (i - 8));
        }
    }
    for i in 40..72 {
        if params.matching_pixels[i] {
            match2 |= 1 << (31 - (i - 40));
        }
    }
    for i in 72..104 {
        if params.matching_pixels[i] {
            match1 |= 1 << (31 - (i - 72));
        }
    }

    // Create command buffer for all operations
    let command_buffer = compute_shaders::begin_command_buffer(data.gpu_device)?;

    // Copy search data to GPU buffers
    let copy_pass = compute_shaders::begin_copy_pass(command_buffer)?;
    compute_shaders::upload_to_gpu_buffer(data.gpu_device, copy_pass, &data.return_val_buffer, &[0; 12])?;
    compute_shaders::end_copy_pass(copy_pass);

    // Push uniform data
    let mut random_flags = 0;
    if params.rng_15bit {
        random_flags |= 1 << 0;
    }
    if params.rng_signed {
        random_flags |= 1 << 1;
    }
    if params.rng_old_poly {
        random_flags |= 1 << 2;
    }
    let uniform_data = [
        u32::to_ne_bytes(random_flags), 
        u32::to_ne_bytes(params.search_range), 
        u32::to_ne_bytes(match1), 
        u32::to_ne_bytes(match2),
        u32::to_ne_bytes(match3),
        u32::to_ne_bytes(match4)
    ].concat();
    compute_shaders::push_uniform_data(command_buffer, 0, &uniform_data);

    // Run main search operation
    let writeable_buffer_storage = [data.return_val_buffer.raw()];
    let all_buffer_storage_ordered = [data.rng_seeds_buffer.raw()];
    compute_shaders::perform_buffer_compute(command_buffer, data.pipeline, &writeable_buffer_storage, &all_buffer_storage_ordered, data.rng_seeds_count / 64, 1, 1)?;

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
    Ok(NamingSearchResult {
        match_count: u32::from_ne_bytes(output_buffer[0..4].try_into().unwrap()),
        single_matched_seed: u32::from_ne_bytes(output_buffer[4..8].try_into().unwrap()),
        single_matched_position: u32::from_ne_bytes(output_buffer[8..12].try_into().unwrap())
    })
}

fn unload(data: &NamingComputeData) {
    compute_shaders::free_gpu_buffer(data.gpu_device, &data.rng_seeds_buffer);
    compute_shaders::free_gpu_buffer(data.gpu_device, &data.return_val_buffer);
    compute_shaders::free_compute_pipeline(data.gpu_device, data.pipeline);
    compute_shaders::free_gpu_device(data.gpu_device);
}

pub struct NamingSearchParameters {
    pub rng_15bit: bool,
    pub rng_signed: bool,
    pub rng_old_poly: bool,
    pub search_range: u32,
    pub matching_pixels: Vec<bool>
}

pub fn thread_func(end_thread: Arc<AtomicBool>, perform_search: Arc<AtomicBool>,
                   unique_seeds: Arc<Mutex<Vec<u32>>>, parameters: Arc<Mutex<NamingSearchParameters>>,
                   output: Arc<Mutex<NamingSearchResult>>) {
    println!("Naming compute thread started");
    let mut naming_data = preload(&unique_seeds.lock().unwrap()).expect("Failed to preload");

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
        let search_result = match search(&mut naming_data, &params) {
            Ok(result) => result,
            Err(e) => {
                *output.lock().unwrap() = NamingSearchResult {
                    match_count: 0,
                    single_matched_seed: 0,
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

    unload(&naming_data);
    println!("Naming compute thread ended");
}