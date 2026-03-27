use crate::{ColorId, TraceError, TraceResult, DEFAULT_COLOR};

const COLOR_NAMES: &[(&str, ColorId)] = &[
    ("Default", DEFAULT_COLOR),
    ("default", DEFAULT_COLOR),
    ("Black", 0),
    ("Red", 1),
    ("Green", 2),
    ("Yellow", 3),
    ("Blue", 4),
    ("Magenta", 5),
    ("Cyan", 6),
    ("White", 7),
    ("BrightBlue", 12),
    ("BrightCyan", 14),
    ("BrightGreen", 10),
    ("BrightMagenta", 13),
    ("BrightWhite", 15),
    ("BrightYellow", 11),
    ("DeepSkyBlue1", 39),
    ("Gold3", 142),
    ("Orange3", 172),
];

pub fn available_color_names() -> Vec<&'static str> {
    let mut names = COLOR_NAMES
        .iter()
        .filter_map(|(name, color)| if *color == DEFAULT_COLOR && *name == "default" { None } else { Some(*name) })
        .collect::<Vec<_>>();
    names.sort_unstable();
    names.dedup();
    names
}

pub fn color(color_name: impl AsRef<str>) -> TraceResult<ColorId> {
    let token = color_name.as_ref().trim();
    if token.is_empty() {
        return Err(TraceError::new("trace color name must not be empty"));
    }

    COLOR_NAMES
        .iter()
        .find(|(name, _)| *name == token)
        .map(|(_, color_id)| *color_id)
        .ok_or_else(|| TraceError::new(format!("unknown trace color '{token}'")))
}
