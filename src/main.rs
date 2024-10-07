#![deny(warnings, clippy::all)]

use std::path::PathBuf;

use logix::{
    config::Shell,
    error::Error,
    managed_file::{FileStatus, LocalFile, ManagedFile},
    managed_package::{ManagedPackage, PackageStatus, PackageVersion},
    system_state::SystemState,
};

mod main_utils;

use main_utils::{colored, context::Context, diff::diff_text_files, theme::Theme};
use owo_colors::OwoColorize;

#[derive(clap::Args)]
struct SharedArgs {
    #[clap(long, short = 'v', global(true))]
    verbose: bool,
}

#[derive(clap::Parser)]
#[command(author, version, about, long_about)]
#[command(propagate_version = true)]
struct Args {
    #[clap(flatten)]
    shared: SharedArgs,

    /// Specify the current log level
    #[clap(long, short = 'l', global(true), default_value = "warn")]
    log_level: log::LevelFilter,

    /// Log to the specified file, defaults to stderr
    #[clap(long, global(true))]
    log_file: Option<PathBuf>,

    #[command(subcommand)]
    command: Command,
}

#[derive(clap::Subcommand)]
enum Command {
    /// Get the status of your system
    Status {},
    /// Get the status of your config files
    ConfigStatus {},
    /// Get the status of your packages
    PackageStatus {
        /// Get more detailed status about the specified package
        #[clap(long, short = 'p')]
        package: Option<String>,
    },
    UpdateConfig {},
    InstallUpdates {},
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

impl Context {
    fn config_status(&self) -> Result<(), Error> {
        writeln!(
            self,
            "{:<15}  {:<10}  {}",
            "Status".color(self.theme.status_header),
            "Owner".color(self.theme.status_header),
            "Local file".color(self.theme.status_header),
        );
        for (status, file) in self.logix.calculate_config_status()? {
            if !self.args.verbose {
                if let FileStatus::UpToDate = status {
                    continue;
                }
            }

            let status = colored::status(status, &self.theme);
            let owner = colored::owner(file.owner(), &self.theme);
            let local = colored::path(file.local_path(), &self.theme.local_file);

            writeln!(self, " {status:<15}  {owner:<10}  {local}");
        }
        writeln!(self);
        Ok(())
    }

    fn print_packages_status<'a>(
        &'a self,
        it: impl Iterator<Item = ManagedPackage<'a>>,
    ) -> Result<(), Error> {
        let state = SystemState::init(self.logix.env())?;

        writeln!(
            self,
            "{:<20}  {:<16}  {:<16}  {:<16}",
            "Name".color(self.theme.status_header),
            "Installed".color(self.theme.status_header),
            "Downloaded".color(self.theme.status_header),
            "Remote".color(self.theme.status_header),
        );

        for package in it {
            let PackageStatus {
                installed_version,
                downloaded_version,
                latest_version,
            } = package.calculate_status(&state)?;
            writeln!(
                self,
                " {:<20}  {:<16}  {:<16}  {:<16}",
                package.name().color(self.theme.owner_package),
                colored::package_version(&installed_version, &self.theme),
                colored::package_version(&downloaded_version, &self.theme),
                colored::package_version(&latest_version, &self.theme),
            );
        }
        writeln!(self);
        Ok(())
    }

    fn packages_status(&self) -> Result<(), Error> {
        self.print_packages_status(self.logix.iter_packages())
    }

    fn package_status(&self, name: &str) -> Result<(), Error> {
        self.print_packages_status(self.logix.find_package(name).into_iter())
    }

    fn update_config(&self) -> Result<(), Error> {
        for (status, file) in self.logix.calculate_config_status()? {
            match status {
                FileStatus::UpToDate => {}
                FileStatus::MissingFromBoth => todo!(),
                FileStatus::LocalAdded => todo!(),
                FileStatus::LogixAdded => todo!(),
                FileStatus::Modified => match file {
                    ManagedFile::Local(_, LocalFile { local, logix }) => {
                        writeln!(self, "Config file has changes",);
                        writeln!(
                            self,
                            "Current config: {}",
                            colored::path(Some(&local), &self.theme.local_file)
                        );
                        writeln!(
                            self,
                            "Logix config:   {}/{}",
                            ".config/logix".color(self.theme.logix_root), // TODO: Need to be dynamic
                            colored::path(Some(&logix), &self.theme.logix_file)
                        );
                        diff_text_files(self, &local, &logix)?;
                    }
                    ManagedFile::Virtual(_, _) => todo!(),
                },
                FileStatus::ErrorReadingLocal(_) => todo!(),
                FileStatus::ErrorReadingLogix(_) => todo!(),
            }
        }
        Ok(())
    }

    pub fn install_updates(&self) -> Result<(), Error> {
        let mut state = SystemState::init(self.logix.env())?;
        for package in self.logix.iter_packages() {
            if package.is_custom() {
                // TODO: Add support for custom packages
                continue;
            }

            let status = package.calculate_status(&state)?;
            if status.need_update() {
                if matches!(status.installed_version, PackageVersion::None) {
                    writeln!(
                        self,
                        "Installing version {} of package {}",
                        colored::package_version(&status.latest_version, &self.theme),
                        package.name().color(self.theme.owner_package),
                    );
                } else {
                    writeln!(
                        self,
                        "Updating package {} from {} to {}",
                        package.name().color(self.theme.owner_package),
                        colored::package_version(&status.installed_version, &self.theme),
                        colored::package_version(&status.latest_version, &self.theme),
                    );
                }

                package.install_update(&mut state)?;
            }
        }
        Ok(())
    }
}

fn main() -> logix::error::Result<()> {
    let Args {
        shared,
        log_level,
        log_file,
        command,
    } = clap::Parser::parse();

    let mut logger = flexi_logger::Logger::with(log_level);

    logger = if let Some(path) = log_file {
        logger.log_to_file(flexi_logger::FileSpec::try_from(path).unwrap())
    } else {
        logger
            .log_to_stderr()
            .adaptive_format_for_stderr(flexi_logger::AdaptiveFormat::Default)
    };

    logger.start().unwrap();

    let theme = Theme::default_term();

    match command {
        Command::Status {} => {
            let ctx = Context::load(theme, shared)?;

            println!("Status of config files:");
            ctx.config_status()?;

            println!("Status of packages:");
            ctx.packages_status()?;
        }
        Command::ConfigStatus {} => {
            let ctx = Context::load(theme, shared)?;
            ctx.config_status()?;
        }
        Command::PackageStatus { package } => {
            let ctx = Context::load(theme, shared)?;
            if let Some(package) = package {
                ctx.package_status(&package)?;
            } else {
                ctx.packages_status()?;
            }
        }
        Command::UpdateConfig {} => {
            let ctx = Context::load(theme, shared)?;
            ctx.update_config()?;
        }
        Command::InstallUpdates {} => {
            let ctx = Context::load(theme, shared)?;
            ctx.install_updates()?;
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
            let ctx = Context::load(theme, shared)?;
            writeln!(ctx, "{:#?}", ctx.logix.config());
        }
    }

    Ok(())
}
