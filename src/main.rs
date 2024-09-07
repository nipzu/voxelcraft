#![windows_subsystem = "console"]

use std::fs::File;

use winit::event_loop::EventLoop;

use simplelog::{
    ColorChoice, CombinedLogger, ConfigBuilder, LevelFilter, TermLogger, TerminalMode, WriteLogger,
};

mod program;
use program::Program;

fn main() {
    init_logging();

    let event_loop = EventLoop::new().expect("could not create event loop");

    let mut program = Program::new();

    log::info!("running program");

    if let Err(e) = event_loop.run_app(&mut program) {
        panic!("event loop error: {e:?}");
    }
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
