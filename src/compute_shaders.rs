use std::{ffi::{c_void, CStr}, fs::{self}, ptr};

use sdl3::{gpu::ShaderFormat, sys::gpu::{SDL_AcquireGPUCommandBuffer, SDL_BeginGPUComputePass, SDL_BeginGPUCopyPass, SDL_BindGPUComputePipeline, SDL_BindGPUComputeStorageBuffers, SDL_CreateGPUBuffer, SDL_CreateGPUComputePipeline, SDL_CreateGPUDevice, SDL_CreateGPUTransferBuffer, SDL_DestroyGPUDevice, SDL_DispatchGPUCompute, SDL_DownloadFromGPUBuffer, SDL_EndGPUComputePass, SDL_EndGPUCopyPass, SDL_GPUBuffer, SDL_GPUBufferCreateInfo, SDL_GPUBufferRegion, SDL_GPUCommandBuffer, SDL_GPUComputePipeline, SDL_GPUComputePipelineCreateInfo, SDL_GPUCopyPass, SDL_GPUDevice, SDL_GPUFence, SDL_GPUShaderFormat, SDL_GPUStorageBufferReadWriteBinding, SDL_GPUStorageTextureReadWriteBinding, SDL_GPUTransferBuffer, SDL_GPUTransferBufferCreateInfo, SDL_GPUTransferBufferLocation, SDL_GetGPUShaderFormats, SDL_MapGPUTransferBuffer, SDL_PushGPUComputeUniformData, SDL_ReleaseGPUBuffer, SDL_ReleaseGPUComputePipeline, SDL_ReleaseGPUFence, SDL_ReleaseGPUTransferBuffer, SDL_SubmitGPUCommandBufferAndAcquireFence, SDL_UnmapGPUTransferBuffer, SDL_UploadToGPUBuffer, SDL_WaitForGPUFences, SDL_GPU_BUFFERUSAGE_COMPUTE_STORAGE_READ, SDL_GPU_BUFFERUSAGE_COMPUTE_STORAGE_WRITE, SDL_GPU_SHADERFORMAT_DXIL, SDL_GPU_SHADERFORMAT_MSL, SDL_GPU_SHADERFORMAT_SPIRV, SDL_GPU_TRANSFERBUFFERUSAGE_DOWNLOAD, SDL_GPU_TRANSFERBUFFERUSAGE_UPLOAD}};

use crate::util;

pub struct ComputePipelineInfo<'a> {
    pub shader_name: &'a str,
    pub num_readwrite_storage_buffers: u32,
    pub num_readonly_storage_buffers: u32,
    pub num_uniform_buffers: u32,
    pub threadcount_x: u32,
    pub threadcount_y: u32,
    pub threadcount_z: u32
}

impl Default for ComputePipelineInfo<'_> {
    fn default() -> Self {
        ComputePipelineInfo {
            shader_name: "",
            num_readwrite_storage_buffers: 0,
            num_readonly_storage_buffers: 0,
            num_uniform_buffers: 0,
            threadcount_x: 64,
            threadcount_y: 1,
            threadcount_z: 1
        }
    }
}

pub fn create_gpu_device() -> Result<*mut SDL_GPUDevice, &'static str> {
    let device = unsafe { 
        SDL_CreateGPUDevice((ShaderFormat::SpirV | ShaderFormat::Dxil | ShaderFormat::Msl) as u32, false, std::ptr::null()) 
    };
    if device.is_null() {
        Err("Failed to create GPU device")
    } else {
        Ok(device)
    }
}

pub fn free_gpu_device(device: *mut SDL_GPUDevice) {
    unsafe { SDL_DestroyGPUDevice(device) };
}

