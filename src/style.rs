use crate::logos::LogoEntry;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ColorMode {
    Normal,
    NoColor,
    Greyscale,
    Plain,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LabelColor {
    Auto,
    AutoPerLine,
    Plain,
    Named(u8),
    Ansi256(u8),
    Rgb(u8, u8, u8),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LabelFont {
    Plain,
    Bold,
    Italic,
    BoldItalic,
}

#[derive(Clone, Copy)]
pub struct LabelStyle {
    pub color: LabelColor,
    pub font: LabelFont,
    pub colon: bool,
}

pub struct RenderOptions {
    pub mode: ColorMode,
    pub style: LabelStyle,
}

pub fn render(
    logo: &LogoEntry,
    info: &[(String, String)],
    colorize: bool,
    opts: &RenderOptions,
) -> String {
    let mode = if colorize {
        opts.mode
    } else {
        ColorMode::Plain
    };
    let auto_params = resolve_auto(logo.colors);
    let width = logo
        .art
        .iter()
        .map(|line| logo_visible_width(line))
        .max()
        .unwrap_or(0);
    let rows = logo.art.len().max(info.len());
    let mut carry = String::new();
    if let Some(base) = logo.colors.first() {
        carry.push_str(base);
    }
    let mut out = String::new();
    for i in 0..rows {
        let art = logo.art.get(i);
        let text = info.get(i);
        let mut line = String::new();
        if let Some(art) = art {
            paint_logo_line(art, logo.colors, &mut carry, mode, &mut line);
            let pad = width.saturating_sub(logo_visible_width(art));
            for _ in 0..pad {
                line.push(' ');
            }
            if text.is_some() {
                line.push_str("  ");
            }
        } else if text.is_some() && width > 0 {
            for _ in 0..width {
                line.push(' ');
            }
            line.push_str("  ");
        }
        if let Some((label, value)) = text {
            let prefix = label_prefix(opts.style, mode, auto_params.as_deref(), &carry);
            line.push_str(&prefix);
            line.push_str(label);
            if opts.style.colon {
                line.push(':');
            }
            if !prefix.is_empty() {
                line.push_str("\u{1b}[m");
            }
            if !opts.style.colon {
                line.push(':');
            }
            line.push(' ');
            line.push_str(value);
        }
        out.push_str(&line);
        out.push('\n');
    }
    out
}

pub fn parse_color(spec: &str) -> Option<LabelColor> {
    let spec = spec.trim();
    let lower = spec.to_ascii_lowercase();
    match lower.as_str() {
        "auto" => return Some(LabelColor::Auto),
        "auto-per-line" | "autoperline" | "per-line" | "perline" => {
            return Some(LabelColor::AutoPerLine);
        }
        "none" | "default" | "plain" | "off" => return Some(LabelColor::Plain),
        _ => {}
    }
    if let Some(code) = named_ansi(&lower) {
        return Some(LabelColor::Named(code));
    }
    if !spec.is_empty() && spec.bytes().all(|b| b.is_ascii_digit()) {
        return match spec.parse::<u16>() {
            Ok(n) if n <= 255 => Some(LabelColor::Ansi256(n as u8)),
            _ => None,
        };
    }
    let hex = spec.strip_prefix('#').unwrap_or(spec);
    if matches!(hex.len(), 3 | 6) && hex.bytes().all(|b| b.is_ascii_hexdigit()) {
        let digits: Vec<u8> = hex
            .bytes()
            .map(|b| match b {
                b'0'..=b'9' => b - b'0',
                _ => b.to_ascii_lowercase() - b'a' + 10,
            })
            .collect();
        if hex.len() == 3 {
            return Some(LabelColor::Rgb(
                digits[0] * 17,
                digits[1] * 17,
                digits[2] * 17,
            ));
        }
        return Some(LabelColor::Rgb(
            digits[0] * 16 + digits[1],
            digits[2] * 16 + digits[3],
            digits[4] * 16 + digits[5],
        ));
    }
    None
}

pub fn parse_font(spec: &str) -> Option<LabelFont> {
    match spec.trim().to_ascii_lowercase().as_str() {
        "plain" | "none" | "normal" | "off" | "default" => Some(LabelFont::Plain),
        "bold" => Some(LabelFont::Bold),
        "italic" => Some(LabelFont::Italic),
        "bold-italic" | "bolditalic" | "bold_italic" | "bold italic" | "italic-bold" => {
            Some(LabelFont::BoldItalic)
        }
        _ => None,
    }
}

pub fn color_spec(color: LabelColor) -> String {
    match color {
        LabelColor::Auto => "auto".to_string(),
        LabelColor::AutoPerLine => "auto-per-line".to_string(),
        LabelColor::Plain => "none".to_string(),
        LabelColor::Named(code) => match code {
            30 => "black",
            31 => "red",
            32 => "green",
            33 => "yellow",
            34 => "blue",
            35 => "magenta",
            36 => "cyan",
            37 => "white",
            90 => "gray",
            91 => "bright-red",
            92 => "bright-green",
            93 => "bright-yellow",
            94 => "bright-blue",
            95 => "bright-magenta",
            96 => "bright-cyan",
            97 => "bright-white",
            _ => "none",
        }
        .to_string(),
        LabelColor::Ansi256(n) => n.to_string(),
        LabelColor::Rgb(r, g, b) => format!("#{r:02x}{g:02x}{b:02x}"),
    }
}

pub fn parse_bool(spec: &str) -> Option<bool> {
    match spec.trim().to_ascii_lowercase().as_str() {
        "true" | "yes" | "on" | "1" | "y" => Some(true),
        "false" | "no" | "off" | "0" | "n" => Some(false),
        _ => None,
    }
}

pub fn bool_spec(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}

pub fn font_spec(font: LabelFont) -> &'static str {
    match font {
        LabelFont::Plain => "plain",
        LabelFont::Bold => "bold",
        LabelFont::Italic => "italic",
        LabelFont::BoldItalic => "bold-italic",
    }
}

fn named_ansi(name: &str) -> Option<u8> {
    let compact = name.replace(['-', '_', ' '], "");
    match compact.as_str() {
        "black" => Some(30),
        "red" => Some(31),
        "green" => Some(32),
        "yellow" => Some(33),
        "blue" => Some(34),
        "magenta" | "purple" => Some(35),
        "cyan" => Some(36),
        "white" => Some(37),
        "gray" | "grey" | "brightblack" | "brightgrey" | "darkgray" | "darkgrey" => Some(90),
        "brightred" => Some(91),
        "brightgreen" => Some(92),
        "brightyellow" => Some(93),
        "brightblue" => Some(94),
        "brightmagenta" | "brightpurple" => Some(95),
        "brightcyan" => Some(96),
        "brightwhite" => Some(97),
        _ => None,
    }
}

fn explicit_params(color: LabelColor) -> String {
    match color {
        LabelColor::Plain | LabelColor::Auto | LabelColor::AutoPerLine => String::new(),
        LabelColor::Named(code) => code.to_string(),
        LabelColor::Ansi256(n) => format!("38;5;{n}"),
        LabelColor::Rgb(r, g, b) => format!("38;2;{r};{g};{b}"),
    }
}

fn font_params(font: LabelFont) -> &'static str {
    match font {
        LabelFont::Plain => "",
        LabelFont::Bold => "1",
        LabelFont::Italic => "3",
        LabelFont::BoldItalic => "1;3",
    }
}

