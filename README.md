# whiskerfetch

[![crates.io](https://img.shields.io/crates/v/whiskerfetch.svg)](https://crates.io/crates/whiskerfetch)
[![license](https://img.shields.io/crates/l/whiskerfetch.svg)](https://github.com/skoshic/wfetch/blob/main/LICENSE)

```text
                    ..
     .@@@.        .@@@@
     *@@@@%     .%@@@@@.
    .@@@@@@@@@@@@@@@@@@%
   .@@@@@@@@@@@@@@@@@@@*    | Whiskerfetch
  .@@@@@@@@@@@@@@@@@@@@@
  %@@@@@@@@@@@@@@@@@@@@@@
 .@@@*%%%%%*@@@@%%%%%@@@@%  | Whiskerfetch is a fast and
 %@@@.......%@@%.......@@%  | customizable sysfetch.
 %@@..%@@@*..@@..%@@*..%@%
 .@@@..%*...*@@...%@...%@%
 .%@@%......@@@%.......@@.
  .%@@@@@@@@@@@@@@*@@@@@%
     %@@@@@@@@@@@@@@@@.
      .%@@@@@@@@%%%%.
```

## Description

```sh
Hostname: MacBook
OS: macOS 27.0.1
Kernel: Darwin 27.0.0
Device: MacBook Air 13-inch M5 2026
Uptime: 8 days, 12 hours, 17 mins
Shell: btsh
Init: launchd
Terminal: Rio
CPU: Apple M5 (10 cores)
RAM: 11.62 GiB / 16.00 GiB (72%)
GPU: Apple M5
Display (1): 2940x1912 @ 60Hz
Display (2): 2560x1440 @ 180Hz
Disk (/): 269.99 GiB / 460.38 GiB (58%)
Local IP (en0): 192.168.178.67/24
```

Whiskerfetch (short: wfetch) is a very fast and zero dependency sysfetch that works mainly via (painful) C extern functions.
I hope to make it more customizable as the project grows.

## Installation

Either ...

... through Cargo:

```sh
cargo install whiskerfetch
```
... through releases:

Go on the [releases page](https://github.com/skoshic/wfetch/releases/latest) and download the latest binary.

... through git:

```sh
git clone https://github.com/skoshic/wfetch
cd wfetch
cargo build --release
cargo install --path .
```

## Usage

After installing it to your PATH you can simply run it with `wfetch`.

### Flags

```sh
wfetch --label-color cyan --label-font italic   # style this run
wfetch --no-label-colon                         # print the colon plain
wfetch --no-color                               # keep fonts, use the default color
wfetch --greyscale                              # keep fonts, turn every color grey
wfetch --plain                                  # no colors and no fonts
wfetch --configure                              # interactive TUI to build the config
```

## Customization

Labels are `bold` and match the logo's `auto` color by default. Both can be changed
permanently or per run.

### Config file

wfetch reads `~/.config/wfetch/config`:

```toml
[label]
color = auto
font = bold
colon = yes
```

`color` accepts `auto` (the logo's primary color), `auto-per-line` (each label
takes the color of the logo part next to it), `none`, a color name
(`black`, `red`, `green`, `yellow`, `blue`, `magenta`, `cyan`, `white`,
`gray`, `bright-<name>`), a `0-255` ANSI index, or `#rrggbb`.

`font` accepts `plain`, `bold`, `italic`, or `bold-italic`.

`colon` accepts `yes` (the colon shares the label's font and color, the
default) or `no` (the colon is printed plain).

Precedence:
- Flags beat config
- `--json` doesn't use styling at all

## Benchmarks

```sh
* tested on a MacBook Air M5:

 [~/Projects/wfetch] % hyperfine -N --warmup 100 ./target/release/wfetch
Benchmark 1: ./target/release/wfetch (317 runs)
                mean     ±       σ          min     …     max
  Wall Time      8.6 ms  ±     0.9 ms       7.6 ms  …    13.3 ms
  Memory         9.6 MiB ±     0.0 MiB      9.5 MiB …     9.6 MiB
```

## License

Whiskerfetch is licensed under the MIT license.

Terminal logos are adapted from [fastfetch](https://github.com/fastfetch-cli/fastfetch) (MIT licensed. see `assets/fastfetch-LICENSE`).
