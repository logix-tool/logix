#[derive(clap::Parser)]
#[command(author, version, about, long_about)]
#[command(propagate_version = true)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(clap::Subcommand)]
enum Command {
    /// Deploy the current configuration
    Deploy {},
}

fn main() {
    let args = <Args as clap::Parser>::parse();
    match args.command {
        Command::Deploy {} => todo!("Deploy the current config"),
    }
}
