#![deny(warnings, clippy::all)]

use logix::{config::Shell, env::Env, Logix};

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

fn main() -> logix::error::Result<()> {
    let args = <Args as clap::Parser>::parse();

    match args.command {
        Command::Status {} => {
            let logix = Logix::load(Env::init()?)?;
            let status = logix.calculate_status();
            println!("{status:#?}");
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
