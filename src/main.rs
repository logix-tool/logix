use std::path::PathBuf;

use logix::{Error, Logix};
use logix_type::LogixLoader;
use logix_vfs::RelFs;

#[derive(clap::Parser)]
#[command(author, version, about, long_about)]
#[command(propagate_version = true)]
struct Args {
    #[command(subcommand)]
    command: Command,

    /// Directory where the config is located, use print-env to see the default
    #[clap(global = true, long)]
    config_dir: Option<PathBuf>,

    /// The root config file to load
    #[clap(global = true, long, default_value = "root.logix")]
    root_config: String,
}

#[derive(clap::Subcommand)]
enum Command {
    /// Create a plan based on the current config
    Plan {},
    Print {
        #[command(subcommand)]
        cmd: PrintCmd,
    },
}

#[derive(clap::Subcommand)]
enum PrintCmd {
    /// Calculate then print the environment that would used by other commands
    Env {},
    /// Load, then print the config
    Config {},
}

fn main() -> logix::Result<()> {
    let args = <Args as clap::Parser>::parse();

    let dirs =
        directories::ProjectDirs::from("com", "logix-tool", "logix").ok_or(Error::LocateHome)?;

    let env = logix::Env {
        config_root: args.config_dir.as_deref().unwrap_or(dirs.config_dir()),
    };

    let mut loader = LogixLoader::new(RelFs::new(env.config_root));

    match args.command {
        Command::Plan {} => {}
        Command::Print {
            cmd: PrintCmd::Env {},
        } => println!("{env:#?}"),
        Command::Print {
            cmd: PrintCmd::Config {},
        } => {
            let config = loader.load_file::<Logix>(&args.root_config)?;
            println!("{config:#?}");
        }
    }

    Ok(())
}
