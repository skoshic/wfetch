use std::fs;
use std::path::PathBuf;

fn ansi_fragment(macro_name: &str, suffix: Option<&str>) -> String {
    let base = match macro_name {
        "FF_COLOR_FG_BLACK" => "30",
        "FF_COLOR_FG_RED" => "31",
        "FF_COLOR_FG_GREEN" => "32",
        "FF_COLOR_FG_YELLOW" => "33",
        "FF_COLOR_FG_BLUE" => "34",
        "FF_COLOR_FG_MAGENTA" => "35",
        "FF_COLOR_FG_CYAN" => "36",
        "FF_COLOR_FG_WHITE" => "37",
        "FF_COLOR_FG_DEFAULT" => "39",
        "FF_COLOR_FG_LIGHT_BLACK" => "90",
        "FF_COLOR_FG_LIGHT_RED" => "91",
        "FF_COLOR_FG_LIGHT_GREEN" => "92",
        "FF_COLOR_FG_LIGHT_YELLOW" => "93",
        "FF_COLOR_FG_LIGHT_BLUE" => "94",
        "FF_COLOR_FG_LIGHT_MAGENTA" => "95",
        "FF_COLOR_FG_LIGHT_CYAN" => "96",
        "FF_COLOR_FG_LIGHT_WHITE" => "97",
        "FF_COLOR_BG_BLACK" => "40",
        "FF_COLOR_BG_RED" => "41",
        "FF_COLOR_BG_GREEN" => "42",
        "FF_COLOR_BG_YELLOW" => "43",
        "FF_COLOR_BG_BLUE" => "44",
        "FF_COLOR_BG_MAGENTA" => "45",
        "FF_COLOR_BG_CYAN" => "46",
        "FF_COLOR_BG_WHITE" => "47",
        "FF_COLOR_BG_DEFAULT" => "49",
        "FF_COLOR_BG_LIGHT_BLACK" => "100",
        "FF_COLOR_BG_LIGHT_RED" => "101",
        "FF_COLOR_BG_LIGHT_GREEN" => "102",
        "FF_COLOR_BG_LIGHT_YELLOW" => "103",
        "FF_COLOR_BG_LIGHT_BLUE" => "104",
        "FF_COLOR_BG_LIGHT_MAGENTA" => "105",
        "FF_COLOR_BG_LIGHT_CYAN" => "106",
        "FF_COLOR_BG_LIGHT_WHITE" => "107",
        "FF_COLOR_FG_256" => "38;5;",
        "FF_COLOR_BG_256" => "48;5;",
        "FF_COLOR_FG_RGB" => "38;2;",
        "FF_COLOR_BG_RGB" => "48;2;",
        _ => panic!("wfetch build: unknown color macro {macro_name}"),
    };
    match suffix {
        Some(code) => format!("\x1b[{base}{code}m"),
        None => format!("\x1b[{base}m"),
    }
}

fn quoted_strings(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '"' {
            let mut current = String::new();
            for c in chars.by_ref() {
                if c == '"' {
                    break;
                }
                current.push(c);
            }
            out.push(current);
        }
    }
    out
}

struct RawEntry {
    art_macro: String,
    names: Vec<String>,
    colors: Vec<String>,
    normal: bool,
}

fn parse_inc(text: &str) -> Vec<RawEntry> {
    let mut entries = Vec::new();
    let mut search = 0;
    while let Some(start) = text[search..].find(".names = {") {
        let block_start = search + start;
        let next = text[block_start + 1..]
            .find(".names = {")
            .map(|i| block_start + 1 + i)
            .unwrap_or(text.len());
        let block = &text[block_start..next];
        let names_start = block.find(".names = {").unwrap() + ".names = {".len();
        let names_end = block[names_start..].find('}').unwrap() + names_start;
        let names = quoted_strings(&block[names_start..names_end]);
        let mut colors = Vec::new();
        if let Some(colors_start) = block.find(".colors = {") {
            let inner = colors_start + ".colors = {".len();
            let inner_end = block[inner..].find('}').unwrap() + inner;
            for token in block[inner..inner_end].split(',') {
                let token = token.trim();
                if !token.starts_with("FF_COLOR_") {
                    continue;
                }
                let end = token.find([' ', '\t']).unwrap_or(token.len());
                let suffix = quoted_strings(&token[end..]).into_iter().next();
                colors.push(ansi_fragment(&token[..end], suffix.as_deref()));
            }
        }
        let art_macro = block
            .find("FASTFETCH_DATATEXT_LOGO_")
            .map(|i| {
                let rest = &block[i + "FASTFETCH_DATATEXT_LOGO_".len()..];
                rest.chars()
                    .take_while(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || *c == '_')
                    .collect::<String>()
            })
            .unwrap_or_default();
        let normal = !block.contains(".type =");
        entries.push(RawEntry {
            art_macro,
            names,
            colors,
            normal,
        });
        search = next;
    }
    entries
}