fn label_prefix(
    style: LabelStyle,
    mode: ColorMode,
    auto_params: Option<&str>,
    carry: &str,
) -> String {
    if mode == ColorMode::Plain {
        return String::new();
    }
    let color = match style.color {
        LabelColor::Plain => None,
        LabelColor::Auto => auto_params.map(|p| p.to_string()),
        LabelColor::AutoPerLine => {
            if carry.is_empty() {
                None
            } else {
                parse_sgr(carry).map(|c| c.params())
            }
        }
        explicit => Some(explicit_params(explicit)),
    };
    let color = color.and_then(|params| transform_params(&params, mode));
    let font = font_params(style.font);
    let params = match (font.is_empty(), color) {
        (true, None) => return String::new(),
        (true, Some(color)) => color,
        (false, None) => font.to_string(),
        (false, Some(color)) => format!("{font};{color}"),
    };
    format!("\u{1b}[{params}m")
}

fn paint_logo_line(
    line: &str,
    colors: &[&str],
    carry: &mut String,
    mode: ColorMode,
    out: &mut String,
) {
    if logo_visible_width(line) == 0 {
        return;
    }
    let colorized = mode != ColorMode::Plain;
    if colorized {
        out.push_str("\u{1b}[1m");
    }
    if !carry.is_empty()
        && let Some(code) = transform_code(carry, mode)
    {
        out.push_str(&code);
    }
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '$' {
            match chars.peek() {
                Some(digit) if ('1'..='9').contains(digit) => {
                    let index = *digit as usize - '1' as usize;
                    chars.next();
                    let code = colors.get(index).copied().unwrap_or("\u{1b}[m");
                    carry.clear();
                    carry.push_str(code);
                    if let Some(code) = transform_code(carry, mode) {
                        out.push_str(&code);
                    }
                }
                Some('$') => {
                    chars.next();
                    out.push('$');
                }
                _ => {
                    out.push('$');
                }
            }
        } else {
            out.push(c);
        }
    }
    if colorized {
        out.push_str("\u{1b}[m");
    }
}

