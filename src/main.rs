// #![windows_subsystem = "console"]

use std::fs::File;

use simplelog::{
    ColorChoice, CombinedLogger, ConfigBuilder, LevelFilter, TermLogger, TerminalMode, WriteLogger,
};

mod program;
use program::Program;

fn main() {
    init_logging();

    let (event_loop, program) = Program::new();
    log::info!("running program");
    program.run(event_loop);
}

fn init_logging() {
    let config = ConfigBuilder::new()
        .set_time_format_str("%T%.6f")
        .set_time_to_local(true)
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
