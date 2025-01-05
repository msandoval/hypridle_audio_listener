# hypridle_audio_listener
A helper tool for hypridle to turn off monitors only if audio is not playing

## About

This tool was created for hypridle users who want to set up a listener to turn off their monitors but only if music isn't playing.

## Requirements
You will need to have a working Pipewire or PulseAudio with either ``pactl`` or ``pw-dump`` installed.
## Install
### Arch/CachyOS
```ini
makepkg -si
```
## How to use
Below is an example ~/.config/hypr/hypridle.conf file:
```ini
general {
	lock_cmd = pidof hyprlock || hyprlock
}
listener {
	timeout = 510  # 8m 30s
	on-timeout = loginctl lock-session
}
listener {
    timeout = 630  # 10m 30s
    on-timeout = /usr/bin/hypridle_audio_listener start  # screen off when timeout has passed
    on-resume = /usr/bin/hypridle_audio_listener stop  # screen on when activity is detected after timeout has fired.
}
```
You can set the hypridle listener time to the time you wish the screen to go off if idle. hypridle_audio_listener will then check to see if audio is playing and if not, it will send the ``hyprctl dispatch dpms off`` command. Once your mouse or keyboard has triggered the ``on-resume`` event, the hypridle_audio_listener will reset and send the ``hyprctl dispatch dpms on`` command.  
