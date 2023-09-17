# scope-tui
A simple oscilloscope/vectorscope/spectroscope in your terminal

![scope-tui interface](https://cdn.alemi.dev/scope-tui-wide.png)

Currently only for Linux (with Pulseaudio)

See it in action [here](https://cdn.alemi.dev/scope-tui-oscilloscope-music.webm) with [Jerobeam Fenderson - Planets](https://youtu.be/XziuEdpVUe0) (oscilloscope music)

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
  -b, --buffer <SIZE>       Size of audio buffer, and width of scope [default: 8192]
  -r, --range <SIZE>        Max value, positive and negative, on amplitude scale [default: 20000]
      --scatter             Use vintage looking scatter mode instead of line mode
      --show-peaks          Show peaks for each channel as dots
      --tune <NOTE>         Tune buffer size to be in tune with given note (overrides buffer option)
      --channels <N>        Number of channels to open [default: 2]
      --sample-rate <HZ>    Sample rate to use [default: 44100]
      --server-buffer <N>   Pulseaudio server buffer size, in block number [default: 32]
      --triggering          Start drawing at first rising edge
      --threshold <VAL>     Threshold value for triggering [default: 0]
      --check-depth <SMPL>  Length of trigger check in samples [default: 1]
      --falling-edge        Trigger upon falling edge instead of rising
      --no-reference        Don't draw reference line
      --no-ui               Hide UI and only draw waveforms
      --no-braille          Don't use braille dots for drawing lines
  -h, --help                Print help information
  -V, --version             Print version information
```

The audio buffer size directly impacts resource usage, latency and refresh rate and its limits are given by the audio refresh rate. Larger buffers are slower but less resource intensive. A good starting value might be `8192` or tuning to the 0th octave.

To change audio buffer size, the PulseAudio client must be restarted. Because of this, such option is configurable only at startup.

## Controls
* Use `q` or `CTRL+C` to exit
* Use `s` to toggle scatter mode
* Use `h` to toggle interface
* Use `r` to toggle reference lines
* Use `<SPACE>` to pause and resume display
* Use `<LEFT>` and `<RIGHT>` to increase or decrease X range
* Use `<UP>` and `<DOWN>` to increase or decrease Y range
* Use `<ESC>` to revert view settings to defaults
* Use `<TAB>` to switch between modes:
  * **Oscilloscope**:
    * Use `t` to toggle triggered mode
    * Use `e` to switch edge-triggering mode (rise/falling)
    * Use `p` to toggle peaks display
    * Use `<PG-UP>` and `<PG-DOWN>` to increase or decrease trigger threshold
    * Use `-`/`_` and `=`/`+` to increase or decrease trigger debouncing
  * **Spectroscope**:
    * Use `<PG-UP>` and `<PG-DOWN>` to increase or decrease averaging count
  * **Vectorscope**:
* Combine increment/decrement commands with `<SHIFT>` to increase or decrease by x10
* Combine increment/decrement commands with `<CTRL>` to increase or decrease by x5
* Combine increment/decrement commands with `<ALT>` to increase or decrease by x 1/5

# About precision
While "scatter" plot mode is as precise as Pulseaudio and the terminal lets us be, "line" plot mode simply draws a straight line across points, meaning high frequencies don't get properly represented.

Latency is kept to a minimum thanks to small buffer and block sizes.

Sample rate can be freely specified but will ultimately be limited by source's actual sample rate.

Decrease/increase terminal font size to increase/decrease scope resolution.
