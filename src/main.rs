#![deny(warnings, clippy::all)]

use logix::{env::Env, Logix};

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

fn main() -> logix::error::Result<()> {
    let args = <Args as clap::Parser>::parse();

    let logix = Logix::load(Env::init()?)?;

    match args.command {
        Command::Status {} => {
            let status = logix.calculate_status();
            println!("{status:#?}");
        }
        Command::Plan {} => {}
        Command::Print {
            cmd: PrintCmd::Config {},
        } => {
            println!("{:#?}", logix.config());
        }
    }

    Ok(())
}