pub fn create_compute_pipeline(device: *mut SDL_GPUDevice, info: &ComputePipelineInfo) -> Result<*mut SDL_GPUComputePipeline, &'static str> {
    // Choose parameters depending on available shader formats
    let formats = unsafe { SDL_GetGPUShaderFormats(device) };
    let chosen_format: SDL_GPUShaderFormat;
    let chosen_path: String;
    let chosen_entrypoint: &CStr;
    if (formats & SDL_GPU_SHADERFORMAT_SPIRV) == SDL_GPU_SHADERFORMAT_SPIRV {
        chosen_format = SDL_GPU_SHADERFORMAT_SPIRV;
        chosen_path = format!("./compiled_shaders/spirv/{}.spv", info.shader_name);
        chosen_entrypoint = CStr::from_bytes_with_nul(b"main\0").unwrap();
    } else if (formats & SDL_GPU_SHADERFORMAT_MSL) == SDL_GPU_SHADERFORMAT_MSL {
        chosen_format = SDL_GPU_SHADERFORMAT_MSL;
        chosen_path = format!("./compiled_shaders/msl/{}.msl", info.shader_name);
        chosen_entrypoint = CStr::from_bytes_with_nul(b"main0\0").unwrap();
    } else if (formats & SDL_GPU_SHADERFORMAT_DXIL) == SDL_GPU_SHADERFORMAT_DXIL {
        chosen_format = SDL_GPU_SHADERFORMAT_DXIL;
        chosen_path = format!("./compiled_shaders/dxil/{}.dxil", info.shader_name);
        chosen_entrypoint = CStr::from_bytes_with_nul(b"main\0").unwrap();
    } else {
        return Err("Unknown shader format");
    }
    
    // Load code
    let code = fs::read(util::get_exe_directory().join(chosen_path));
    if code.is_err() {
        return Err("Failed to read compiled shader file");
    }
    let code = code.unwrap();

    // Build pipeline
    let pipeline_create_info = SDL_GPUComputePipelineCreateInfo {
        format: chosen_format,
        code: code.as_ptr(),
        code_size: code.len(),
        entrypoint: chosen_entrypoint.as_ptr(),
        num_readwrite_storage_buffers: info.num_readwrite_storage_buffers,
        num_readonly_storage_buffers: info.num_readonly_storage_buffers,
        num_uniform_buffers: info.num_uniform_buffers,
        threadcount_x: info.threadcount_x,
        threadcount_y: info.threadcount_y,
        threadcount_z: info.threadcount_z,
        ..Default::default()
    };
    let compute_pipeline = unsafe { SDL_CreateGPUComputePipeline(device, &pipeline_create_info) };
    if compute_pipeline.is_null() {
        Err("Failed to create compute pipeline")
    } else {
        Ok(compute_pipeline)
    }
}

pub fn free_compute_pipeline(device: *mut SDL_GPUDevice, compute_pipeline: *mut SDL_GPUComputePipeline) {
    unsafe { SDL_ReleaseGPUComputePipeline(device, compute_pipeline) };
}

fn round_up_power_of_two(num: usize, multiple: usize) -> usize {
    debug_assert!(((multiple & (multiple - 1)) == 0));
    return (((num + multiple - 1)) as i64 & -(multiple as i64)) as usize;
}

pub fn begin_command_buffer(device: *mut SDL_GPUDevice) -> Result<*mut SDL_GPUCommandBuffer, &'static str> {
    let command_buffer = unsafe { SDL_AcquireGPUCommandBuffer(device) };
    if command_buffer.is_null() {
        Err("Failed to acquire GPU command buffer")
    } else {
        Ok(command_buffer)
    }
}

pub fn end_command_buffer_and_get_fence(command_buffer: *mut SDL_GPUCommandBuffer) -> Result<*mut SDL_GPUFence, &'static str> {
    let fence = unsafe { SDL_SubmitGPUCommandBufferAndAcquireFence(command_buffer) };
    if fence.is_null() {
        Err("Failed to submit command buffer and acquire fence")
    } else {
        Ok(fence)
    }
}

pub fn end_command_buffer_and_wait_for_fence(device: *mut SDL_GPUDevice, command_buffer: *mut SDL_GPUCommandBuffer) -> bool {
    let fence = end_command_buffer_and_get_fence(command_buffer);
    if fence.is_err() {
        return false;
    }
    wait_for_and_release_fences(device, &[fence.unwrap()])
}

