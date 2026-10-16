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

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Align {
    Top,
    Center,
    Bottom,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum InfoTheme {
    Plain,
    Tree,
    Dots,
    Bar,
    Arrow,
    Diamond,
    Block,
    Pipe,
    Round,
    Square,
    Double,
    Ascii,
    Bold,
    Column,
    Groups,
    Title,
    Outline,
    Dotted,
    Grid,
    Compact,
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
    pub align: Align,
    pub theme: InfoTheme,
    pub border_color: LabelColor,
}

#[derive(Clone, Copy)]
struct Frame {
    top: (char, char, char),
    bottom: (char, char, char),
    left: char,
    right: char,
}

impl InfoTheme {
    fn frame(self) -> Option<Frame> {
        match self {
            InfoTheme::Round => Some(Frame {
                top: ('\u{256d}', '\u{2500}', '\u{256e}'),
                bottom: ('\u{2570}', '\u{2500}', '\u{256f}'),
                left: '\u{2502}',
                right: '\u{2502}',
            }),
            InfoTheme::Square => Some(Frame {
                top: ('\u{250c}', '\u{2500}', '\u{2510}'),
                bottom: ('\u{2514}', '\u{2500}', '\u{2518}'),
                left: '\u{2502}',
                right: '\u{2502}',
            }),
            InfoTheme::Double => Some(Frame {
                top: ('\u{2554}', '\u{2550}', '\u{2557}'),
                bottom: ('\u{255a}', '\u{2550}', '\u{255d}'),
                left: '\u{2551}',
                right: '\u{2551}',
            }),
            InfoTheme::Ascii => Some(Frame {
                top: ('+', '-', '+'),
                bottom: ('+', '-', '+'),
                left: '|',
                right: '|',
            }),
            InfoTheme::Bold => Some(Frame {
                top: ('\u{250f}', '\u{2501}', '\u{2513}'),
                bottom: ('\u{2517}', '\u{2501}', '\u{251b}'),
                left: '\u{2503}',
                right: '\u{2503}',
            }),
            _ => None,
        }
    }

    fn lead(self, last: bool) -> &'static str {
        match self {
            InfoTheme::Tree => {
                if last {
                    "\u{2570}\u{2500} "
                } else {
                    "\u{251c}\u{2500} "
                }
            }
            InfoTheme::Dots => "\u{25cf} ",
            InfoTheme::Bar => "\u{258c} ",
            InfoTheme::Arrow => "\u{2192} ",
            InfoTheme::Diamond => "\u{25c6} ",
            InfoTheme::Block => "\u{2588} ",
            InfoTheme::Pipe => "\u{2502} ",
            _ => "",
        }
    }

    fn divider(self) -> &'static str {
        match self {
            InfoTheme::Column => " \u{2502} ",
            _ => "  ",
        }
    }
}

enum Block {
    Data(usize),
    Gap,
    Title,
    Rule,
    Head(usize),
    GridTop,
    GridBottom,
}

fn section_for_label(label: &str) -> usize {
    if label.starts_with("Hostname")
        || label.starts_with("OS")
        || label.starts_with("Kernel")
        || label.starts_with("Device")
        || label.starts_with("Uptime")
    {
        0
    } else if label.starts_with("Shell") || label.starts_with("Init") || label.starts_with("Terminal") {
        1
    } else if label.starts_with("CPU") || label.starts_with("RAM") || label.starts_with("GPU") {
        2
    } else {
        3
    }
}

fn section_name(section: usize) -> &'static str {
    match section {
        0 => "System",
        1 => "Software",
        2 => "Hardware",
        _ => "Details",
    }
}

