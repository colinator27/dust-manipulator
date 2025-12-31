#![cfg_attr(
  all(
    target_os = "windows",
    not(debug_assertions),
    not(feature = "console")
  ),
  windows_subsystem = "windows"
)]

use std::{ffi::CString, panic, ptr, sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}, mpsc::{self, Receiver, Sender}}, thread};
use config::Config;
use sdl3::{render::{Canvas, TextureCreator}, video::{Window, WindowContext}, Sdl};
use sdl3_sys::{init::SDL_IsMainThread, messagebox::{SDL_MESSAGEBOX_ERROR, SDL_ShowSimpleMessageBox}, video::SDL_Window};
use server::{MessageToSend, ScreenshotData};
use text_rendering::Font;

use crate::{program_common::SubProgram, windowing::window_set_always_on_top};

extern crate sdl3;

mod rng;
mod server;
mod text_rendering;
mod dust;
mod snowballs;
mod compute_shaders;
mod compute_dust_search;
mod compute_naming_search;
mod compute_snowball_search;
mod frame_images;
mod windowing;
mod encounter_data;
mod manip_data;
mod util;
mod program_selector;
mod program_dust_manip;
mod program_naming_seed_search;
mod program_dogi_manip;
mod program_error;
mod program_rng_override;
mod program_common;
mod config;

// Context for specific (speed)runs
pub struct RunContext {
    // Whether RNG seed/min. position have been found
    rng_found: bool,

    // Found RNG seed and minimum possible current RNG position
    rng_seed: u32,
    min_rng_position: usize,
}
impl RunContext {
    pub fn new() -> Self {
        Self {
            rng_found: false,
            rng_seed: 0,
            min_rng_position: 0
        }
    }
    pub fn rng_seed(&self) -> Option<u32> {
        if self.rng_found { Some(self.rng_seed) } else { None }
    }
    pub fn min_rng_position(&self) -> Option<usize> {
        if self.rng_found { Some(self.min_rng_position) } else { None }
    }
    pub fn set_rng(&mut self, rng_seed: u32, min_rng_position: usize) {
        self.rng_found = true;
        self.rng_seed = rng_seed;
        self.min_rng_position = min_rng_position;
    }
    pub fn set_min_rng_position(&mut self, min_rng_position: usize) {
        self.min_rng_position = min_rng_position;
    }
    pub fn reset(&mut self) {
        self.rng_found = false;
        self.rng_seed = 0;
        self.min_rng_position = 0;
    }
    pub fn rng_found(&self) -> bool {
        self.rng_found
    }
}

// Context shared across all tools
pub struct MainContext<'a> {
    // Main program config
    pub config: &'a Config,

    // Window/rendering
    pub sdl_context: &'a Sdl,
    pub font: &'a Font<'a>,
    pub canvas: &'a mut Canvas<Window>,
    pub texture_creator: &'a TextureCreator<WindowContext>,
    pub window_shown: bool,

    // Panic handling
    pub panic_occurred: Arc<AtomicBool>,

    // Server communication
    pub hotkey_receiver: &'a Receiver<u32>,
    pub message_to_send_sender: &'a Sender<MessageToSend>,
    pub screenshot_data: Arc<Mutex<Vec<ScreenshotData>>>,
    pub server_connected: Arc<AtomicBool>,

    // Error message for when in the error sub-program
    pub error_message: &'static str,

    // Main run context
    pub run_context: RunContext
}
impl MainContext<'_> {
    pub fn ignore_server_messages(&mut self) {
        // Ignore all incoming hotkeys
        for _ in self.hotkey_receiver.try_iter() {
        }

        // Ignore all incoming screenshots
        let mut screenshot_data = self.screenshot_data.lock().unwrap();
        screenshot_data.clear();
        drop(screenshot_data);
    }
}

struct PanicParameters {
    error_window: *mut SDL_Window,
    thread_error_payload_str: Option<String>,
    dump_path_str: String
}
unsafe impl Send for PanicParameters {}

