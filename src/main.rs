mod audiosys;
mod config;

use std::{fs, process};
use std::path::Path;
use tokio::time::{sleep, Duration};
use audiosys::{get_audio_status, turn_off_monitors, turn_on_monitors, AudioStatus};
use clap_derive::{Subcommand,Parser};
use clap::Parser;
use crate::audiosys::MonitorStatus;
use crate::config::SETTINGS;

/// hypridle_audio_listener: hypridle monitor audio utility
///
/// A tool that is used with hypridle to turn off monitors when audio is not playing.
///
/// You would use this in place of directly using 'hyprctl dispatch dpms'.
/// This tools can start monitoring audio to determine whether to turn off monitors if no audio is
/// playing, or stop the process and restore monitor power.

#[derive(Parser)]
#[command(name = "hypridle_audio_listener", version = "0.1.0", author = "Manuel Sandoval", version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start monitoring audio and turn off monitors if no audio is playing
    Start,
    /// Stop monitoring audio and turn monitors back on
    Stop,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Start => {
            let check_interval = &SETTINGS.check_interval;
            let stop_file = &SETTINGS.stop_file;
            // Remove any stale stop signal file
            if Path::new(stop_file).exists() {
                println!("Stale stop signal detected. Removing it...");
                fs::remove_file(stop_file).expect("Failed to remove stop signal file");
            }
            loop {
                if Path::new(stop_file).exists() {
                    fs::remove_file(stop_file).expect("Failed to remove stop signal file");
                    process::exit(0)
                }
                match get_audio_status() {
                    Ok(AudioStatus::NotPlaying) => turn_off_monitors(),
                    Err(e) => { println!("{:?}", e); break }
                    _ => MonitorStatus::MonitorOn
                };
                sleep(Duration::from_secs(*check_interval)).await;
            }
        }
        Commands::Stop => {
            let stop_file = &SETTINGS.stop_file;
            println!("Sending stop signal...");
            fs::File::create(stop_file).expect("Failed to create stop signal file");
            turn_on_monitors();
        }
    }
}