fn build_block(theme: InfoTheme, info: &[(String, String)]) -> Vec<Block> {
    match theme {
        InfoTheme::Groups => {
            let mut out = Vec::new();
            let mut prev: Option<usize> = None;
            for (index, (label, _)) in info.iter().enumerate() {
                let section = section_for_label(label);
                if let Some(p) = prev {
                    if p != section {
                        out.push(Block::Gap);
                    }
                }
                out.push(Block::Data(index));
                prev = Some(section);
            }
            out
        }
        InfoTheme::Title => {
            let mut out = Vec::new();
            if info.is_empty() {
                return out;
            }
            out.push(Block::Title);
            out.push(Block::Rule);
            for index in 1..info.len() {
                out.push(Block::Data(index));
            }
            out
        }
        InfoTheme::Outline => {
            let mut out = Vec::new();
            let mut prev: Option<usize> = None;
            for (index, (label, _)) in info.iter().enumerate() {
                let section = section_for_label(label);
                if prev != Some(section) {
                    out.push(Block::Head(section));
                }
                out.push(Block::Data(index));
                prev = Some(section);
            }
            out
        }
        InfoTheme::Grid => {
            let mut out = Vec::new();
            if info.is_empty() {
                return out;
            }
            out.push(Block::GridTop);
            for index in 0..info.len() {
                out.push(Block::Data(index));
            }
            out.push(Block::GridBottom);
            out
        }
        _ => {
            let mut out = Vec::new();
            for index in 0..info.len() {
                out.push(Block::Data(index));
            }
            out
        }
    }
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
    let logo_rows = logo.art.len();
    let frame = opts.theme.frame();
    let max_label = info
        .iter()
        .map(|(label, _)| text_width(label))
        .max()
        .unwrap_or(0);
    let max_value = info
        .iter()
        .map(|(_, value)| text_width(value))
        .max()
        .unwrap_or(0);
    let inner_plain = info
        .iter()
        .map(|(label, value)| text_width(label) + 2 + text_width(value))
        .max()
        .unwrap_or(0);
    let inner_outline = info
        .iter()
        .map(|(label, value)| 2 + text_width(label) + 2 + text_width(value))
        .max()
        .unwrap_or(0);
    let block = build_block(opts.theme, info);
    let info_rows = info.len();
    let legacy_inner = info
        .iter()
        .enumerate()
        .map(|(index, (label, value))| {
            let lead = opts.theme.lead(index + 1 == info_rows);
            text_width(lead) + text_width(label) + 2 + text_width(value)
        })
        .max()
        .unwrap_or(0);
    let inner_width = match opts.theme {
        InfoTheme::Outline => inner_outline,
        InfoTheme::Grid => max_label + max_value + 7,
        _ => {
            if frame.is_some() {
                legacy_inner
            } else {
                inner_plain
            }
        }
    };
    let block_rows = if frame.is_some() {
        info_rows + 2
    } else {
        block.len()
    };
    let offset = match opts.align {
        Align::Top => 0,
        Align::Center => logo_rows.saturating_sub(block_rows) / 2,
        Align::Bottom => logo_rows.saturating_sub(block_rows),
    };
    let rows = logo_rows.max(block_rows);
    let mut carry = String::new();
    if let Some(base) = logo.colors.first() {
        carry.push_str(base);
    }
    let mut out = String::new();
    for i in 0..rows {
        let art = logo.art.get(i);
        let slot = i.checked_sub(offset).filter(|index| *index < block_rows);
        let mut line = String::new();
        if let Some(art) = art {
            paint_logo_line(art, logo.colors, &mut carry, mode, &mut line);
            let pad = width.saturating_sub(logo_visible_width(art));
            for _ in 0..pad {
                line.push(' ');
            }
            if slot.is_some() {
                line.push_str(opts.theme.divider());
            }
        } else if slot.is_some() && width > 0 {
            for _ in 0..width {
                line.push(' ');
            }
            line.push_str(opts.theme.divider());
        }
        if let Some(index) = slot {
            render_block_row(
                &mut line,
                index,
                frame.as_ref(),
                &block,
                info,
                info_rows,
                inner_width,
                max_label,
                max_value,
                opts,
                mode,
                auto_params.as_deref(),
                &carry,
            );
        }
        out.push_str(&line);
        out.push('\n');
    }
    out
}