fn resolve_auto(colors: &[&str]) -> Option<String> {
    let parsed: Vec<SgrColor> = colors.iter().filter_map(|c| parse_sgr(c)).collect();
    for color in &parsed {
        if let Some((r, g, b)) = color.rgb()
            && r > 60
            && g > 60
            && b > 60
        {
            return Some(color.params());
        }
    }
    parsed
        .iter()
        .find_map(|color| color.rgb().map(|_| color.params()))
}

fn transform_code(code: &str, mode: ColorMode) -> Option<String> {
    match mode {
        ColorMode::Plain | ColorMode::NoColor => None,
        ColorMode::Normal => Some(code.to_string()),
        ColorMode::Greyscale => Some(greyscale_code(code)),
    }
}

fn transform_params(params: &str, mode: ColorMode) -> Option<String> {
    match mode {
        ColorMode::Plain | ColorMode::NoColor => None,
        ColorMode::Normal => Some(params.to_string()),
        ColorMode::Greyscale => {
            let code = format!("\u{1b}[{params}m");
            let grey = greyscale_code(&code);
            Some(
                grey.trim_start_matches("\u{1b}[")
                    .trim_end_matches('m')
                    .to_string(),
            )
        }
    }
}

fn greyscale_code(code: &str) -> String {
    match parse_sgr(code).and_then(|c| c.rgb()) {
        Some((r, g, b)) => {
            let luma = ((77 * r as u32) + (150 * g as u32) + (29 * b as u32)) >> 8;
            format!("\u{1b}[38;2;{luma};{luma};{luma}m")
        }
        None => code.to_string(),
    }
}

#[derive(Clone, Copy)]
enum SgrColor {
    Ansi(u8),
    Ansi256(u8),
    Rgb(u8, u8, u8),
    Default,
}

impl SgrColor {
    fn params(self) -> String {
        match self {
            SgrColor::Ansi(code) => code.to_string(),
            SgrColor::Ansi256(n) => format!("38;5;{n}"),
            SgrColor::Rgb(r, g, b) => format!("38;2;{r};{g};{b}"),
            SgrColor::Default => "39".to_string(),
        }
    }

