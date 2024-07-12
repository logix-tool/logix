use std::fmt;

use logix::{
    based_path::BasedPath,
    managed_file::{FileStatus, Owner},
    managed_package::PackageVersion,
};
use owo_colors::{AnsiColors, DynColor, OwoColorize};
use time::macros::format_description;

use super::theme::Theme;

#[derive(Clone, Copy)]
pub enum Color {
    Ansi(AnsiColors),
    Rgb(owo_colors::Rgb),
}

impl Color {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self::Rgb(owo_colors::Rgb(r, g, b))
    }

    pub const fn ansi(color: AnsiColors) -> Self {
        Self::Ansi(color)
    }
}

impl DynColor for Color {
    fn fmt_ansi_fg(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Ansi(color) => color.fmt_ansi_fg(f),
            Self::Rgb(color) => color.fmt_ansi_fg(f),
        }
    }

    fn fmt_ansi_bg(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Ansi(color) => color.fmt_ansi_bg(f),
            Self::Rgb(color) => color.fmt_ansi_bg(f),
        }
    }

    fn fmt_raw_ansi_fg(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Ansi(color) => color.fmt_raw_ansi_fg(f),
            Self::Rgb(color) => color.fmt_raw_ansi_fg(f),
        }
    }

    fn fmt_raw_ansi_bg(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Ansi(color) => color.fmt_raw_ansi_bg(f),
            Self::Rgb(color) => color.fmt_raw_ansi_bg(f),
        }
    }

    fn get_dyncolors_fg(&self) -> owo_colors::DynColors {
        match self {
            Self::Ansi(color) => color.get_dyncolors_fg(),
            Self::Rgb(color) => color.get_dyncolors_fg(),
        }
    }

    fn get_dyncolors_bg(&self) -> owo_colors::DynColors {
        match self {
            Self::Ansi(color) => color.get_dyncolors_bg(),
            Self::Rgb(color) => color.get_dyncolors_bg(),
        }
    }
}

pub struct OptColor {
    pub is_some: Color,
    pub is_none: Color,
}

pub fn status(status: FileStatus, theme: &Theme) -> impl fmt::Display {
    // TODO: The state display names need a better naming convention
    match status {
        FileStatus::UpToDate => "Up to date".color(theme.status_up_to_date),
        FileStatus::MissingFromBoth => "Missing from both".color(theme.status_missing_from_both),
        FileStatus::LocalAdded => "Missing logix".color(theme.status_local_added),
        FileStatus::LogixAdded => "Missing local".color(theme.status_logix_added),
        FileStatus::Modified => "Has changes".color(theme.status_modified),
        FileStatus::ErrorReadingLocal(_) => "Local error".color(theme.status_error),
        FileStatus::ErrorReadingLogix(_) => "Logix error".color(theme.status_error),
    }
}

pub fn path<'a>(path: Option<&'a BasedPath>, color: &'a OptColor) -> impl fmt::Display + 'a {
    struct ColoredPath<'a> {
        path: Option<&'a BasedPath>,
        color: &'a OptColor,
    }

    impl<'a> fmt::Display for ColoredPath<'a> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self.path {
                Some(path) => {
                    fmt::Display::fmt(&path.rel_path().display().color(self.color.is_some), f)
                }
                None => fmt::Display::fmt(&"<none>".color(self.color.is_none), f),
            }
        }
    }

    ColoredPath { path, color }
}

pub fn owner<'a>(owner: &'a Owner, theme: &'a Theme) -> impl fmt::Display + 'a {
    struct ColoredOwner<'a> {
        owner: &'a Owner,
        theme: &'a Theme,
    }

    impl<'a> fmt::Display for ColoredOwner<'a> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self.owner {
                Owner::Ssh => fmt::Display::fmt(&"ssh".color(self.theme.owner_builtin), f),
                Owner::Shell => fmt::Display::fmt(&"shell".color(self.theme.owner_builtin), f),
                Owner::Package(name) => fmt::Display::fmt(&name.color(self.theme.owner_package), f),
            }
        }
    }

    ColoredOwner { owner, theme }
}

pub fn package_version<'a>(
    version: &'a PackageVersion,
    theme: &'a Theme,
) -> impl fmt::Display + 'a {
    struct ColoredVersion<'a> {
        version: &'a PackageVersion,
        theme: &'a Theme,
    }

    impl<'a> fmt::Display for ColoredVersion<'a> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self.version {
                PackageVersion::None => {
                    fmt::Display::fmt(&"<none>".color(self.theme.package_version_none), f)
                }
                PackageVersion::Commit { id: _, date } => {
                    let tmp = &date
                        .format(&format_description!("[year]-[month]-[day] [hour]:[minute]"))
                        .unwrap();
                    fmt::Display::fmt(&tmp.color(self.theme.package_version_date), f)
                }
            }
        }
    }

    ColoredVersion { version, theme }
}
