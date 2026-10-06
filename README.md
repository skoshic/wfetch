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
Uptime: 2 days, 12 hours, 9 mins
Shell: btsh
Init: launchd
Terminal: Rio
CPU: Apple M5 (10 cores)
RAM: 10.22 GiB / 16.00 GiB (63%)
GPU: Apple M5
Display: 2940x1912 @ 60Hz
Disk (/): 265.11 GiB / 460.38 GiB (57%)
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

## Benchmarks

```sh
* tested on a MacBook Air M5:

 [~/Projects/wfetch] % hyperfine -N --warmup 100 ./target/release/wfetch
Benchmark 1: ./target/release/wfetch
  Time (mean ± σ):       7.6 ms ±   1.5 ms    [User: 1.8 ms, System: 1.8 ms]
  Range (min … max):     6.4 ms …  12.8 ms    445 runs
```

## License

Whiskerfetch is licensed under the MIT license.

Terminal logos are adapted from [fastfetch](https://github.com/fastfetch-cli/fastfetch) (MIT licensed. see `assets/fastfetch-LICENSE`).