pub fn wait_for_and_release_fences(device: *mut SDL_GPUDevice, fences: &[*mut SDL_GPUFence]) -> bool {
    let success = unsafe { SDL_WaitForGPUFences(device, true, &fences[0], fences.len() as u32) };
    for fence in fences {
        unsafe { SDL_ReleaseGPUFence(device, fence.clone()); }
    }
    success
}

pub struct GPUBufferInfo {
    buffer: *mut SDL_GPUBuffer,
    size: u32,
    rounded_size: u32
}

impl GPUBufferInfo {
    pub fn raw(&self) -> *mut SDL_GPUBuffer {
        self.buffer
    }
    pub fn size(&self) -> u32 {
        self.size
    }
    pub fn rounded_size(&self) -> u32 {
        self.rounded_size
    }
}

pub fn create_gpu_buffer(device: *mut SDL_GPUDevice, size: usize, allow_read: bool, allow_write: bool) -> Result<GPUBufferInfo, &'static str> {
    // Round up size for 16-byte alignment
    let rounded_size = round_up_power_of_two(size, 16);

    // Handle overflow gracefully
    if rounded_size > (u32::MAX as usize) {
        return Err("Buffer size is larger than maximum 32-bit integer");
    }

    // Get appropriate buffer usage flags
    let mut flags: u32 = 0;
    debug_assert!(allow_read || allow_write);
    if allow_read {
        flags |= SDL_GPU_BUFFERUSAGE_COMPUTE_STORAGE_READ;
    }
    if allow_write {
        flags |= SDL_GPU_BUFFERUSAGE_COMPUTE_STORAGE_WRITE;
    }
    
    // Create buffer
    let buffer_info = SDL_GPUBufferCreateInfo {
        usage: flags,
        size: rounded_size as u32,
        ..Default::default()
    };
    let buffer = unsafe { SDL_CreateGPUBuffer(device, &buffer_info) };
    if buffer.is_null() {
        Err("Failed to create GPU buffer")
    } else {
        Ok(GPUBufferInfo {
            buffer,
            size: size as u32,
            rounded_size: rounded_size as u32
        })
    }
}

pub fn free_gpu_buffer(device: *mut SDL_GPUDevice, info: &GPUBufferInfo) {
    unsafe { SDL_ReleaseGPUBuffer(device, info.buffer) };
}

pub fn begin_copy_pass(command_buffer: *mut SDL_GPUCommandBuffer) -> Result<*mut SDL_GPUCopyPass, &'static str> {
    let copy_pass = unsafe { SDL_BeginGPUCopyPass(command_buffer) };
    if copy_pass.is_null() {
        Err("Failed to begin copy pass")
    } else {
        Ok(copy_pass)
    }
}

pub fn end_copy_pass(copy_pass: *mut SDL_GPUCopyPass) {
    unsafe { SDL_EndGPUCopyPass(copy_pass) };
}

pub fn upload_to_gpu_buffer(device: *mut SDL_GPUDevice, copy_pass: *mut SDL_GPUCopyPass, buffer_info: &GPUBufferInfo, data: &[u8]) -> Result<(), &'static str> {
    if data.len() > (buffer_info.size as usize) {
        return Err("Too much data being uploaded to buffer");
    }

    // Create transfer buffer
    let transfer_buffer_info = SDL_GPUTransferBufferCreateInfo {
        size: buffer_info.rounded_size,
        usage: SDL_GPU_TRANSFERBUFFERUSAGE_UPLOAD,
        ..Default::default()
    };
    let transfer_buffer = unsafe { SDL_CreateGPUTransferBuffer(device, &transfer_buffer_info) };
    if transfer_buffer.is_null() {
        return Err("Failed to create transfer buffer");
    }

    // Map transfer buffer memory to application address space, so we can copy data to it
    let transfer_buffer_data: *mut c_void = unsafe { SDL_MapGPUTransferBuffer(device, transfer_buffer, false) };
    if transfer_buffer_data.is_null() {
        return Err("Failed to map GPU transfer buffer to application address space");
    }

    // Copy data to transfer buffer
    unsafe { ptr::copy_nonoverlapping(data.as_ptr(), transfer_buffer_data as *mut u8, data.len()) };

    // Unmap transfer buffer memory from application address space
    unsafe { SDL_UnmapGPUTransferBuffer(device, transfer_buffer) };

    // Perform actual upload as part of the provided copy pass
    let buffer_region = SDL_GPUBufferRegion {
        buffer: buffer_info.buffer,
        offset: 0,
        size: buffer_info.rounded_size
    };
    let transfer_buffer_location = SDL_GPUTransferBufferLocation {
        transfer_buffer,
        offset: 0
    };
    unsafe { SDL_UploadToGPUBuffer(copy_pass, &transfer_buffer_location, &buffer_region, false) };
    
    // Release the transfer buffer (will actually occur when it's safe to do so)
    unsafe { SDL_ReleaseGPUTransferBuffer(device, transfer_buffer) };

    Ok(())
}


