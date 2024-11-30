use std::path::PathBuf;
use std::{fs, process};
use dirs_next::config_dir;
use once_cell::sync::Lazy;
use serde::Deserialize;
use config::{Config, File, Environment};

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub(crate) debug : bool,
    pub(crate) check_interval: u64,
    pub(crate) double_check: u64,
    pub(crate) audio_system: Option<String>,
    pub(crate) stop_file: String,
}

pub(crate) static SETTINGS: Lazy<Settings> = Lazy::new(|| {
    get_settings().unwrap_or_else(|err| {
        eprintln!("Failed to load settings: {}", err);
        process::exit(1);
    })
});

pub static DEBUG_MODE: Lazy<bool> = Lazy::new(|| {
    SETTINGS.debug
});
pub fn debug_do<T, F, G>(debug_action: F, default_action: G) -> T
where
    F: FnOnce() -> T,
    G: FnOnce() -> T,
{
    if *DEBUG_MODE {
        debug_action()
    } else {
        default_action()
    }
}

fn get_config_path() -> PathBuf {
    // Get the user's config directory (cross-platform)
    let config_path = config_dir()
        .expect("Could not determine the configuration directory")
        .join("hypridle_audio_listener") // App-specific folder
        .join("config.toml"); // The config file

    // Create the parent directory if it doesn't exist
    if let Some(parent_dir) = config_path.parent() {
        if !parent_dir.exists() {
            fs::create_dir_all(parent_dir).expect("Could not create configuration directory");
        }
    }
    config_path
}

pub(crate) fn get_settings() -> Result<Settings, config::ConfigError>{
    let config_path = get_config_path();
    println!("Loading config from: {:?}", config_path);

    let settings = Config::builder()
        .set_default("debug", false)?
        .set_default("check_interval", 10)?
        .set_default("double_check", 0)?
        .set_default("audio_system", None::<String>)?
        .set_default("stop_file", "/tmp/hypridle_audio_listener_stop_file")?
        // Load from the config file in the user's ~/.config/myapp directory
        .add_source(File::from(config_path).required(false))
        // Override with environment variables
        .add_source(Environment::with_prefix("HYPRIDLE_AUDIO_LISTENER"))
        .build()?;
    let settings: Settings = settings.try_deserialize()?;
    Ok(settings)
}