fn attempt_show_thread_panic(panic_parameters: &PanicParameters) {
    let Ok(title) = CString::new("Error") else { return };
    let message = if let Some(payload_str) = &panic_parameters.thread_error_payload_str {
        CString::new(format!("A fatal error has occurred in the tool. The error message is below:\n\n{}\n\nFor more details, a report file was placed at: {}", payload_str, panic_parameters.dump_path_str))
    } else {
        CString::new(format!("A fatal error has occurred in the tool. No error message was able to be retrieved.\n\nFor more details, a report file was placed at: {}", panic_parameters.dump_path_str))
    };
    let Ok(message) = message else { return };
    unsafe { SDL_ShowSimpleMessageBox(SDL_MESSAGEBOX_ERROR, title.as_ptr(), message.as_ptr(), panic_parameters.error_window) };
}

fn remove_info_from_path(path: String) -> String {
    if cfg!(windows) {
        // Hide username for privacy reasons (at least in obvious cases), just in case it's needed for some reason...
        if path.len() < 4 {
            return path;
        }
        let Some(first_char) = path.chars().nth(0) else { return path };
        if first_char < 'A' || first_char > 'Z' {
            return path;
        }
        if path.chars().nth(1) != Some(':') {
            return path;
        }
        if !path[1..].starts_with(":\\Users\\") {
            return path;
        }
        let mut end_username_index = 0;
        for c in path.char_indices().skip("C:\\Users\\".len()) {
            if c.1 == '\\' {
                end_username_index = c.0;
                break;
            }
        }
        if end_username_index == 0 {
            return path;
        }
        path[0..("C:\\Users\\".len())].to_string() + "<username>" + &path[end_username_index..]
    } else {
        path
    }
}

