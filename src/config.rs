use std::path::PathBuf;

use crate::style::{
    LabelColor, LabelFont, bool_spec, color_spec, font_spec, parse_bool, parse_color, parse_font,
};

pub struct Config {
    pub label_color: Option<LabelColor>,
    pub label_font: Option<LabelFont>,
    pub label_colon: Option<bool>,
}

pub fn load() -> Config {
    let empty = Config {
        label_color: None,
        label_font: None,
        label_colon: None,
    };
    let Some(path) = config_path() else {
        return empty;
    };
    let Ok(content) = std::fs::read_to_string(&path) else {
        return empty;
    };
    match parse(&content) {
        Ok(config) => config,
        Err(err) => {
            eprintln!("wfetch: config error in {}: {err}", path.display());
            empty
        }
    }
}

pub fn save(style: &crate::style::LabelStyle) -> Result<PathBuf, String> {
    let path = config_path().ok_or_else(|| "could not resolve config directory".to_string())?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("could not create {}: {e}", parent.display()))?;
    }
    let content = format!(
        "[label]\ncolor = {}\nfont = {}\ncolon = {}\n",
        color_spec(style.color),
        font_spec(style.font),
        bool_spec(style.colon)
    );
    std::fs::write(&path, content)
        .map_err(|e| format!("could not write {}: {e}", path.display()))?;
    Ok(path)
}

pub fn config_path() -> Option<PathBuf> {
    if let Some(xdg) = std::env::var_os("XDG_CONFIG_HOME")
        && !xdg.is_empty()
    {
        return Some(PathBuf::from(xdg).join("wfetch").join("config"));
    }
    let home = std::env::var_os("HOME")?;
    if home.is_empty() {
        return None;
    }
    Some(
        PathBuf::from(home)
            .join(".config")
            .join("wfetch")
            .join("config"),
    )
}

fn parse(content: &str) -> Result<Config, String> {
    let mut config = Config {
        label_color: None,
        label_font: None,
        label_colon: None,
    };
    let mut section: Option<String> = None;
    for (index, raw) in content.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(rest) = line.strip_prefix('[') {
            match rest.strip_suffix(']') {
                Some(name) => {
                    let name = name.trim().to_ascii_lowercase();
                    if name != "label" {
                        return Err(format!("unknown section [{name}] on line {}", index + 1));
                    }
                    section = Some(name);
                }
                None => return Err(format!("malformed section on line {}", index + 1)),
            }
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            return Err(format!("expected key = value on line {}", index + 1));
        };
        let key = key.trim().to_ascii_lowercase();
        let value = value.trim();
        match section.as_deref() {
            Some("label") => match key.as_str() {
                "color" => match parse_color(value) {
                    Some(color) => config.label_color = Some(color),
                    None => {
                        return Err(format!("invalid color {value:?} on line {}", index + 1));
                    }
                },
                "font" => match parse_font(value) {
                    Some(font) => config.label_font = Some(font),
                    None => {
                        return Err(format!("invalid font {value:?} on line {}", index + 1));
                    }
                },
                "colon" => match parse_bool(value) {
                    Some(colon) => config.label_colon = Some(colon),
                    None => {
                        return Err(format!("invalid colon {value:?} on line {}", index + 1));
                    }
                },
                _ => return Err(format!("unknown key {key:?} on line {}", index + 1)),
            },
            _ => return Err(format!("key outside of [label] on line {}", index + 1)),
        }
    }
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::style::LabelColor;

    #[test]
    fn parses_colon_key() {
        let config = parse("[label]\ncolon = no\n").unwrap();
        assert_eq!(config.label_colon, Some(false));
        let config = parse("[label]\ncolon = yes\n").unwrap();
        assert_eq!(config.label_colon, Some(true));
    }

    #[test]
    fn rejects_bad_colon() {
        assert!(parse("[label]\ncolon = maybe\n").is_err());
    }

    #[test]
    fn parses_label_section() {
        let config = parse("[label]\ncolor = auto\nfont = bold\n").unwrap();
        assert_eq!(config.label_color, Some(LabelColor::Auto));
        assert_eq!(config.label_font, Some(crate::style::LabelFont::Bold));
    }

    #[test]
    fn ignores_comments_and_blanks() {
        let config = parse("# hi\n\n[label]\n  color = cyan  \nfont = italic\n").unwrap();
        assert_eq!(config.label_color, Some(LabelColor::Named(36)));
        assert_eq!(config.label_font, Some(crate::style::LabelFont::Italic));
    }

    #[test]
    fn rejects_unknown_section() {
        assert!(parse("[logo]\ncolor = auto\n").is_err());
    }

    #[test]
    fn rejects_unknown_key() {
        assert!(parse("[label]\ncolour = red\n").is_err());
    }

    #[test]
    fn rejects_keys_outside_section() {
        assert!(parse("color = auto\n").is_err());
    }

    #[test]
    fn rejects_bad_values() {
        assert!(parse("[label]\ncolor = nope\n").is_err());
        assert!(parse("[label]\nfont = nope\n").is_err());
        assert!(parse("[label\ncolor = auto\n").is_err());
        assert!(parse("[label]\ncolor auto\n").is_err());
    }
}
