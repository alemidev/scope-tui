# scope-tui
A simple oscilloscope/vectorscope in your terminal

![scope-tui interface](https://cdn.alemi.dev/scope-tui.png)

Currently only for Linux (with Pulseaudio)

## Why
I really love [cava](https://github.com/karlstav/cava). It provides a crude but pleasant frequency plot for your music: just the bare minimum to see leads solos and basslines.
I wanted to also be able to see waveforms, but to my knowledge nothing is available. There is some soundcard oscilloscope software available, but the graphical GUI is usually dated and breaks the magic.
I thus decided to solve this very critical issue with my own hands! And over a night of tinkering with pulseaudio (via [libpulse-simple-binding](https://crates.io/crates/libpulse-simple-binding)) and some TUI graphics (via [tui-rs](https://github.com/fdehau/tui-rs)), 
the first version of `scope-tui` was developed, with very minimal settings given from command line, but a bonus vectorscope mode baked in.

# Usage
```
$ scope-tui [OPTIONS] [DEVICE]

Arguments:
  [DEVICE]  Audio device to attach to

Options:
  -b, --buffer <BUFFER>  Size of audio buffer, and width of scope [default: 8192]
  -r, --range <RANGE>    Max value, positive and negative, on amplitude scale [default: 20000]
      --no-reference     Don't draw reference line
      --no-braille       Don't use braille dots for drawing lines
      --scatter          Use vintage looking scatter mode instead of line mode
      --vectorscope      Combine left and right channels into vectorscope view
  -h, --help             Print help information
  -V, --version          Print version information
```

The audio buffer size directly impacts resource usage, latency and refresh rate and its limits are given by the audio refresh rate. Larger buffers are slower but less resource intensive. A good starting value might be `8192`

To change audio buffer size, the PulseAudio client must be restarted. Because of this, such option is configurable only at startup.

## Controls
* Use `q` or `CTRL+C` to exit
* Use `<SPACE>` to pause and resume display
* Use `-` and `=` to decrease or increase range (`_` and `+` for smaller steps)
* Use `v` to toggle vectorscope mode
* Use `s` to toggle scatter mode
* Use `h` to toggle interface
* Decrease/increase terminal font size to increase/decrease scope resolution.
