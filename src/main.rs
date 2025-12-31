use std::{sync::{atomic::{AtomicBool, Ordering}, mpsc::{self, Receiver, Sender}, Arc, Mutex}, thread};
use config::Config;
use sdl3::{render::{Canvas, TextureCreator}, video::{Window, WindowContext}, Sdl};
use server::{MessageToSend, ScreenshotData};
use text_rendering::Font;

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

#[derive(Clone, Copy)]
pub enum SubProgram {
    None,
    ProgramSelector,
    DustManip,
    NamingSeedSearch,
    DogiManip,
    Error,
    RNGOverride
}

fn main() {
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
    let window = video_subsystem.window("Dust Manipulator", default_width, default_height)
        .position_centered()
        .resizable()
        .hidden()
        .build()
        .expect("Failed to create window");
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

    // Close server
    server_end_signal.store(true, Ordering::Relaxed);
    server_join_handle.join().expect("Failed to join server thread");

    println!("Finished!");
}