#[allow(clippy::too_many_arguments)]
fn render_block_row(
    line: &mut String,
    index: usize,
    frame: Option<&Frame>,
    block: &[Block],
    info: &[(String, String)],
    info_rows: usize,
    inner_width: usize,
    max_label: usize,
    max_value: usize,
    opts: &RenderOptions,
    mode: ColorMode,
    auto_params: Option<&str>,
    carry: &str,
) {
    if let Some(frame) = frame {
        let border = decoration_prefix(opts.border_color, mode, auto_params, carry);
        let reset = if border.is_empty() { "" } else { "\u{1b}[m" };
        if index == 0 {
            line.push_str(&border);
            line.push(frame.top.0);
            for _ in 0..inner_width + 2 {
                line.push(frame.top.1);
            }
            line.push(frame.top.2);
            line.push_str(reset);
            return;
        }
        if index == info_rows + 1 {
            line.push_str(&border);
            line.push(frame.bottom.0);
            for _ in 0..inner_width + 2 {
                line.push(frame.bottom.1);
            }
            line.push(frame.bottom.2);
            line.push_str(reset);
            return;
        }
        line.push_str(&border);
        line.push(frame.left);
        line.push_str(reset);
        line.push(' ');
        let used = render_data(
            line,
            index - 1,
            info,
            info_rows,
            inner_width,
            max_label,
            opts,
            mode,
            auto_params,
            carry,
        );
        for _ in used..inner_width {
            line.push(' ');
        }
        line.push(' ');
        line.push_str(&border);
        line.push(frame.right);
        line.push_str(reset);
        return;
    }
    let item = match block.get(index) {
        Some(item) => item,
        None => return,
    };
    match item {
        Block::Data(info_index) => {
            if opts.theme == InfoTheme::Grid {
                render_grid_row(
                    line,
                    *info_index,
                    info,
                    max_label,
                    max_value,
                    opts,
                    mode,
                    auto_params,
                    carry,
                );
            } else {
                render_data(
                    line,
                    *info_index,
                    info,
                    info_rows,
                    inner_width,
                    max_label,
                    opts,
                    mode,
                    auto_params,
                    carry,
                );
            }
        }
        Block::Gap => {
            let decor = decoration_prefix(opts.border_color, mode, auto_params, carry);
            line.push_str(&decor);
            line.push('\u{2502}');
            if !decor.is_empty() {
                line.push_str("\u{1b}[m");
            }
        }
        Block::Title => {
            if let Some((_, value)) = info.first() {
                let prefix = label_prefix(opts.style, mode, auto_params, carry);
                line.push_str(&prefix);
                line.push_str(value);
                if !prefix.is_empty() {
                    line.push_str("\u{1b}[m");
                }
            }
        }
        Block::Rule => {
            if let Some((_, value)) = info.first() {
                let decor = decoration_prefix(opts.border_color, mode, auto_params, carry);
                line.push_str(&decor);
                let w = text_width(value).max(1);
                for _ in 0..w {
                    line.push('\u{2500}');
                }
                if !decor.is_empty() {
                    line.push_str("\u{1b}[m");
                }
            }
        }
        Block::Head(section) => {
            let name = section_name(*section);
            let decor = decoration_prefix(opts.border_color, mode, auto_params, carry);
            let reset = if decor.is_empty() { "" } else { "\u{1b}[m" };
            line.push_str(&decor);
            line.push('\u{250c}');
            line.push('\u{2500}');
            line.push(' ');
            line.push_str(reset);
            let prefix = label_prefix(opts.style, mode, auto_params, carry);
            line.push_str(&prefix);
            line.push_str(name);
            if !prefix.is_empty() {
                line.push_str("\u{1b}[m");
            }
            line.push_str(&decor);
            line.push(' ');
            let used = 4 + text_width(name);
            if inner_width > used {
                for _ in 0..inner_width - used {
                    line.push('\u{2500}');
                }
            } else {
                for _ in 0..8 {
                    line.push('\u{2500}');
                }
            }
            line.push_str(reset);
        }
        Block::GridTop => {
            let decor = decoration_prefix(opts.border_color, mode, auto_params, carry);
            let reset = if decor.is_empty() { "" } else { "\u{1b}[m" };
            line.push_str(&decor);
            line.push('\u{250c}');
            for _ in 0..max_label + 2 {
                line.push('\u{2500}');
            }
            line.push('\u{252c}');
            for _ in 0..max_value + 2 {
                line.push('\u{2500}');
            }
            line.push('\u{2510}');
            line.push_str(reset);
        }
        Block::GridBottom => {
            let decor = decoration_prefix(opts.border_color, mode, auto_params, carry);
            let reset = if decor.is_empty() { "" } else { "\u{1b}[m" };
            line.push_str(&decor);
            line.push('\u{2514}');
            for _ in 0..max_label + 2 {
                line.push('\u{2500}');
            }
            line.push('\u{2534}');
            for _ in 0..max_value + 2 {
                line.push('\u{2500}');
            }
            line.push('\u{2518}');
            line.push_str(reset);
        }
    }
}

fn render_grid_row(
    line: &mut String,
    index: usize,
    info: &[(String, String)],
    max_label: usize,
    max_value: usize,
    opts: &RenderOptions,
    mode: ColorMode,
    auto_params: Option<&str>,
    carry: &str,
) {
    let (label, value) = &info[index];
    let decor = decoration_prefix(opts.border_color, mode, auto_params, carry);
    let reset = if decor.is_empty() { "" } else { "\u{1b}[m" };
    line.push_str(&decor);
    line.push('\u{2502}');
    line.push_str(reset);
    line.push(' ');
    let prefix = label_prefix(opts.style, mode, auto_params, carry);
    line.push_str(&prefix);
    line.push_str(label);
    if !prefix.is_empty() {
        line.push_str("\u{1b}[m");
    }
    let pad = max_label.saturating_sub(text_width(label));
    for _ in 0..pad {
        line.push(' ');
    }
    line.push(' ');
    line.push_str(&decor);
    line.push('\u{2502}');
    line.push_str(reset);
    line.push(' ');
    line.push_str(value);
    let vpad = max_value.saturating_sub(text_width(value));
    for _ in 0..vpad {
        line.push(' ');
    }
    line.push(' ');
    line.push_str(&decor);
    line.push('\u{2502}');
    line.push_str(reset);
}

