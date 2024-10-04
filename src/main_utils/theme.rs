use owo_colors::AnsiColors;

use super::colored::{Color, OptColor};

pub struct DiffTheme {
    pub removed: Color,
    pub added: Color,
    pub modified: Color,
    pub line_a: Color,
    pub line_b: Color,
}

pub struct Theme {
    pub status_header: Color,
    pub status_up_to_date: Color,
    pub status_missing_from_both: Color,
    pub status_local_added: Color,
    pub status_logix_added: Color,
    pub status_modified: Color,
    pub status_error: Color,
    pub local_file: OptColor,
    pub logix_file: OptColor,
    pub logix_root: Color,
    pub owner_builtin: Color,
    pub owner_package: Color,
    pub package_version_none: Color,
    pub package_version_date: Color,
    pub diff: DiffTheme,
}

impl Theme {
    pub fn default_term() -> Self {
        let _white = Color::ansi(AnsiColors::White);
        let dimmed_white = Color::rgb(154, 163, 173);
        let red = Color::rgb(245, 66, 66);
        let yellow = Color::rgb(237, 193, 62);

        let bright_blue = Color::rgb(77, 157, 255);
        //let blue = Color::rgb(100, 129, 245);
        let white_blue = Color::rgb(189, 222, 255);
        let dimmed_blue = Color::rgb(109, 122, 179);

        let bright_green = Color::rgb(79, 247, 99);
        let white_green = Color::rgb(166, 237, 185);
        let dimmed_green = Color::rgb(113, 166, 127);
        let green = Color::rgb(37, 133, 11);
        //let dimmed_green = Color::rgb(66, 97, 57);

        Self {
            status_header: dimmed_white,
            status_up_to_date: green,
            status_missing_from_both: red,
            status_local_added: yellow,
            status_logix_added: red,
            status_modified: yellow,
            status_error: red,
            local_file: OptColor {
                is_some: white_blue,
                is_none: dimmed_blue,
            },
            logix_file: OptColor {
                is_some: white_green,
                is_none: dimmed_green,
            },
            logix_root: dimmed_white,
            owner_builtin: bright_blue,
            owner_package: bright_green,
            package_version_none: dimmed_blue,
            package_version_date: white_blue,
            diff: DiffTheme {
                removed: red,
                added: green,
                modified: yellow,
                line_a: bright_blue,
                line_b: white_blue,
            },
        }
    }
}