pub fn upload_to_gpu_buffer_u32(device: *mut SDL_GPUDevice, copy_pass: *mut SDL_GPUCopyPass, buffer_info: &GPUBufferInfo, data: &[u32]) -> Result<(), &'static str> {
    if (data.len() * 4) > (buffer_info.size as usize) {
        return Err("Too much data being uploaded to buffer");
    }

    // Create transfer buffer
    let transfer_buffer_info = SDL_GPUTransferBufferCreateInfo {
        size: buffer_info.rounded_size,
        usage: SDL_GPU_TRANSFERBUFFERUSAGE_UPLOAD,
        ..Default::default()
    };
    let transfer_buffer = unsafe { SDL_CreateGPUTransferBuffer(device, &transfer_buffer_info) };
    if transfer_buffer.is_null() {
        return Err("Failed to create transfer buffer");
    }

    // Map transfer buffer memory to application address space, so we can copy data to it
    let transfer_buffer_data: *mut c_void = unsafe { SDL_MapGPUTransferBuffer(device, transfer_buffer, false) };
    if transfer_buffer_data.is_null() {
        return Err("Failed to map GPU transfer buffer to application address space");
    }

    // Copy data to transfer buffer
    unsafe { ptr::copy_nonoverlapping::<u32>(data.as_ptr(), transfer_buffer_data as *mut u32, data.len()) };

    // Unmap transfer buffer memory from application address space
    unsafe { SDL_UnmapGPUTransferBuffer(device, transfer_buffer) };

    // Perform actual upload as part of the provided copy pass
    let buffer_region = SDL_GPUBufferRegion {
        buffer: buffer_info.buffer,
        offset: 0,
        size: buffer_info.rounded_size
    };
    let transfer_buffer_location = SDL_GPUTransferBufferLocation {
        transfer_buffer,
        offset: 0
    };
    unsafe { SDL_UploadToGPUBuffer(copy_pass, &transfer_buffer_location, &buffer_region, false) };
    
    // Release the transfer buffer (will actually occur when it's safe to do so)
    unsafe { SDL_ReleaseGPUTransferBuffer(device, transfer_buffer) };

    Ok(())
}

pub fn queue_download_from_gpu_buffer(device: *mut SDL_GPUDevice, copy_pass: *mut SDL_GPUCopyPass, buffer_info: &GPUBufferInfo) -> Result<*mut SDL_GPUTransferBuffer, &'static str> {
    // Create transfer buffer
    let transfer_buffer_info = SDL_GPUTransferBufferCreateInfo {
        size: buffer_info.rounded_size,
        usage: SDL_GPU_TRANSFERBUFFERUSAGE_DOWNLOAD,
        ..Default::default()
    };
    let transfer_buffer = unsafe { SDL_CreateGPUTransferBuffer(device, &transfer_buffer_info) };
    if transfer_buffer.is_null() {
        return Err("Failed to create transfer buffer");
    }

    // Perform download as part of copy pass (only guaranteed to finish when the command buffer's fence is signaled)
    let buffer_region = SDL_GPUBufferRegion {
        buffer: buffer_info.buffer,
        offset: 0,
        size: buffer_info.rounded_size
    };
    let transfer_buffer_location = SDL_GPUTransferBufferLocation {
        transfer_buffer,
        offset: 0
    };
    unsafe { SDL_DownloadFromGPUBuffer(copy_pass, &buffer_region, &transfer_buffer_location) };

    Ok(transfer_buffer)
}

