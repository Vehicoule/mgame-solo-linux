//! Production-Grade M-Audio M-Game Solo Companion Software for Linux.

mod audio;
mod midi;
mod state;
mod ui;

use ui::app::MGameApp;

fn main() -> glib::ExitCode {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    log::info!("Starting M-Audio M-Game Solo Linux Control Suite...");
    
    let app = MGameApp::new();
    app.run()
}