    fn rgb(self) -> Option<(u8, u8, u8)> {
        match self {
            SgrColor::Ansi(code) => Some(ansi_rgb(code)),
            SgrColor::Ansi256(n) => Some(ansi256_rgb(n)),
            SgrColor::Rgb(r, g, b) => Some((r, g, b)),
            SgrColor::Default => None,
        }
    }
}

fn parse_sgr(code: &str) -> Option<SgrColor> {
    let body = code.strip_prefix("\u{1b}[")?;
    let body = body.strip_suffix('m')?;
    if body == "39" {
        return Some(SgrColor::Default);
    }
    if let Some(rest) = body.strip_prefix("38;5;") {
        let n: u8 = rest.parse().ok()?;
        return Some(SgrColor::Ansi256(n));
    }
    if let Some(rest) = body.strip_prefix("38;2;") {
        let mut parts = rest.split(';');
        let r: u8 = parts.next()?.parse().ok()?;
        let g: u8 = parts.next()?.parse().ok()?;
        let b: u8 = parts.next()?.parse().ok()?;
        return Some(SgrColor::Rgb(r, g, b));
    }
    let code: u8 = body.parse().ok()?;
    if (30..=37).contains(&code) || (90..=97).contains(&code) {
        Some(SgrColor::Ansi(code))
    } else {
        None
    }
}

const BASE: [(u8, u8, u8); 8] = [
    (0, 0, 0),
    (205, 0, 0),
    (0, 205, 0),
    (205, 205, 0),
    (0, 0, 238),
    (205, 0, 205),
    (0, 205, 205),
    (229, 229, 229),
];

const BRIGHT: [(u8, u8, u8); 8] = [
    (127, 127, 127),
    (255, 0, 0),
    (0, 255, 0),
    (255, 255, 0),
    (92, 92, 255),
    (255, 0, 255),
    (0, 255, 255),
    (255, 255, 255),
];

fn cube_level(level: u8) -> u8 {
    if level == 0 { 0 } else { level * 40 + 55 }
}

fn ansi_rgb(code: u8) -> (u8, u8, u8) {
    match code {
        30..=37 => BASE[(code - 30) as usize],
        90..=97 => BRIGHT[(code - 90) as usize],
        _ => (0, 0, 0),
    }
}

fn ansi256_rgb(n: u8) -> (u8, u8, u8) {
    match n {
        0..=7 => BASE[n as usize],
        8..=15 => BRIGHT[(n - 8) as usize],
        16..=231 => {
            let i = n - 16;
            let r = i / 36;
            let g = (i / 6) % 6;
            let b = i % 6;
            (cube_level(r), cube_level(g), cube_level(b))
        }
        232..=255 => {
            let v = n as u16 * 10 + 8;
            (v as u8, v as u8, v as u8)
        }
    }
}

pub fn char_width(c: char) -> usize {
    match c as u32 {
        0x0300..=0x036F | 0x200D | 0xFE00..=0xFE0F => 0,
        0x1100..=0x115F
        | 0x2E80..=0xA4CF
        | 0xAC00..=0xD7A3
        | 0xF900..=0xFAFF
        | 0xFE30..=0xFE4F
        | 0xFF00..=0xFF60
        | 0xFFE0..=0xFFE6
        | 0x2600..=0x27BF
        | 0x2B00..=0x2BFF
        | 0x1F300..=0x1FAFF
        | 0x20000..=0x3FFFD => 2,
        _ => 1,
    }
}

