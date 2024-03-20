# scope-tui
A simple oscilloscope/vectorscope/spectroscope for your terminal

![scope-tui interface](https://cdn.alemi.dev/scope-tui-wide.png)

[See it in action here](https://cdn.alemi.dev/scope-tui-oscilloscope-music.webm) with [Planets](https://youtu.be/XziuEdpVUe0) (oscilloscope music by Jerobeam Fenderson)

## Why
I really love [cava](https://github.com/karlstav/cava). It provides a crude but pleasant frequency plot for your music: just the bare minimum to see leads solos and basslines.
I wanted to also be able to see waveforms, but to my knowledge nothing is available. There is some soundcard oscilloscope software available, but the graphical GUI is usually dated and breaks the magic.
I thus decided to solve this very critical issue with my own hands! And over a night of tinkering with pulseaudio (via [libpulse-simple-binding](https://crates.io/crates/libpulse-simple-binding)) and some TUI graphics (via [tui-rs](https://github.com/fdehau/tui-rs)), 
the first version of `scope-tui` was developed, with very minimal settings given from command line, but a bonus vectorscope mode baked in.

# Installation
Currently no binaries or packages are available and you must compile this yourself.

If you don't have the rust toolchain already installed, get it with [rustup](https://rustup.rs/)

Once you have `rustc` and `cargo`, just use `cargo install`:
```bash
# either stable releases from crates.io
$ cargo install scope-tui

# or dev builds from my source repository
$ CARGO_NET_GIT_FETCH_WITH_CLI=true cargo install --git https://git.alemi.dev/scope-tui.git
```
_(note that my git server doesn't support smart http clones, setting fetch-with-cli allows dumb http clones)_

The resulting binary will be under `$HOME./cargo/bin`. Either add such folder to your `$PATH` or copy the resulting binary somewhere in your `$PATH`.

## Sources
A very crude file source is always available, which can be a named pipe. While this allows connecting `scope-tui` to a lot of things, it's not super convenient, and more specialized sources should be used when available.

Currently only the PulseAudio source on Linux has been implemented, but more are planned for the future thanks to the modular sources structure.

Enable sources by passing the respective feature flags while compiling: `--features=pulseaudio,...`. Disable default features with `--no-default-features`. 
 * `pulseaudio` : pulseaudio implementation with LibPulse Simple bindings **(enabled by default)**


# Usage
```
$ scope-tui [OPTIONS] <COMMAND>

Commands:
  pulse  use PulseAudio Simple api to read data from an audio sink
  file   use a file from filesystem and read its content
  help   Print this message or the help of the given subcommand(s)

Options:
      --channels <N>      number of channels to open [default: 2]
      --tune <NOTE>       tune buffer size to be in tune with given note (overrides buffer option)
  -b, --buffer <SIZE>     size of audio buffer, and width of scope [default: 8192]
      --sample-rate <HZ>  sample rate to use [default: 44100]
  -r, --range <SIZE>      max value, positive and negative, on amplitude scale [default: 20000]
      --scatter           use vintage looking scatter mode instead of line mode
      --no-reference      don't draw reference line
      --no-ui             hide UI and only draw waveforms
      --no-braille        don't use braille dots for drawing lines
  -h, --help              Print help information
  -V, --version           Print version information
```

The audio buffer size directly impacts resource usage, latency and refresh rate and its limits are given by the audio refresh rate. Larger buffers are slower but less resource intensive. A good starting value might be `8192` or tuning to the 0th octave.

To change audio buffer size, the PulseAudio client must be restarted. Because of this, such option is configurable only at startup.

## Docker Compose

Included is a simple `Dockerfile` to build the project, and an accompanying `docker-compose.yml` to orchestrate starting the container with the proper volume mounts and environment variables.

Also included is a helper shell script to launch the container, attach to the container instance, and destroy the container on exiting.

```sh
docker compose up -d
docker attach scope_tui
docker compose down

# OR

./docker_start.sh
```

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

## About precision
While "scatter" plot mode is as precise as the samples are and the terminal lets us be, "line" plot mode simply draws a straight line across points, meaning high frequencies don't get properly represented.

Latency is kept to a minimum thanks to small buffer and block sizes.

Sample rate can be freely specified but will ultimately be limited by source's actual sample rate.

Decrease/increase terminal font size to increase/decrease scope resolution.

# Development
Any help is appreciated, feel free to contact me if you want to contribuite.

Some features I plan to work on and would like to add:
 * [x] Oscilloscope
 * [x] Vectorscope
 * [x] Linux audio source
 * [x] Simple controls
 * [x] Simple triggering
 * [x] Multiple channels
 * [x] Spectroscope
 * [x] File source
 * [x] Dockerize
 * [ ] Mac audio sources
 * [ ] Windows audio sources
 * [ ] Improve file audio source
 * [ ] Network sources
 * [ ] GUI frontend
 * [ ] Serial sources
 * [ ] USB sources
 * [ ] SDR sources
