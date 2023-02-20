use std::fmt::{Display, Formatter};
use std::path::PathBuf;

use clap::Parser;

use crate::mig::spec;

#[derive(Debug, Parser)]
#[command(
    name = "edifact",
    about = "An EDIFACT tool for the edi@energy subset"
)]
pub struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Parser)]
enum Command {
    #[command(subcommand)]
    Mig(Mig),
}

#[derive(Debug, Parser)]
enum Mig {
    #[command(name = "parse", about = "Parse the message integration guide.")]
    Parse {
        #[arg(help = "A PDF file.")]
        file: PathBuf,
    },
}

#[derive(Debug)]
pub enum Error {
    NoPdf(),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "An error occurred")
    }
}

pub fn parse() -> Cli {
    Cli::parse()
}

pub fn run(cli: Cli) -> Result<(), Error> {
    match cli.command {
        Command::Mig(Mig::Parse { file }) => {
            println!("{:?}", spec::parse(file));
        }
    }
    Ok(())
}
