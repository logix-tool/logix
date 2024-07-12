#![deny(warnings, clippy::all)]

use logix::{
    config::Shell, env::Env, error::Error, managed_file::FileStatus,
    managed_package::PackageStatus, Logix,
};

mod main_utils;

use main_utils::{colored, theme::Theme};
use owo_colors::OwoColorize;

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
    Status {
        #[clap(long, short = 'v')]
        verbose: bool,
    },
    /// Get the status of your config files
    ConfigStatus {
        #[clap(long, short = 'v')]
        verbose: bool,
    },
    /// Get the status of your packages
    PackageStatus {
        /// Get more detailed status about the specified package
        #[clap(long, short = 'p')]
        package: Option<String>,
        #[clap(long, short = 'v')]
        verbose: bool,
    },
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

fn config_status(logix: &Logix, theme: &Theme, verbose: bool) -> Result<(), Error> {
    println!(
        "{:<15}  {:<10}  {}",
        "Status".color(theme.status_header),
        "Owner".color(theme.status_header),
        "Local file".color(theme.status_header),
    );
    for (status, file) in logix.calculate_config_status()? {
        if !verbose {
            if let FileStatus::UpToDate = status {
                continue;
            }
        }

        let status = colored::status(status, theme);
        let owner = colored::owner(file.owner(), theme);
        let local = colored::path(file.local_path(), &theme.local_file);

        println!(" {status:<15}  {owner:<10}  {local}");
    }
    println!();
    Ok(())
}

fn packages_status(logix: &Logix, theme: &Theme, _verbose: bool) -> Result<(), Error> {
    println!(
        "{:<10}  {:<16}  {:<16}  {:<16}",
        "Name".color(theme.status_header),
        "Installed".color(theme.status_header),
        "Downloaded".color(theme.status_header),
        "Remote".color(theme.status_header),
    );

    for package in logix.iter_packages() {
        let PackageStatus {
            installed_version,
            downloaded_version,
            latest_version,
        } = package.calculate_status()?;
        println!(
            " {:<10}  {:<16}  {:<16}  {:<16}",
            package.name().color(theme.owner_package),
            colored::package_version(&installed_version, theme),
            colored::package_version(&downloaded_version, theme),
            colored::package_version(&latest_version, theme),
        );
    }
    println!();
    Ok(())
}

fn package_status(
    _logix: &Logix,
    _theme: &Theme,
    _verbose: bool,
    _name: &str,
) -> Result<(), Error> {
    todo!()
}

fn main() -> logix::error::Result<()> {
    let args = <Args as clap::Parser>::parse();
    let theme = Theme::default_term();

    match args.command {
        Command::Status { verbose } => {
            let logix = Logix::load(Env::init()?)?;

            println!("Status of config files:");
            config_status(&logix, &theme, verbose)?;

            println!("Status of packages:");
            packages_status(&logix, &theme, verbose)?;
        }
        Command::ConfigStatus { verbose } => {
            let logix = Logix::load(Env::init()?)?;
            config_status(&logix, &theme, verbose)?;
        }
        Command::PackageStatus { package, verbose } => {
            let logix = Logix::load(Env::init()?)?;
            if let Some(package) = package {
                package_status(&logix, &theme, verbose, &package)?;
            } else {
                packages_status(&logix, &theme, verbose)?;
            }
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
