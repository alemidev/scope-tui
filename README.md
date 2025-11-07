# scope-tui
[![Actions Status](https://github.com/alemidev/scope-tui/actions/workflows/test.yml/badge.svg?branch=dev)](https://github.com/alemidev/scope-tui/actions/workflows/test.yml)
[![Actions Status](https://github.com/alemidev/scope-tui/actions/workflows/release.yml/badge.svg)](https://github.com/alemidev/scope-tui/actions/workflows/release.yml)
[![Crates.io Version](https://img.shields.io/crates/v/scope-tui)](https://crates.io/crates/scope-tui)
[![Crates.io Downloads (latest version)](https://img.shields.io/crates/dv/scope-tui)](https://crates.io/crates/scope-tui)
[![GitHub last commit](https://img.shields.io/github/last-commit/alemidev/scope-tui)](https://github.com/alemidev/scope-tui/commits/dev/)
[![GitHub commits since tagged version](https://img.shields.io/github/commits-since/alemidev/scope-tui/v0.3.2)](https://github.com/alemidev/scope-tui/releases/tag/v0.3.2)

A simple oscilloscope/vectorscope/spectroscope for your terminal

![scope-tui interface](https://cdn.alemi.dev/scope-tui-wide.png)

[See it in action here](https://cdn.alemi.dev/scope-tui-oscilloscope-music.webm) with [Planets](https://youtu.be/XziuEdpVUe0) (oscilloscope music by Jerobeam Fenderson)

## Why
I really love [cava](https://github.com/karlstav/cava). It provides a crude but pleasant frequency plot for your music: just the bare minimum to see leads solos and basslines.
I wanted to also be able to see waveforms, but to my knowledge nothing is available. There is some soundcard oscilloscope software available, but the graphical GUI is usually dated and breaks the magic.
I thus decided to solve this very critical issue with my own hands! And over a night of tinkering with pulseaudio (via [libpulse-simple-binding](https://crates.io/crates/libpulse-simple-binding)) and some TUI graphics (via [tui-rs](https://github.com/fdehau/tui-rs)), 
the first version of `scope-tui` was developed, with very minimal settings given from command line, but a bonus vectorscope mode baked in.

# Installation
### Binaries
Pre-build binaries for windows (x64), linux (gnu-64) and macos (arm64, unsigned) are available under the [releases](https://github.com/alemidev/scope-tui/releases) tab

### From source
> [!TIP]
> If you don't have the rust toolchain already installed, get it with [rustup](https://rustup.rs/)

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
The `audio` source is included by default, it works across platforms (windows, macos, linux) and uses the default sound system available. Note that being able to listen your computer audio needs your sound system to have a loopback device (for example, macos doesn't! look into BlackHole)

A very crude file source is also included by default, which can be a named pipe. While this allows connecting `scope-tui` to a lot of things, it's not super convenient, and more specialized sources should be used when available.

Optionally, on Linux the PulseAudio source is available, and directly connects to PulseAudio.

Enable sources by passing the respective feature flags while compiling: `--features=pulseaudio,...`. Disable default features with `--no-default-features`. 
 * `cpal` : generic cross-platform audio implementation, backed by [cpal](https://github.com/rustaudio/cpal) **(enabled by default)**
 * `file` : simple raw-file implementation, just consuming a file source **(enabled by default)**
 * `pulseaudio` : pulseaudio implementation with LibPulse Simple bindings


# Usage
```
Usage: scope-tui [OPTIONS] <COMMAND>

Commands:
  pulse  use PulseAudio Simple api to read data from an audio sink
  file   use a file from filesystem and read its content
  audio  use new experimental CPAL backend
  help   Print this message or the help of the given subcommand(s)

Options:
  -c, --channels <N>                   number of channels to open [default: 2]
  -b, --buffer <SIZE>                  size of audio buffer, and width of scope [default: 2048]
  -r, --sample-rate <HZ>               sample rate to use [default: 48000]
  -t, --tune <NOTE>                    tune buffer size to be in tune with given note (overrides buffer option)
  -s, --scale <x>                      floating point vertical scale, from 0 to 1 [default:1]
      --scatter                        use vintage looking scatter mode instead of line mode
      --no-reference                   don't draw reference line
      --no-ui                          hide UI and only draw waveforms
      --no-braille                     don't use braille dots for drawing lines
      --palette-color <color1,color2>  palette to use for scope signal lines [default: red,yellow,green,magenta]
      --labels-color <color>           color to use for axis labels [default: Cyan]
      --axis-color <color>             color to use for axis lines [default: DarkGray]
  -h, --help                           Print help
  -V, --version                        Print version
```

The audio buffer size directly impacts resource usage, latency and refresh rate and its limits are given by the audio refresh rate. Larger buffers are slower but less resource intensive. A good starting value might be `2048` at `48kHz`, which should guarantee at least 20 frames per second. Increasing sample rate and decreasing buffer size will allow scope-tui to refresh the screen faster.

To change audio buffer size or sample rate, scope-tui must be restarted, as it needs to re-initialize the audio device. Because of this, such options are only available from the command line, and not during usage.

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
	* Use `l` to toggle logarithmic view (default ON)
	* Use `w` to toggle [Hann Window](https://en.wikipedia.org/wiki/Hann_function) (a bit of smoothing)
  * **Vectorscope**:
* Combine increment/decrement commands with `<SHIFT>` to increase or decrease by x10
* Combine increment/decrement commands with `<CTRL>` to increase or decrease by x5
* Combine increment/decrement commands with `<ALT>` to increase or decrease by x 1/5

## About precision
While "scatter" plot mode is as precise as the samples are and the terminal lets us be, "line" plot mode simply draws a straight line across points, meaning high frequencies don't get properly represented.

Latency is kept to a minimum thanks to small buffer and block sizes.

Sample rate and channel count can be freely specified but will ultimately be limited by source's actual sample rate / channel count.

Decrease/increase terminal font size to increase/decrease scope resolution.

# Development
> [!IMPORTANT]
> Any help is appreciated, feel free to contact me if you want to contribuite, either via issues/PRs or with [any of my contacts](https://alemi.dev/about/contacts/)

Some features I plan to work on and would like to add:
 * [x] Oscilloscope
 * [x] Vectorscope
 * [x] Linux audio source
 * [x] Simple controls
 * [x] Simple triggering
 * [x] Multiple channels
 * [x] Spectroscope
 * [x] File source
 * [x] Mac audio sources
 * [x] Windows audio sources
 * [ ] Improve file audio source
 * [ ] Network sources
 * [ ] GUI frontend
 * [ ] Serial sources
 * [ ] USB sources
 * [ ] SDR sources