pub fn logo_visible_width(line: &str) -> usize {
    let mut width = 0;
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '$' {
            match chars.peek() {
                Some('1'..='9') => {
                    chars.next();
                }
                Some('$') => {
                    chars.next();
                    width += 1;
                }
                _ => {
                    width += 1;
                }
            }
        } else {
            width += char_width(c);
        }
    }
    width
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sgr(params: &str) -> &'static str {
        Box::leak(format!("\u{1b}[{params}m").into_boxed_str())
    }

    #[test]
    fn parses_color_forms() {
        assert_eq!(parse_color("auto"), Some(LabelColor::Auto));
        assert_eq!(parse_color("AUTO"), Some(LabelColor::Auto));
        assert_eq!(parse_color("auto-per-line"), Some(LabelColor::AutoPerLine));
        assert_eq!(parse_color("none"), Some(LabelColor::Plain));
        assert_eq!(parse_color("plain"), Some(LabelColor::Plain));
        assert_eq!(parse_color("cyan"), Some(LabelColor::Named(36)));
        assert_eq!(parse_color("BRIGHT-blue"), Some(LabelColor::Named(94)));
        assert_eq!(parse_color("gray"), Some(LabelColor::Named(90)));
        assert_eq!(parse_color("202"), Some(LabelColor::Ansi256(202)));
        assert_eq!(parse_color("#ff0000"), Some(LabelColor::Rgb(255, 0, 0)));
        assert_eq!(parse_color("#f00"), Some(LabelColor::Rgb(255, 0, 0)));
        assert_eq!(parse_color(""), None);
        assert_eq!(parse_color("nonsense"), None);
        assert_eq!(parse_color("256"), None);
    }

    #[test]
    fn parses_fonts() {
        assert_eq!(parse_font("plain"), Some(LabelFont::Plain));
        assert_eq!(parse_font("bold"), Some(LabelFont::Bold));
        assert_eq!(parse_font("italic"), Some(LabelFont::Italic));
        assert_eq!(parse_font("bold-italic"), Some(LabelFont::BoldItalic));
        assert_eq!(parse_font("weird"), None);
    }

    #[test]
    fn auto_picks_primary_color() {
        let palette = [sgr("36"), sgr("36")];
        assert_eq!(resolve_auto(&palette), Some("36".to_string()));
    }

    #[test]
    fn auto_skips_near_black() {
        let palette = [sgr("30"), sgr("37")];
        assert_eq!(resolve_auto(&palette), Some("37".to_string()));
    }

    #[test]
    fn auto_falls_back_when_all_dark() {
        let palette = [sgr("30")];
        assert_eq!(resolve_auto(&palette), Some("30".to_string()));
    }

    #[test]
    fn auto_skips_default_color() {
        let palette = [sgr("39"), sgr("34")];
        assert_eq!(resolve_auto(&palette), Some("34".to_string()));
    }

    #[test]
    fn auto_handles_truecolor() {
        let palette = [sgr("38;2;79;157;207")];
        assert_eq!(resolve_auto(&palette), Some("38;2;79;157;207".to_string()));
    }

    #[test]
    fn greyscale_uses_luma() {
        assert_eq!(
            transform_code(sgr("31"), ColorMode::Greyscale).as_deref(),
            Some(sgr("38;2;61;61;61"))
        );
        assert_eq!(
            transform_code(sgr("38;2;255;255;255"), ColorMode::Greyscale).as_deref(),
            Some(sgr("38;2;255;255;255"))
        );
    }

    #[test]
    fn no_color_drops_color_codes() {
        assert_eq!(transform_code(sgr("36"), ColorMode::NoColor), None);
        assert_eq!(transform_params("36", ColorMode::NoColor), None);
    }

    #[test]
    fn label_prefix_composition() {
        let style = LabelStyle {
            color: LabelColor::Named(36),
            font: LabelFont::Bold,
            colon: true,
        };
        assert_eq!(
            label_prefix(style, ColorMode::Normal, None, ""),
            "\u{1b}[1;36m"
        );

        let plain = LabelStyle {
            color: LabelColor::Plain,
            font: LabelFont::Plain,
            colon: true,
        };
        assert_eq!(label_prefix(plain, ColorMode::Normal, None, ""), "");

        let no_color = LabelStyle {
            color: LabelColor::Named(36),
            font: LabelFont::Italic,
            colon: true,
        };
        assert_eq!(
            label_prefix(no_color, ColorMode::NoColor, None, ""),
            "\u{1b}[3m"
        );

        let plain_mode = LabelStyle {
            color: LabelColor::Named(36),
            font: LabelFont::Bold,
            colon: true,
        };
        assert_eq!(label_prefix(plain_mode, ColorMode::Plain, None, ""), "");

        let per_line = LabelStyle {
            color: LabelColor::AutoPerLine,
            font: LabelFont::Bold,
            colon: true,
        };
        assert_eq!(
            label_prefix(per_line, ColorMode::Normal, None, sgr("38;2;10;20;30")),
            "\u{1b}[1;38;2;10;20;30m"
        );
        assert_eq!(
            label_prefix(per_line, ColorMode::Normal, None, ""),
            "\u{1b}[1m"
        );
    }

    #[test]
    fn per_line_tracks_logo_color() {
        static LOGO: LogoEntry = LogoEntry {
            names: &["test"],
            colors: &["\u{1b}[36m", "\u{1b}[31m"],
            art: &["$2xx", "yy"],
        };
        let info = vec![
            ("Hostname".to_string(), "mac".to_string()),
            ("OS".to_string(), "test".to_string()),
        ];
        let opts = RenderOptions {
            mode: ColorMode::Normal,
            style: LabelStyle {
                color: LabelColor::AutoPerLine,
                font: LabelFont::Bold,
                colon: true,
            },
        };
        let out = render(&LOGO, &info, true, &opts);
        assert!(out.contains("\u{1b}[1;31mHostname"));
        assert!(out.contains("\u{1b}[1;31mOS"));
    }

    #[test]
    fn colon_is_styled_by_default() {
        static LOGO: LogoEntry = LogoEntry {
            names: &["test"],
            colors: &["\u{1b}[36m"],
            art: &["xxxxx"],
        };
        let info = vec![("Hostname".to_string(), "mac".to_string())];
        let opts = RenderOptions {
            mode: ColorMode::Normal,
            style: LabelStyle {
                color: LabelColor::Named(36),
                font: LabelFont::Bold,
                colon: true,
            },
        };
        let out = render(&LOGO, &info, true, &opts);
        assert!(out.contains("\u{1b}[1;36mHostname:\u{1b}[m mac"));
    }

    #[test]
    fn colon_stays_plain_when_disabled() {
        static LOGO: LogoEntry = LogoEntry {
            names: &["test"],
            colors: &["\u{1b}[36m"],
            art: &["xxxxx"],
        };
        let info = vec![("Hostname".to_string(), "mac".to_string())];
        let opts = RenderOptions {
            mode: ColorMode::Normal,
            style: LabelStyle {
                color: LabelColor::Named(36),
                font: LabelFont::Bold,
                colon: false,
            },
        };
        let out = render(&LOGO, &info, true, &opts);
        assert!(out.contains("\u{1b}[1;36mHostname\u{1b}[m: mac"));
    }

    #[test]
    fn parses_bools() {
        assert_eq!(parse_bool("yes"), Some(true));
        assert_eq!(parse_bool("false"), Some(false));
        assert_eq!(parse_bool("ON"), Some(true));
        assert_eq!(parse_bool("n"), Some(false));
        assert_eq!(parse_bool("maybe"), None);
        assert_eq!(bool_spec(true), "yes");
        assert_eq!(bool_spec(false), "no");
    }

    #[test]
    fn render_without_color_is_plain() {
        static LOGO: LogoEntry = LogoEntry {
            names: &["test"],
            colors: &["\u{1b}[36m"],
            art: &["xxxxx"],
        };
        let info = vec![("Hostname".to_string(), "mac".to_string())];
        let opts = RenderOptions {
            mode: ColorMode::Normal,
            style: LabelStyle {
                color: LabelColor::Auto,
                font: LabelFont::Bold,
                colon: true,
            },
        };
        let out = render(&LOGO, &info, false, &opts);
        assert_eq!(out, "xxxxx  Hostname: mac\n");
    }
}