fn main() {
    // Handle panics somewhat gracefully
    let existing_hook = panic::take_hook();
    let panic_parameters = Arc::new(Mutex::new(PanicParameters {
        error_window: ptr::null_mut(),
        thread_error_payload_str: None,
        dump_path_str: "(failed to get file path)".to_string()
    }));
    let panic_occurred = Arc::new(AtomicBool::new(false));
    let panic_parameters_callback = panic_parameters.clone();
    let panic_occurred_callback = panic_occurred.clone();
    panic::set_hook(Box::new(move |info| {
        // Dump information to temp file
        let dump_path = human_panic::handle_dump(&human_panic::Metadata::new("Dust Manipulator", env!("CARGO_PKG_VERSION")), info);
        let mut dump_path_str: String = "(failed to write file)".to_string();
        if let Some(dump_path) = dump_path {
            dump_path_str = dump_path.to_str().unwrap_or("(failed to get file path)").to_string();
        }
        dump_path_str = remove_info_from_path(dump_path_str);

        // Show error message window, if possible (and if none shown already)
        if unsafe { SDL_IsMainThread() } {
            if !panic_occurred_callback.load(Ordering::Relaxed) {
                let Ok(title) = CString::new("Error") else { return };
                let message = if let Some(payload_str) = info.payload_as_str() {
                    CString::new(format!("A fatal error has occurred in the tool. The error message is below:\n\n{}\n\nFor more details, a report file was placed at: {}", payload_str, dump_path_str))
                } else {
                    CString::new(format!("A fatal error has occurred in the tool. No error message was able to be retrieved.\n\nFor more details, a report file was placed at: {}", dump_path_str))
                };
                let Ok(message) = message else { return };
                let Ok(params) = panic_parameters_callback.lock() else { return };
                unsafe { SDL_ShowSimpleMessageBox(SDL_MESSAGEBOX_ERROR, title.as_ptr(), message.as_ptr(), params.error_window) };
            }
        } else if let Ok(mut params) = panic_parameters_callback.lock() {
            if let Some(payload_str) = info.payload_as_str() {
                params.thread_error_payload_str = Some(payload_str.to_string());
            }
            params.dump_path_str = dump_path_str;
        }

        // Mark that a panic has occurred (particularly if this happened on a thread)
        panic_occurred_callback.store(true, Ordering::Relaxed);

        // Call pre-existing hook
        existing_hook(info);
    }));

    // Load config
    let config = Config::read().expect("Failed to load config");

    // Start server
    let server_config = config.clone();
    let server_end_signal = Arc::new(AtomicBool::new(false));
    let server_end_signal_thread = server_end_signal.clone();
    let server_connected = Arc::new(AtomicBool::new(false));
    let server_connected_thread = server_connected.clone();
    let screenshot_data: Arc<Mutex<Vec<ScreenshotData>>> = Arc::new(Mutex::new(Vec::with_capacity(100)));
    let screenshot_data_thread = screenshot_data.clone();
    let (hotkey_sender, hotkey_receiver) = mpsc::channel::<u32>();
    let (message_to_send_sender, message_to_send_receiver) = mpsc::channel::<MessageToSend>();
    let server_join_handle = thread::spawn(move || {
        server::run_server(
            &server_config,
            Arc::clone(&server_end_signal_thread), 
            Arc::clone(&server_connected_thread),
            Arc::clone(&screenshot_data_thread),
            hotkey_sender,
            message_to_send_receiver);
    });

    // Initialize SDL and its video subsystem
    let sdl_context = sdl3::init().expect("Failed to initialize SDL");
    let video_subsystem = sdl_context.video().expect("Failed to get SDL video subsystem");

    // Load main font
    let ttf_context = text_rendering::TTFContext::new().expect("Failed to create TTF context");
    let font = ttf_context.load_font("./assets/8bitoperator_jve.ttf", 16.0).expect("Failed to load main font");

    // Determine default window width/height
    let primary_display_bounds = video_subsystem.get_primary_display().expect("Failed to get primary display").get_bounds().expect("Failed to get display bounds");
    let mut default_width: u32 = 640;
    let mut default_height: u32 = 480;
    for i in 2..6 {
        let attempt_width: u32 = 640 * i;
        let attempt_height: u32 = 480 * i;
        if (primary_display_bounds.width() / 2) > attempt_width && (primary_display_bounds.height() / 2) > attempt_height {
            default_width = attempt_width;
            default_height = attempt_height;
        } else {
            break;
        }
    }

    // Start window and graphics for GUI
    let mut window = video_subsystem.window("Dust Manipulator", default_width, default_height)
        .position_centered()
        .resizable()
        .hidden()
        .build()
        .expect("Failed to create window");
    if let Ok(mut panic_parameters) = panic_parameters.lock() {
        panic_parameters.error_window = window.raw();
    }
    if config.window_always_on_top {
        if !window_set_always_on_top(&mut window, true) {
            println!("Failed to set window to be always on top");
        }
    }
    if config.window_opacity != 1.0 {
        if let Err(e) = window.set_opacity(config.window_opacity) {
            println!("Failed to set window opacity: {}", e);
        }
    }
    let mut canvas = window.into_canvas();
    let texture_creator = canvas.texture_creator();

    // Create main context
    let mut main_context = MainContext {
        config: &config,
        sdl_context: &sdl_context,
        font: &font,
        canvas: &mut canvas,
        texture_creator: &texture_creator,
        window_shown: false,
        panic_occurred,
        hotkey_receiver: &hotkey_receiver,
        message_to_send_sender: &message_to_send_sender,
        server_connected,
        screenshot_data,
        error_message: "",
        run_context: RunContext::new()
    };

    // Run sub-programs
    let mut program = SubProgram::ProgramSelector;
    'running: loop {
        program = match program {
            SubProgram::None => break 'running,
            SubProgram::ProgramSelector => program_selector::run(&mut main_context),
            SubProgram::DustManip => program_dust_manip::run(&mut main_context),
            SubProgram::NamingSeedSearch => program_naming_seed_search::run(&mut main_context),
            SubProgram::DogiManip => program_dogi_manip::run(&mut main_context),
            SubProgram::Error => program_error::run(&mut main_context),
            SubProgram::RNGOverride => program_rng_override::run(&mut main_context)
        }
    }

    // Show message if a thread panicked
    let mut shown_thread_panic = false;
    if let Ok(panic_parameters) = panic_parameters.lock() {
        if main_context.panic_occurred.load(Ordering::Relaxed) {
            attempt_show_thread_panic(&panic_parameters);
            shown_thread_panic = true;
        }
    }

    // Close server
    server_end_signal.store(true, Ordering::Relaxed);
    server_join_handle.join().expect("Failed to join server thread");

    // Unregister window from panic handling
    if let Ok(mut panic_parameters) = panic_parameters.lock() {
        // But... if a thread panicked at the last moment, show its error too... (if not already shown)
        if !shown_thread_panic && main_context.panic_occurred.load(Ordering::Relaxed) {
            attempt_show_thread_panic(&panic_parameters);
        }
        panic_parameters.error_window = ptr::null_mut();
    }

    println!("Finished!");
}