#[allow(clippy::too_many_arguments)]
fn render_data(
    line: &mut String,
    index: usize,
    info: &[(String, String)],
    info_rows: usize,
    inner_width: usize,
    max_label: usize,
    opts: &RenderOptions,
    mode: ColorMode,
    auto_params: Option<&str>,
    carry: &str,
) -> usize {
    let (label, value) = &info[index];
    if opts.theme == InfoTheme::Compact {
        let prefix = label_prefix(opts.style, mode, auto_params, carry);
        line.push_str(&prefix);
        line.push_str(label);
        if !prefix.is_empty() {
            line.push_str("\u{1b}[m");
        }
        line.push(' ');
        line.push_str(value);
        return text_width(label) + 1 + text_width(value);
    }
    if opts.theme == InfoTheme::Dotted {
        let prefix = label_prefix(opts.style, mode, auto_params, carry);
        line.push_str(&prefix);
        line.push_str(label);
        if !prefix.is_empty() {
            line.push_str("\u{1b}[m");
        }
        line.push(' ');
        let decor = decoration_prefix(opts.border_color, mode, auto_params, carry);
        line.push_str(&decor);
        let fill = max_label.saturating_sub(text_width(label)) + 4;
        for _ in 0..fill {
            line.push('.');
        }
        if !decor.is_empty() {
            line.push_str("\u{1b}[m");
        }
        line.push(' ');
        line.push_str(value);
        return text_width(label) + 1 + fill + 1 + text_width(value);
    }
    if opts.theme == InfoTheme::Groups {
        let section = section_for_label(label);
        let mut group_last = index + 1 == info_rows;
        if !group_last {
            if let Some((next_label, _)) = info.get(index + 1) {
                group_last = section_for_label(next_label) != section;
            }
        }
        let lead = if group_last {
            "\u{2514}\u{2500} "
        } else {
            "\u{251c}\u{2500} "
        };
        let decor = decoration_prefix(opts.border_color, mode, auto_params, carry);
        line.push_str(&decor);
        line.push_str(lead);
        if !decor.is_empty() {
            line.push_str("\u{1b}[m");
        }
        let prefix = label_prefix(opts.style, mode, auto_params, carry);
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
        return 3 + text_width(label) + 2 + text_width(value);
    }
    if opts.theme == InfoTheme::Outline {
        let decor = decoration_prefix(opts.border_color, mode, auto_params, carry);
        line.push_str(&decor);
        line.push('\u{2502}');
        line.push(' ');
        if !decor.is_empty() {
            line.push_str("\u{1b}[m");
        }
        let prefix = label_prefix(opts.style, mode, auto_params, carry);
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
        return 2 + text_width(label) + 2 + text_width(value);
    }
    if opts.theme == InfoTheme::Title {
        let prefix = label_prefix(opts.style, mode, auto_params, carry);
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
        return text_width(label) + 2 + text_width(value);
    }
    let _ = inner_width;
    let (label, value) = &info[index];
    let lead = opts.theme.lead(index + 1 == info_rows);
    if !lead.is_empty() {
        let decor = decoration_prefix(opts.border_color, mode, auto_params, carry);
        line.push_str(&decor);
        line.push_str(lead);
        if !decor.is_empty() {
            line.push_str("\u{1b}[m");
        }
    }
    let prefix = label_prefix(opts.style, mode, auto_params, carry);
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
    text_width(lead) + text_width(label) + 2 + text_width(value)
}

fn text_width(text: &str) -> usize {
    text.chars().map(char_width).sum()
}

