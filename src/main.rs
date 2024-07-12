#![deny(warnings, clippy::all)]

use std::fmt;

use logix::{based_path::BasedPath, config::Shell, env::Env, managed_file::FileStatus, Logix};
use owo_colors::{AnsiColors, DynColor, OwoColorize};

#[derive(clap::Parser)]
#[command(author, version, about, long_about)]
#[command(propagate_version = true)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(clap::Subcommand)]
enum Command {
    /// Get the status of your system
    Status {},
    /// Create a plan based on the current config
    Plan {},
    NewConfig {
        #[clap(short = 'u', long)]
        username: String,
        #[clap(short = 'n', long)]
        name: String,
        #[clap(short = 'e', long)]
        email: String,
        #[clap(long, default_value = "bash")]
        shell: String,
        #[clap(long, default_value = "hx")]
        editor: String,
    },
    Print {
        #[command(subcommand)]
        cmd: PrintCmd,
    },
}

#[derive(clap::Subcommand)]
enum PrintCmd {
    /// Load and then print the resolved config
    Config {},
}

#[derive(Clone, Copy)]
enum Color {
    Ansi(AnsiColors),
    Rgb(owo_colors::Rgb),
}

impl Color {
    const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self::Rgb(owo_colors::Rgb(r, g, b))
    }

    const fn ansi(color: AnsiColors) -> Self {
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

struct OptColor {
    is_some: Color,
    is_none: Color,
}

struct Theme {
    status_header: Color,
    status_up_to_date: Color,
    status_missing_from_both: Color,
    status_local_added: Color,
    status_logix_added: Color,
    status_modified: Color,
    status_error: Color,
    local_file: OptColor,
    logix_file: OptColor,
}

impl Theme {
    fn default_term() -> Self {
        let white = Color::ansi(AnsiColors::White);
        let red = Color::rgb(245, 66, 66);
        let yellow = Color::rgb(237, 193, 62);
        let blue = Color::rgb(100, 129, 245);
        let dimmed_blue = Color::rgb(109, 122, 179);
        let green = Color::rgb(37, 133, 11);
        let dimmed_green = Color::rgb(66, 97, 57);

        Self {
            status_header: Color::rgb(205, 239, 250),
            status_up_to_date: white,
            status_missing_from_both: red,
            status_local_added: yellow,
            status_logix_added: red,
            status_modified: yellow,
            status_error: red,
            local_file: OptColor {
                is_some: blue,
                is_none: dimmed_blue,
            },
            logix_file: OptColor {
                is_some: green,
                is_none: dimmed_green,
            },
        }
    }
}

fn colored_status(status: FileStatus, theme: &Theme) -> impl fmt::Display {
    // TODO: The state display names need a better naming convention
    match status {
        FileStatus::UpToDate => "Up to date".color(theme.status_up_to_date),
        FileStatus::MissingFromBoth => "Missing from both".color(theme.status_missing_from_both),
        FileStatus::LocalAdded => "Missing from logix".color(theme.status_local_added),
        FileStatus::LogixAdded => "Missing from local".color(theme.status_logix_added),
        FileStatus::Modified => "Has changes".color(theme.status_modified),
        FileStatus::ErrorReadingLocal(_) => "Failed to read local".color(theme.status_error),
        FileStatus::ErrorReadingLogix(_) => "Failed to read logix".color(theme.status_error),
    }
}

fn colored_path<'a>(path: Option<&'a BasedPath>, color: &'a OptColor) -> impl fmt::Display + 'a {
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

fn main() -> logix::error::Result<()> {
    let args = <Args as clap::Parser>::parse();
    let theme = Theme::default_term();

    match args.command {
        Command::Status {} => {
            println!(
                "{:<22}  {:<16}  {:<16}",
                "Status".color(theme.status_header),
                "Local file".color(theme.status_header),
                "Logix file".color(theme.status_header)
            );
            for (status, file) in Logix::load(Env::init()?)?.calculate_status()? {
                if let FileStatus::UpToDate = status {
                    continue;
                }

                let status = colored_status(status, &theme);
                let local = colored_path(file.local_path(), &theme.local_file);
                let logix = colored_path(file.logix_path(), &theme.logix_file);

                println!(" {status:<22}  {local:<16}  {logix:<16}");
            }
            println!()
        }
        Command::Plan {} => {
            let _logix = Logix::load(Env::init()?)?;
        }
        Command::NewConfig {
            ref username,
            ref name,
            ref email,
            ref shell,
            ref editor,
        } => {
            let config = logix::LogixConfigGenerator {
                username,
                name,
                email,
                shell: match shell.as_str() {
                    "bash" => Shell::Bash,
                    _ => {
                        eprintln!("Unknown shell {shell:?}");
                        std::process::exit(1);
                    }
                },
                editor,
            }
            .generate()?;
            println!("{config}");
        }
        Command::Print {
            cmd: PrintCmd::Config {},
        } => {
            let logix = Logix::load(Env::init()?)?;
            println!("{:#?}", logix.config());
        }
    }

    Ok(())
}
