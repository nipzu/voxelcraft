#![windows_subsystem = "console"]

use std::fs::File;

use winit::{
    dpi::PhysicalSize,
    event_loop::EventLoop,
    window::Window,
};

use simplelog::{
    ColorChoice, CombinedLogger, ConfigBuilder, LevelFilter, TermLogger, TerminalMode, WriteLogger,
};

mod program;
use program::Program;

fn main() {
    init_logging();

    let icon = winit::window::Icon::from_rgba([0, 150, 0, 255].repeat(16 * 16), 16, 16);
    let event_loop = EventLoop::new().expect("could not create event loop");
    let window_attrs = Window::default_attributes()
        .with_title("Voxelcraft 0.0.1")
        .with_window_icon(icon.ok())
        // .with_inner_size(PhysicalSize::new(640_u32, 360))
        .with_inner_size(PhysicalSize::new(1280_u32, 720))
        // .with_inner_size(PhysicalSize::new(1920_u32, 1080))
        // .with_inner_size(PhysicalSize::new(2560_u32, 1440))
        .with_min_inner_size(PhysicalSize::new(1_u32, 1))
        .with_transparent(false)
        .with_fullscreen(
            None,
            // Some(winit::window::Fullscreen::Borderless(None)),
        )
        .with_resizable(false);
    let window = event_loop
        .create_window(window_attrs)
        .expect("unable to create window");

    let program = Program::new(&window);


    log::info!("running program");
    program.run(event_loop);
}

fn init_logging() {
    let config = ConfigBuilder::new()
        //.set_time_format_custom("%T%.6f")
        //.set_time_to_local(true)
        .build();

    if let Ok(file) = File::create("log.txt") {
        CombinedLogger::init(vec![
            TermLogger::new(
                LevelFilter::Info,
                config.clone(),
                TerminalMode::Mixed,
                ColorChoice::Auto,
            ),
            WriteLogger::new(LevelFilter::Info, config, file),
        ])
        .unwrap();
    } else {
        TermLogger::init(
            LevelFilter::Warn,
            config,
            TerminalMode::Mixed,
            ColorChoice::Auto,
        )
        .unwrap();

        log::warn!(
            "unable to create log file, warnings and errors will still be displayed in the console"
        );
    }

    log::info!("logging started")
}
