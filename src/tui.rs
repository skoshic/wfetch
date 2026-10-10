use std::io::Write;
use std::process::{Command, Stdio};

use crate::style::{
    ColorMode, LabelColor, LabelFont, LabelStyle, RenderOptions, bool_spec, color_spec, font_spec,
    render,
};
use crate::{config, gather_info, select_platform_logo};

#[repr(C)]
struct PollFd {
    fd: i32,
    events: i16,
    revents: i16,
}

unsafe extern "C" {
    fn poll(fds: *mut PollFd, nfds: u64, timeout: i32) -> i32;
    fn isatty(fd: i32) -> i32;
    fn read(fd: i32, buf: *mut std::ffi::c_void, count: usize) -> isize;
}

const POLLIN: i16 = 0x001;

enum Key {
    Up,
    Down,
    Left,
    Right,
    Save,
    Quit,
    Unknown,
}

struct RawTerminal {
    saved: Option<String>,
}

impl RawTerminal {
    fn enable() -> RawTerminal {
        let saved = Command::new("stty")
            .arg("-g")
            .stdin(Stdio::inherit())
            .output()
            .ok()
            .and_then(|out| String::from_utf8(out.stdout).ok())
            .map(|state| state.trim().to_string())
            .filter(|state| !state.is_empty());
        if saved.is_some() {
            let _ = Command::new("stty")
                .args(["raw", "-echo"])
                .stdin(Stdio::inherit())
                .status();
        }
        let _ = std::io::stdout().write_all(b"\x1b[?25l");
        RawTerminal { saved }
    }
}

impl Drop for RawTerminal {
    fn drop(&mut self) {
        let _ = std::io::stdout().write_all(b"\x1b[?25h");
        if let Some(state) = &self.saved {
            let _ = Command::new("stty")
                .arg(state)
                .stdin(Stdio::inherit())
                .status();
        }
    }
}

fn readable(timeout_ms: i32) -> bool {
    let mut fds = PollFd {
        fd: 0,
        events: POLLIN,
        revents: 0,
    };
    unsafe { poll(&mut fds, 1, timeout_ms) == 1 }
}

fn read_fd_byte() -> Option<u8> {
    let mut buf = [0u8; 1];
    if unsafe { read(0, buf.as_mut_ptr() as *mut std::ffi::c_void, 1) } == 1 {
        Some(buf[0])
    } else {
        None
    }
}

fn read_byte(timeout_ms: i32) -> Option<u8> {
    if !readable(timeout_ms) {
        return None;
    }
    read_fd_byte()
}

fn read_byte_blocking() -> Option<u8> {
    loop {
        if !readable(1000) {
            continue;
        }
        return read_fd_byte();
    }
}

fn read_key() -> Key {
    let Some(byte) = read_byte_blocking() else {
        return Key::Quit;
    };
    match byte {
        b'q' | 3 => Key::Quit,
        b's' => Key::Save,
        b'j' => Key::Down,
        b'k' => Key::Up,
        b'h' => Key::Left,
        b'l' | b' ' => Key::Right,
        b'\r' | b'\n' => Key::Right,
        27 => {
            if !readable(60) {
                return Key::Quit;
            }
            let mut tail = [0u8; 2];
            let mut got = 0;
            while got < 2 {
                match read_byte(60) {
                    Some(byte) => {
                        tail[got] = byte;
                        got += 1;
                    }
                    None => break,
                }
            }
            if got == 0 {
                return Key::Quit;
            }
            if got == 2 && tail[0] == b'[' {
                match tail[1] {
                    b'A' => Key::Up,
                    b'B' => Key::Down,
                    b'C' => Key::Right,
                    b'D' => Key::Left,
                    _ => Key::Unknown,
                }
            } else {
                Key::Unknown
            }
        }
        _ => Key::Unknown,
    }
}

fn cycle(index: usize, len: usize, delta: isize) -> usize {
    let mut next = index as isize + delta;
    while next < 0 {
        next += len as isize;
    }
    (next % len as isize) as usize
}