pub fn finish_download_from_gpu_buffer(device: *mut SDL_GPUDevice, buffer_info: &GPUBufferInfo, transfer_buffer: *mut SDL_GPUTransferBuffer, output_buffer: &mut [u8]) -> Result<(), &'static str> {
    // Validate output buffer size
    if output_buffer.len() > (buffer_info.size as usize) {
        return Err("Too much data being downloaded from buffer");
    }
    
    // Map transfer buffer memory to application address space, so we can copy data from it
    let transfer_buffer_data: *mut c_void = unsafe { SDL_MapGPUTransferBuffer(device, transfer_buffer, false) };
    if transfer_buffer_data.is_null() {
        return Err("Failed to map GPU transfer buffer to application address space");
    }

    // Copy data to output buffer
    unsafe { ptr::copy_nonoverlapping(transfer_buffer_data as *const u8, &mut output_buffer[0] as *mut u8, output_buffer.len()) };

    // Unmap transfer buffer memory from application address space
    unsafe { SDL_UnmapGPUTransferBuffer(device, transfer_buffer) };

    // Release the transfer buffer
    unsafe { SDL_ReleaseGPUTransferBuffer(device, transfer_buffer) };

    Ok(())
}

pub fn push_uniform_data(command_buffer: *mut SDL_GPUCommandBuffer, slot_index: u32, data: &[u8]) {
    unsafe { SDL_PushGPUComputeUniformData(command_buffer, slot_index, (&data[0] as *const u8) as *const c_void, data.len() as u32) };
}

pub fn perform_buffer_compute(command_buffer: *mut SDL_GPUCommandBuffer, compute_pipeline: *mut SDL_GPUComputePipeline,
                          writeable_buffer_storage: &[*mut SDL_GPUBuffer], all_buffer_storage_ordered: &[*mut SDL_GPUBuffer],
                          groupcount_x: u32, groupcount_y: u32, groupcount_z: u32) -> Result<(), &'static str> {
    // Create arrays for buffer bindings (and an empty array for texture bindings)
    let storage_texture_bindings: [SDL_GPUStorageTextureReadWriteBinding; 0] = [];
    let mut storage_buffer_bindings: Vec<SDL_GPUStorageBufferReadWriteBinding> = Vec::with_capacity(writeable_buffer_storage.len());
    for writeable_buffer in writeable_buffer_storage {
        storage_buffer_bindings.push(SDL_GPUStorageBufferReadWriteBinding {
            buffer: writeable_buffer.clone(),
            cycle: false,
            ..Default::default()
        });
    }

    // Begin compute pass
    let compute_pass = unsafe {
        SDL_BeginGPUComputePass(
            command_buffer,
            storage_texture_bindings.as_ptr().cast(),
            storage_texture_bindings.len() as u32,
            storage_buffer_bindings.as_ptr().cast(),
            storage_buffer_bindings.len() as u32,
        )
    };
    if compute_pass.is_null() {
        return Err("Failed to begin compute pass");
    }

    // Bind the compute pipeline for this pass
    unsafe { SDL_BindGPUComputePipeline(compute_pass, compute_pipeline) };
    
    // Bind the storage buffers as used by the compute pipeline
    unsafe { SDL_BindGPUComputeStorageBuffers(compute_pass, 0, &all_buffer_storage_ordered[0], all_buffer_storage_ordered.len() as u32) };

    // Actually perform dispatch
    unsafe { SDL_DispatchGPUCompute(compute_pass, groupcount_x, groupcount_y, groupcount_z) };

    // End compute pass
    unsafe { SDL_EndGPUComputePass(compute_pass) };

    Ok(())
}

#[derive(Clone, Copy)]
pub struct PointU32 {
    value: u32
}

impl PointU32 {
    pub fn new(x: i16, y: i16) -> Self {
        PointU32 { 
            value: (((x as u32) & 0xffff) << 16) | ((y as u32) & 0xffff)
        }
    }
    pub fn x(&self) -> i16 {
        ((self.value & 0xffff0000) >> 16) as i16
    }
    pub fn y(&self) -> i16 {
        (self.value & 0xffff) as i16
    }
}