fn color_params(
    color: LabelColor,
    mode: ColorMode,
    auto_params: Option<&str>,
    carry: &str,
) -> Option<String> {
    let params = match color {
        LabelColor::Plain => return None,
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
    transform_params(&params?, mode)
}

fn decoration_prefix(
    color: LabelColor,
    mode: ColorMode,
    auto_params: Option<&str>,
    carry: &str,
) -> String {
    match color_params(color, mode, auto_params, carry) {
        Some(params) => format!("\u{1b}[{params}m"),
        None => String::new(),
    }
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

pub fn parse_align(spec: &str) -> Option<Align> {
    match spec.trim().to_ascii_lowercase().as_str() {
        "top" | "start" | "up" => Some(Align::Top),
        "center" | "centre" | "middle" | "mid" => Some(Align::Center),
        "bottom" | "end" | "down" => Some(Align::Bottom),
        _ => None,
    }
}

pub fn align_spec(align: Align) -> &'static str {
    match align {
        Align::Top => "top",
        Align::Center => "center",
        Align::Bottom => "bottom",
    }
}

pub fn parse_theme(spec: &str) -> Option<InfoTheme> {
    match spec.trim().to_ascii_lowercase().as_str() {
        "plain" | "none" | "off" | "default" => Some(InfoTheme::Plain),
        "tree" | "branch" | "branches" => Some(InfoTheme::Tree),
        "dots" | "dot" | "bullet" | "bullets" => Some(InfoTheme::Dots),
        "bar" | "bars" | "line" => Some(InfoTheme::Bar),
        "arrow" | "arrows" | "caret" => Some(InfoTheme::Arrow),
        "diamond" | "diamonds" | "rhombus" => Some(InfoTheme::Diamond),
        "block" | "blocks" => Some(InfoTheme::Block),
        "pipe" | "pipes" | "rail" => Some(InfoTheme::Pipe),
        "round" | "rounded" | "arc" => Some(InfoTheme::Round),
        "square" | "sharp" | "box" => Some(InfoTheme::Square),
        "double" => Some(InfoTheme::Double),
        "ascii" | "plus" => Some(InfoTheme::Ascii),
        "bold" | "heavy" | "thick" => Some(InfoTheme::Bold),
        "column" | "columns" | "split" | "divider" => Some(InfoTheme::Column),
        "groups" | "group" | "grouped" | "sections" => Some(InfoTheme::Groups),
        "title" | "header" | "neofetch" => Some(InfoTheme::Title),
        "outline" | "outline-groups" | "sections-box" | "headers" => Some(InfoTheme::Outline),
        "dotted" | "dotted-fill" | "filler" | "dots-fill" => Some(InfoTheme::Dotted),
        "grid" | "table" | "cells" => Some(InfoTheme::Grid),
        "compact" | "short" | "bare" | "minimal" => Some(InfoTheme::Compact),
        _ => None,
    }
}

pub fn theme_spec(theme: InfoTheme) -> &'static str {
    match theme {
        InfoTheme::Plain => "plain",
        InfoTheme::Tree => "tree",
        InfoTheme::Dots => "dots",
        InfoTheme::Bar => "bar",
        InfoTheme::Arrow => "arrow",
        InfoTheme::Diamond => "diamond",
        InfoTheme::Block => "block",
        InfoTheme::Pipe => "pipe",
        InfoTheme::Round => "round",
        InfoTheme::Square => "square",
        InfoTheme::Double => "double",
        InfoTheme::Ascii => "ascii",
        InfoTheme::Bold => "bold",
        InfoTheme::Column => "column",
        InfoTheme::Groups => "groups",
        InfoTheme::Title => "title",
        InfoTheme::Outline => "outline",
        InfoTheme::Dotted => "dotted",
        InfoTheme::Grid => "grid",
        InfoTheme::Compact => "compact",
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
    let color = color_params(style.color, mode, auto_params, carry);
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
            align: Align::Top,
            style: LabelStyle {
                color: LabelColor::AutoPerLine,
                font: LabelFont::Bold,
                colon: true,
            },
            theme: InfoTheme::Plain,
            border_color: LabelColor::Plain,
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
            align: Align::Top,
            style: LabelStyle {
                color: LabelColor::Named(36),
                font: LabelFont::Bold,
                colon: true,
            },
            theme: InfoTheme::Plain,
            border_color: LabelColor::Plain,
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
            align: Align::Top,
            style: LabelStyle {
                color: LabelColor::Named(36),
                font: LabelFont::Bold,
                colon: false,
            },
            theme: InfoTheme::Plain,
            border_color: LabelColor::Plain,
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
            align: Align::Top,
            style: LabelStyle {
                color: LabelColor::Auto,
                font: LabelFont::Bold,
                colon: true,
            },
            theme: InfoTheme::Plain,
            border_color: LabelColor::Plain,
        };
        let out = render(&LOGO, &info, false, &opts);
        assert_eq!(out, "xxxxx  Hostname: mac\n");
    }

    #[test]
    fn parses_aligns() {
        assert_eq!(parse_align("top"), Some(Align::Top));
        assert_eq!(parse_align("START"), Some(Align::Top));
        assert_eq!(parse_align("center"), Some(Align::Center));
        assert_eq!(parse_align("centre"), Some(Align::Center));
        assert_eq!(parse_align("middle"), Some(Align::Center));
        assert_eq!(parse_align("bottom"), Some(Align::Bottom));
        assert_eq!(parse_align("end"), Some(Align::Bottom));
        assert_eq!(parse_align("sideways"), None);
        assert_eq!(align_spec(Align::Top), "top");
        assert_eq!(align_spec(Align::Center), "center");
        assert_eq!(align_spec(Align::Bottom), "bottom");
    }

    #[test]
    fn aligns_info_against_logo() {
        static LOGO: LogoEntry = LogoEntry {
            names: &["test"],
            colors: &["\u{1b}[36m"],
            art: &["aaaaa", "bbbbb", "ccccc", "ddddd", "eeeee"],
        };
        let info = vec![("Hostname".to_string(), "mac".to_string())];
        let style = LabelStyle {
            color: LabelColor::Auto,
            font: LabelFont::Bold,
            colon: true,
        };
        let expected = |index: usize| {
            let mut lines = ["aaaaa", "bbbbb", "ccccc", "ddddd", "eeeee"];
            lines[index] = match index {
                0 => "aaaaa  Hostname: mac",
                2 => "ccccc  Hostname: mac",
                4 => "eeeee  Hostname: mac",
                _ => lines[index],
            };
            let mut out = lines.join("\n");
            out.push('\n');
            out
        };
        for (align, index) in [(Align::Top, 0), (Align::Center, 2), (Align::Bottom, 4)] {
            let opts = RenderOptions {
                mode: ColorMode::Plain,
                align,
                style,
                theme: InfoTheme::Plain,
                border_color: LabelColor::Plain,
            };
            assert_eq!(render(&LOGO, &info, false, &opts), expected(index));
        }
    }

    #[test]
    fn align_ignores_taller_info() {
        static LOGO: LogoEntry = LogoEntry {
            names: &["test"],
            colors: &["\u{1b}[36m"],
            art: &["aaaaa"],
        };
        let info = vec![
            ("Hostname".to_string(), "mac".to_string()),
            ("OS".to_string(), "test".to_string()),
        ];
        let opts = RenderOptions {
            mode: ColorMode::Plain,
            align: Align::Bottom,
            style: LabelStyle {
                color: LabelColor::Auto,
                font: LabelFont::Bold,
                colon: true,
            },
            theme: InfoTheme::Plain,
            border_color: LabelColor::Plain,
        };
        let out = render(&LOGO, &info, false, &opts);
        assert_eq!(out, "aaaaa  Hostname: mac\n       OS: test\n");
    }

    #[test]
    fn parses_themes() {
        assert_eq!(parse_theme("plain"), Some(InfoTheme::Plain));
        assert_eq!(parse_theme("TREE"), Some(InfoTheme::Tree));
        assert_eq!(parse_theme("dots"), Some(InfoTheme::Dots));
        assert_eq!(parse_theme("bar"), Some(InfoTheme::Bar));
        assert_eq!(parse_theme("arrow"), Some(InfoTheme::Arrow));
        assert_eq!(parse_theme("diamond"), Some(InfoTheme::Diamond));
        assert_eq!(parse_theme("block"), Some(InfoTheme::Block));
        assert_eq!(parse_theme("pipe"), Some(InfoTheme::Pipe));
        assert_eq!(parse_theme("rounded"), Some(InfoTheme::Round));
        assert_eq!(parse_theme("square"), Some(InfoTheme::Square));
        assert_eq!(parse_theme("double"), Some(InfoTheme::Double));
        assert_eq!(parse_theme("ascii"), Some(InfoTheme::Ascii));
        assert_eq!(parse_theme("bold"), Some(InfoTheme::Bold));
        assert_eq!(parse_theme("column"), Some(InfoTheme::Column));
        assert_eq!(parse_theme("groups"), Some(InfoTheme::Groups));
        assert_eq!(parse_theme("title"), Some(InfoTheme::Title));
        assert_eq!(parse_theme("outline"), Some(InfoTheme::Outline));
        assert_eq!(parse_theme("dotted"), Some(InfoTheme::Dotted));
        assert_eq!(parse_theme("grid"), Some(InfoTheme::Grid));
        assert_eq!(parse_theme("compact"), Some(InfoTheme::Compact));
        assert_eq!(parse_theme("nope"), None);
        assert_eq!(theme_spec(InfoTheme::Plain), "plain");
        assert_eq!(theme_spec(InfoTheme::Tree), "tree");
        assert_eq!(theme_spec(InfoTheme::Ascii), "ascii");
        assert_eq!(theme_spec(InfoTheme::Column), "column");
        assert_eq!(theme_spec(InfoTheme::Groups), "groups");
        assert_eq!(theme_spec(InfoTheme::Title), "title");
        assert_eq!(theme_spec(InfoTheme::Outline), "outline");
        assert_eq!(theme_spec(InfoTheme::Dotted), "dotted");
        assert_eq!(theme_spec(InfoTheme::Grid), "grid");
        assert_eq!(theme_spec(InfoTheme::Compact), "compact");
    }

    #[test]
    fn column_theme_draws_a_divider() {
        static LOGO: LogoEntry = LogoEntry {
            names: &["test"],
            colors: &["\u{1b}[36m"],
            art: &["xxxxx"],
        };
        let info = vec![("OS".to_string(), "mac".to_string())];
        let opts = RenderOptions {
            mode: ColorMode::Plain,
            align: Align::Top,
            style: LabelStyle {
                color: LabelColor::Auto,
                font: LabelFont::Bold,
                colon: true,
            },
            theme: InfoTheme::Column,
            border_color: LabelColor::Plain,
        };
        let out = render(&LOGO, &info, false, &opts);
        assert_eq!(out, "xxxxx \u{2502} OS: mac\n");
    }

    #[test]
    fn glyph_lead_themes_prefix_rows() {
        static LOGO: LogoEntry = LogoEntry {
            names: &["test"],
            colors: &["\u{1b}[36m"],
            art: &["xxxxx"],
        };
        let info = vec![("OS".to_string(), "mac".to_string())];
        let style = LabelStyle {
            color: LabelColor::Auto,
            font: LabelFont::Bold,
            colon: true,
        };
        for (theme, glyph) in [
            (InfoTheme::Arrow, "\u{2192}"),
            (InfoTheme::Diamond, "\u{25c6}"),
            (InfoTheme::Block, "\u{2588}"),
            (InfoTheme::Pipe, "\u{2502}"),
        ] {
            let opts = RenderOptions {
                mode: ColorMode::Plain,
                align: Align::Top,
                style,
                theme,
                border_color: LabelColor::Plain,
            };
            let out = render(&LOGO, &info, false, &opts);
            assert_eq!(out, format!("xxxxx  {glyph} OS: mac\n"));
        }
    }

    #[test]
    fn rounds_info_into_a_box() {
        static LOGO: LogoEntry = LogoEntry {
            names: &["test"],
            colors: &["\u{1b}[36m"],
            art: &["xxxxx"],
        };
        let info = vec![
            ("OS".to_string(), "mac".to_string()),
            ("Shell".to_string(), "zsh".to_string()),
        ];
        let opts = RenderOptions {
            mode: ColorMode::Plain,
            align: Align::Top,
            style: LabelStyle {
                color: LabelColor::Auto,
                font: LabelFont::Bold,
                colon: true,
            },
            theme: InfoTheme::Round,
            border_color: LabelColor::Plain,
        };
        let out = render(&LOGO, &info, false, &opts);
        let bar = "\u{2500}".repeat(12);
        let expected = format!(
            "xxxxx  \u{256d}{bar}\u{256e}\n       \u{2502} OS: mac    \u{2502}\n       \u{2502} Shell: zsh \u{2502}\n       \u{2570}{bar}\u{256f}\n"
        );
        assert_eq!(out, expected);
    }

    #[test]
    fn tree_marks_the_last_row() {
        static LOGO: LogoEntry = LogoEntry {
            names: &["test"],
            colors: &["\u{1b}[36m"],
            art: &["xxxxx"],
        };
        let info = vec![
            ("OS".to_string(), "mac".to_string()),
            ("Shell".to_string(), "zsh".to_string()),
        ];
        let opts = RenderOptions {
            mode: ColorMode::Plain,
            align: Align::Top,
            style: LabelStyle {
                color: LabelColor::Auto,
                font: LabelFont::Bold,
                colon: true,
            },
            theme: InfoTheme::Tree,
            border_color: LabelColor::Plain,
        };
        let out = render(&LOGO, &info, false, &opts);
        assert_eq!(out, "xxxxx  \u{251c}\u{2500} OS: mac\n       \u{2570}\u{2500} Shell: zsh\n");
    }

    #[test]
    fn bar_prefixes_every_row() {
        static LOGO: LogoEntry = LogoEntry {
            names: &["test"],
            colors: &["\u{1b}[36m"],
            art: &["xxxxx"],
        };
        let info = vec![
            ("OS".to_string(), "mac".to_string()),
            ("Shell".to_string(), "zsh".to_string()),
        ];
        let opts = RenderOptions {
            mode: ColorMode::Plain,
            align: Align::Top,
            style: LabelStyle {
                color: LabelColor::Auto,
                font: LabelFont::Bold,
                colon: true,
            },
            theme: InfoTheme::Bar,
            border_color: LabelColor::Plain,
        };
        let out = render(&LOGO, &info, false, &opts);
        assert_eq!(out, "xxxxx  \u{258c} OS: mac\n       \u{258c} Shell: zsh\n");
    }

    #[test]
    fn border_color_styles_the_frame() {
        static LOGO: LogoEntry = LogoEntry {
            names: &["test"],
            colors: &["\u{1b}[36m"],
            art: &["xxxxx"],
        };
        let info = vec![("OS".to_string(), "mac".to_string())];
        let opts = RenderOptions {
            mode: ColorMode::Normal,
            align: Align::Top,
            style: LabelStyle {
                color: LabelColor::Plain,
                font: LabelFont::Plain,
                colon: true,
            },
            theme: InfoTheme::Round,
            border_color: LabelColor::Named(31),
        };
        let out = render(&LOGO, &info, true, &opts);
        assert!(out.contains("\u{1b}[31m\u{256d}"));
        assert!(out.contains("\u{1b}[31m\u{2502}"));
        assert!(out.contains("\u{1b}[31m\u{2570}"));
    }

    #[test]
    fn decoration_color_overrides_prefix() {
        static LOGO: LogoEntry = LogoEntry {
            names: &["test"],
            colors: &["\u{1b}[36m"],
            art: &["xxxxx"],
        };
        let info = vec![
            ("OS".to_string(), "mac".to_string()),
            ("Shell".to_string(), "zsh".to_string()),
        ];
        let opts = RenderOptions {
            mode: ColorMode::Normal,
            align: Align::Top,
            style: LabelStyle {
                color: LabelColor::Plain,
                font: LabelFont::Plain,
                colon: true,
            },
            theme: InfoTheme::Tree,
            border_color: LabelColor::Named(31),
        };
        let out = render(&LOGO, &info, true, &opts);
        assert!(out.contains("\u{1b}[31m\u{251c}\u{2500} \u{1b}[mOS: mac"));
        assert!(out.contains("\u{1b}[31m\u{2570}\u{2500} \u{1b}[mShell: zsh"));
    }

    #[test]
    fn border_color_can_track_the_logo_per_line() {
        static LOGO: LogoEntry = LogoEntry {
            names: &["test"],
            colors: &["\u{1b}[36m", "\u{1b}[31m"],
            art: &["$2xx", "yy"],
        };
        let info = vec![
            ("OS".to_string(), "mac".to_string()),
            ("Shell".to_string(), "zsh".to_string()),
        ];
        let opts = RenderOptions {
            mode: ColorMode::Normal,
            align: Align::Top,
            style: LabelStyle {
                color: LabelColor::Plain,
                font: LabelFont::Plain,
                colon: true,
            },
            theme: InfoTheme::Round,
            border_color: LabelColor::AutoPerLine,
        };
        let out = render(&LOGO, &info, true, &opts);
        assert!(out.contains("\u{1b}[31m\u{256d}"));
        assert!(out.contains("\u{1b}[31m\u{2502}"));
    }

    #[test]
    fn compact_drops_the_colon() {
        static LOGO: LogoEntry = LogoEntry {
            names: &["test"],
            colors: &["\u{1b}[36m"],
            art: &["xxxxx"],
        };
        let info = vec![("OS".to_string(), "mac".to_string())];
        let opts = RenderOptions {
            mode: ColorMode::Plain,
            align: Align::Top,
            style: LabelStyle {
                color: LabelColor::Auto,
                font: LabelFont::Bold,
                colon: true,
            },
            theme: InfoTheme::Compact,
            border_color: LabelColor::Plain,
        };
        let out = render(&LOGO, &info, false, &opts);
        assert_eq!(out, "xxxxx  OS mac\n");
    }

    #[test]
    fn dotted_fills_to_the_value_column() {
        static LOGO: LogoEntry = LogoEntry {
            names: &["test"],
            colors: &["\u{1b}[36m"],
            art: &["xxxxx"],
        };
        let info = vec![
            ("Hostname".to_string(), "mac".to_string()),
            ("OS".to_string(), "macOS".to_string()),
        ];
        let opts = RenderOptions {
            mode: ColorMode::Plain,
            align: Align::Top,
            style: LabelStyle {
                color: LabelColor::Auto,
                font: LabelFont::Bold,
                colon: true,
            },
            theme: InfoTheme::Dotted,
            border_color: LabelColor::Plain,
        };
        let out = render(&LOGO, &info, false, &opts);
        assert!(out.contains("Hostname .... mac"));
        assert!(out.contains("OS .......... macOS"));
    }

    #[test]
    fn groups_splits_sections() {
        static LOGO: LogoEntry = LogoEntry {
            names: &["test"],
            colors: &["\u{1b}[36m"],
            art: &["xxxxx"],
        };
        let info = vec![
            ("OS".to_string(), "macOS".to_string()),
            ("Shell".to_string(), "zsh".to_string()),
            ("CPU".to_string(), "M5".to_string()),
        ];
        let opts = RenderOptions {
            mode: ColorMode::Plain,
            align: Align::Top,
            style: LabelStyle {
                color: LabelColor::Auto,
                font: LabelFont::Bold,
                colon: true,
            },
            theme: InfoTheme::Groups,
            border_color: LabelColor::Plain,
        };
        let out = render(&LOGO, &info, false, &opts);
        assert!(out.contains("\u{2514}\u{2500} OS: macOS"));
        assert!(out.contains("\u{2502}\n"));
        assert!(out.contains("\u{2514}\u{2500} CPU: M5"));
    }

    #[test]
    fn title_uses_hostname_as_header() {
        static LOGO: LogoEntry = LogoEntry {
            names: &["test"],
            colors: &["\u{1b}[36m"],
            art: &["xxxxx"],
        };
        let info = vec![
            ("Hostname".to_string(), "mac".to_string()),
            ("OS".to_string(), "macOS".to_string()),
        ];
        let opts = RenderOptions {
            mode: ColorMode::Plain,
            align: Align::Top,
            style: LabelStyle {
                color: LabelColor::Auto,
                font: LabelFont::Bold,
                colon: true,
            },
            theme: InfoTheme::Title,
            border_color: LabelColor::Plain,
        };
        let out = render(&LOGO, &info, false, &opts);
        assert!(out.contains("xxxxx  mac\n"));
        assert!(out.contains("\u{2500}"));
        assert!(out.contains("OS: macOS"));
        assert!(!out.contains("Hostname: mac"));
    }

    #[test]
    fn outline_draws_section_heads() {
        static LOGO: LogoEntry = LogoEntry {
            names: &["test"],
            colors: &["\u{1b}[36m"],
            art: &["xxxxx"],
        };
        let info = vec![
            ("OS".to_string(), "macOS".to_string()),
            ("Shell".to_string(), "zsh".to_string()),
        ];
        let opts = RenderOptions {
            mode: ColorMode::Plain,
            align: Align::Top,
            style: LabelStyle {
                color: LabelColor::Auto,
                font: LabelFont::Bold,
                colon: true,
            },
            theme: InfoTheme::Outline,
            border_color: LabelColor::Plain,
        };
        let out = render(&LOGO, &info, false, &opts);
        assert!(out.contains("System"));
        assert!(out.contains("Software"));
        assert!(out.contains("\u{2502} OS: macOS"));
        assert!(out.contains("\u{2502} Shell: zsh"));
    }

    #[test]
    fn grid_draws_a_table() {
        static LOGO: LogoEntry = LogoEntry {
            names: &["test"],
            colors: &["\u{1b}[36m"],
            art: &["xxxxx"],
        };
        let info = vec![("OS".to_string(), "mac".to_string())];
        let opts = RenderOptions {
            mode: ColorMode::Plain,
            align: Align::Top,
            style: LabelStyle {
                color: LabelColor::Auto,
                font: LabelFont::Bold,
                colon: true,
            },
            theme: InfoTheme::Grid,
            border_color: LabelColor::Plain,
        };
        let out = render(&LOGO, &info, false, &opts);
        assert!(out.contains("\u{250c}"));
        assert!(out.contains("\u{252c}"));
        assert!(out.contains("\u{2502} OS \u{2502} mac \u{2502}"));
        assert!(out.contains("\u{2514}"));
    }
}