pub fn run() {
    if unsafe { isatty(0) } != 1 {
        eprintln!("wfetch: --configure needs an interactive terminal");
        return;
    }
    let logo = select_platform_logo();
    let info = gather_info();
    let config = config::load();
    let color = config.label_color.unwrap_or(LabelColor::Auto);
    let font = config.label_font.unwrap_or(LabelFont::Bold);
    let colon = config.label_colon.unwrap_or(true);
    let mut colors = vec![
        LabelColor::Auto,
        LabelColor::AutoPerLine,
        LabelColor::Plain,
        LabelColor::Named(30),
        LabelColor::Named(31),
        LabelColor::Named(32),
        LabelColor::Named(33),
        LabelColor::Named(34),
        LabelColor::Named(35),
        LabelColor::Named(36),
        LabelColor::Named(37),
        LabelColor::Named(90),
        LabelColor::Named(97),
    ];
    if !colors.contains(&color) {
        colors.push(color);
    }
    let mut fonts = vec![
        LabelFont::Plain,
        LabelFont::Bold,
        LabelFont::Italic,
        LabelFont::BoldItalic,
    ];
    if !fonts.contains(&font) {
        fonts.push(font);
    }
    let colons = [false, true];
    let mut color_index = colors.iter().position(|c| *c == color).unwrap_or(0);
    let mut font_index = fonts.iter().position(|f| *f == font).unwrap_or(1);
    let mut colon_index = colons.iter().position(|c| *c == colon).unwrap_or(1);
    let mut row = 0;
    let rows_len = 3;
    let config_path = config::config_path()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| "~/.config/wfetch/config".to_string());
    let _terminal = RawTerminal::enable();
    let mut stdout = std::io::stdout();
    let mut result: Option<LabelStyle> = None;
    loop {
        let style = LabelStyle {
            color: colors[color_index],
            font: fonts[font_index],
            colon: colons[colon_index],
        };
        let preview = render(
            logo,
            &info,
            true,
            &RenderOptions {
                mode: ColorMode::Normal,
                style,
            },
        );
        let menu = [
            ("Label color", color_spec(colors[color_index])),
            ("Label font", font_spec(fonts[font_index]).to_string()),
            ("Label colon", bool_spec(colons[colon_index]).to_string()),
        ];
        let mut screen = String::from("\u{1b}[2J\u{1b}[H");
        screen.push_str("wfetch configuration\r\n\r\n");
        for (index, (key, value)) in menu.iter().enumerate() {
            let marker = if index == row {
                "\u{1b}[1;36m>\u{1b}[m"
            } else {
                " "
            };
            screen.push_str(&format!("{marker} {key:<12} {value}\r\n"));
        }
        screen.push_str("\r\n");
        screen.push_str("\u{1b}[2mup/down select, left/right change, s save, q quit\u{1b}[m\r\n");
        screen.push_str(&format!("\u{1b}[2mconfig: {config_path}\u{1b}[m\r\n"));
        screen.push_str("\u{1b}[2mpreview: \u{1b}[m\r\n");
        screen.push_str(&preview.replace('\n', "\r\n"));
        let _ = stdout.write_all(screen.as_bytes());
        let _ = stdout.flush();
        let key = read_key();
        match key {
            Key::Up => row = row.saturating_sub(1),
            Key::Down => row = (row + 1).min(rows_len - 1),
            Key::Left | Key::Right => {
                let delta = if matches!(key, Key::Left) { -1 } else { 1 };
                match row {
                    0 => color_index = cycle(color_index, colors.len(), delta),
                    1 => font_index = cycle(font_index, fonts.len(), delta),
                    2 => colon_index = cycle(colon_index, colons.len(), delta),
                    _ => {}
                }
            }
            Key::Save => {
                result = Some(style);
                break;
            }
            Key::Quit => break,
            Key::Unknown => {}
        }
    }
    drop(_terminal);
    if let Some(style) = result {
        match config::save(&style) {
            Ok(path) => println!("wfetch: saved configuration to {}", path.display()),
            Err(err) => eprintln!("wfetch: {err}"),
        }
    }
}
