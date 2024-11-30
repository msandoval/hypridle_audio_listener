use std::cmp::PartialEq;
use std::{fmt, thread};
use std::process::Command;
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::Duration;
use once_cell::sync::Lazy;
use serde_json::Value;
use which::which;
use crate::config::{debug_do, DEBUG_MODE, SETTINGS};
#[derive(Debug)]
pub enum AudioSystem {
    PipeWire,
    PulseAudio,
    NotCompatible
}
#[derive(PartialEq, Clone, Debug)]
pub(crate) enum AudioStatus {
    Playing,
    NotPlaying
}
#[derive(Clone,Debug)]
pub enum AudioListenerError {
    CommandFailed(String),
    MissingCommand(String),
    InvalidOutput(String),
}

#[derive(PartialEq, Debug)]
pub enum MonitorStatus {
    MonitorOn,
    MonitorOff
}

impl fmt::Display for AudioListenerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AudioListenerError::CommandFailed(msg) => write!(f, "Command failed: {}", msg),
            AudioListenerError::MissingCommand(msg) => write!(f, "Command not found: {}", msg),
            AudioListenerError::InvalidOutput(msg) => write!(f, "Output invalid: {}", msg)
        }
    }
}
impl FromStr for AudioSystem {
    type Err = String; // Define the error type; here, a simple `String` is used.

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input.to_lowercase().as_str() {
            "pipewire" => Ok(AudioSystem::PipeWire),
            "pulseaudio" => Ok(AudioSystem::PulseAudio),
            _ => Err(format!("Invalid audio system: {}", input)),
        }
    }
}

static DEBUG_MONITOR_STATE: Lazy<Mutex<bool>> = Lazy::new(|| Mutex::new(true));
static DOUBLECHECK_ONCE: AtomicBool = AtomicBool::new(false);

fn is_audio_playing_pipewire() -> Result<AudioStatus,AudioListenerError> {
    if which::which("pw-dump").is_err() {
        return Err(AudioListenerError::MissingCommand("pw-dump not found on system".to_string()));
    }

    let output = Command::new("pw-dump")
        .output()
        .map_err(|_| AudioListenerError::CommandFailed("Failed to execute pw-dump".to_string()))?;

    let data = String::from_utf8_lossy(&output.stdout);

    match serde_json::from_str::<Value>(&data) {
        Ok(parsed) => {
            if let Some(nodes) = parsed.as_array() {
                match nodes.iter().any(|node| {
                    node.get("info")
                        .and_then(|info| info.get("state"))
                        .map_or(false, |state| state == "active" )
                }) {
                    true => { Ok(AudioStatus::Playing) }
                    false => { Ok(AudioStatus::NotPlaying) }
                }
            } else {
                Ok(AudioStatus::NotPlaying)
            }
        }
        Err(e) => Err(AudioListenerError::InvalidOutput(format!("Failed to parse pw-dump output: {}", e).to_string())),
    }
}

fn is_audio_playing_pulseaudio() -> Result<AudioStatus, AudioListenerError> {
    // Check if `pactl` is available
    if which::which("pactl").is_err() {
        return Err(AudioListenerError::MissingCommand("pactl not found on system".to_string()));
    }

    // Execute `pactl list sink-inputs`
    let output = Command::new("pactl")
        .args(["list", "sink-inputs"])
        .output()
        .map_err(|_| AudioListenerError::CommandFailed("Failed to execute pactl".to_string()))?;

    let data = String::from_utf8_lossy(&output.stdout);

    let is_playing = data.lines().any(|line| {
        if line.contains("State:") {
            // Check if "RUNNING" is present on the same lines
            line.contains("RUNNING")
        } else {
            false
        }
    });

    Ok(if is_playing {
        AudioStatus::Playing
    } else {
        AudioStatus::NotPlaying
    })
}

fn detect_audio_system() -> AudioSystem {
    if which("pw-dump").is_ok() {
        AudioSystem::PipeWire
    } else if which("pactl").is_ok() {
        AudioSystem::PulseAudio
    } else {
        AudioSystem::NotCompatible // Neither PipeWire nor PulseAudio found
    }
}

pub(crate) fn get_audio_status() -> Result<AudioStatus, AudioListenerError> {
    let selected_audio_system = match &SETTINGS.audio_system {
        None => None,
        Some(val) => val.parse::<AudioSystem>().ok()
    };
    let audio_system = selected_audio_system.unwrap_or_else(|| { detect_audio_system() });
    let mut status = match audio_system {
        AudioSystem::PipeWire => is_audio_playing_pipewire(),
        AudioSystem::PulseAudio => is_audio_playing_pulseaudio(),
        AudioSystem::NotCompatible => Err(AudioListenerError::MissingCommand(
            "Neither PipeWire nor PulseAudio found on system".to_string(),
        )),
    };
    let require_double_check = SETTINGS.double_check;
    if require_double_check != 0
        && !DOUBLECHECK_ONCE.swap(true, Ordering::SeqCst)
        && status.as_ref().ok() == Some(&AudioStatus::NotPlaying)
    {
        thread::sleep(Duration::from_secs(require_double_check));
        status = get_audio_status();
    }
    DOUBLECHECK_ONCE.store(false, Ordering::SeqCst);
    status
}

pub(crate) fn turn_off_monitors() -> MonitorStatus {
    // Get monitor information
    let output = Command::new("hyprctl")
        .args(["monitors"])
        .output()
        .expect("Failed to get monitor state");

    let data = String::from_utf8_lossy(&output.stdout);

    // Check if any monitor has DPMS "on". if in debug mode, do debug things
    let monitors_on = if *DEBUG_MODE {
        let mut state = DEBUG_MONITOR_STATE.lock().unwrap();
        state.then(|| {
            *state = false;
            true
        }).unwrap_or(false)
    } else {
        data.lines().any(|line| line.contains("dpmsStatus: 1"))
    };

    if monitors_on {
        println!("Turning off monitors...");
        debug_do(
            || { println!("DEBUG ON: Bypassing..") },
            || {
                let _ = Command::new("hyprctl")
                    .args(["dispatch", "dpms", "off"])
                    .output()
                    .expect("Failed to turn off monitors");
        });
    } else {
        if *DEBUG_MODE {
            println!("Monitors are already off. Skipping command.");
        }
    }
    MonitorStatus::MonitorOff
}

pub(crate) fn turn_on_monitors() -> MonitorStatus {
    println!("Turning monitors on...");
    if *DEBUG_MODE {
        let mut state = DEBUG_MONITOR_STATE.lock().unwrap();
        if !*state {
            *state = true;
        }
    } else {
        if let Err(e) = Command::new("hyprctl")
            .args(["dispatch", "dpms", "on"])
            .spawn()
            .expect("Failed to spawn hyprctl command")
            .wait()
        {
            eprintln!("Error turning monitors on: {}", e);
        }
    }
    MonitorStatus::MonitorOn
}

#[cfg(test)]
mod tests {
    use std::thread::sleep;
    use super::*;

    // Set debug = 1 in .config/hypridle_audio_listener/config.toml to test in debug mode
    #[test]
    fn verify_monitor_off() {
        assert_eq!(turn_off_monitors(), MonitorStatus::MonitorOff);
    }

    #[test]
    fn verify_monitor_on() {
        sleep(Duration::from_secs(3));
        assert_eq!(turn_on_monitors(), MonitorStatus::MonitorOn);
    }

    #[test]
    fn verify_audio_status() {
        let result = get_audio_status();
        assert!(matches!(result,Ok(_)), "Unexpected error: {:?}", result);
    }
}