mod cli;
mod model;
mod gui;
mod theme;

use std::env;
use gui::PomimiApp;

fn main() -> iced::Result {
    let args: Vec<String> = env::args().collect();
    
    let run_cli = if args.len() > 1 {
        if args[1] == "--cli" {
            true
        } else {
            let arg = &args[1];
            // Simple heuristic to check if arg is time
            if arg.ends_with('m') || arg.ends_with('s') || arg.parse::<u64>().is_ok() {
                true
            } else {
                false
            }
        }
    } else {
        false
    };

    if run_cli {
        cli::run();
        Ok(())
    } else {
        iced::application(PomimiApp::title, PomimiApp::update, PomimiApp::view)
            .theme(PomimiApp::theme)
            .subscription(PomimiApp::subscription)
            .run_with(PomimiApp::new)
    }
}