fn collect_art_files(root: &std::path::Path, out: &mut Vec<PathBuf>) {
    let entries = match fs::read_dir(root) {
        Ok(entries) => entries,
        Err(_) => return,
    };
    let mut names: Vec<PathBuf> = entries
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .collect();
    names.sort();
    for path in names {
        if path.is_dir() {
            collect_art_files(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "txt") {
            out.push(path);
        }
    }
}

fn rust_literal(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 2);
    out.push('"');
    for c in text.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            _ => out.push(c),
        }
    }
    out.push('"');
    out
}

fn main() {
    let root = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let logos = root.join("assets").join("logos");
    let out = PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("logos.rs");

    let mut art_files = Vec::new();
    collect_art_files(&logos, &mut art_files);
    let mut art_by_name = std::collections::HashMap::new();
    for path in art_files {
        if let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) {
            art_by_name.insert(stem.to_lowercase(), path);
        }
    }

    let mut letters: Vec<String> = fs::read_dir(&logos)
        .unwrap()
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .filter(|name| name.len() == 1)
        .collect();
    letters.sort();

    let mut generated = String::from(
        "pub(crate) struct LogoEntry {\n    pub(crate) names: &'static [&'static str],\n    pub(crate) colors: &'static [&'static str],\n    pub(crate) art: &'static [&'static str],\n}\n\npub(crate) static LOGOS: &[LogoEntry] = &[\n",
    );
    let mut count = 0;
    for letter in &letters {
        let inc = logos.join(format!("{letter}.inc"));
        if !inc.is_file() {
            continue;
        }
        println!("cargo::rerun-if-changed={}", inc.display());
        let text = fs::read_to_string(&inc).unwrap();
        for entry in parse_inc(&text) {
            if !entry.normal || entry.art_macro.is_empty() || entry.names.is_empty() {
                continue;
            }
            let key = entry.art_macro.to_lowercase();
            let file = match art_by_name.get(&key) {
                Some(path) => path.clone(),
                None => panic!("wfetch build: missing art file for {key}"),
            };
            println!("cargo::rerun-if-changed={}", file.display());
            let art = fs::read_to_string(&file).unwrap().replace("\r\n", "\n");
            let mut lines: Vec<&str> = art.split('\n').collect();
            if lines.last() == Some(&"") {
                lines.pop();
            }
            generated.push_str("    LogoEntry {\n        names: &[");
            for name in &entry.names {
                generated.push_str(&rust_literal(name));
                generated.push_str(", ");
            }
            generated.push_str("],\n        colors: &[");
            for color in &entry.colors {
                generated.push_str(&rust_literal(color));
                generated.push_str(", ");
            }
            generated.push_str("],\n        art: &[");
            for line in &lines {
                generated.push_str(&rust_literal(line));
                generated.push_str(", ");
            }
            generated.push_str("],\n    },\n");
            count += 1;
        }
    }

    let unknown = logos.join("_").join("unknown.txt");
    println!("cargo::rerun-if-changed={}", unknown.display());
    let art = fs::read_to_string(&unknown).unwrap().replace("\r\n", "\n");
    let mut lines: Vec<&str> = art.split('\n').collect();
    if lines.last() == Some(&"") {
        lines.pop();
    }
    generated.push_str("];\n\npub(crate) static LOGO_UNKNOWN: LogoEntry = LogoEntry {\n    names: &[\"unknown\"],\n    colors: &[\"\\x1b[39m\"],\n    art: &[");
    for line in &lines {
        generated.push_str(&rust_literal(line));
        generated.push_str(", ");
    }
    generated.push_str("],\n};\n");

    fs::write(&out, generated).unwrap();
    println!("cargo::warning=wfetch: embedded {count} logos plus fallback");
}
