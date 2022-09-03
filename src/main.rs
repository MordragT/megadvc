use std::{env::current_dir, path::PathBuf};

use clap::{Parser, Subcommand};
/// megadvc program to push and pull data
/// from mega.nz in a similar fashion
/// like git works.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Initialises a data repository
    Init {
        /// The directory where to initialise the repository
        /// [default: current directory]
        #[clap(short, long)]
        dir: Option<PathBuf>,
        /// The directory where to initialise the repository in mega
        /// [default: current directory name]
        #[clap(short, long)]
        remote_dir: Option<PathBuf>,
    },
    /// Pushes all local changes to the remote folder
    Push,
    /// Pulls all remote changes into the local folder
    Pull,
    /// Adds files to be pushed
    Add {
        /// A list of files to be added
        #[clap(value_parser)]
        files: Vec<PathBuf>,
    },
    /// Removes files to be pushed
    Remove {
        /// A list of files to be removed
        #[clap(value_parser)]
        files: Vec<PathBuf>,
    },
    // Remote { # options in .megadvc.toml
    //     #[clap(subcommand)]
    //     command: Option<RemoteCommand>,
    // },
    /// Prints the current status of the repository
    Status,
    /// Login the user into mega
    Login {
        #[clap(value_parser)]
        username: String,
        #[clap(value_parser)]
        password: String,
    },
    /// Logout the user
    Logout,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let cli = Cli::parse();

    match cli.command {
        Command::Init { dir, remote_dir } => {
            if let Some(dir) = dir {
                megadvc::init(dir, remote_dir)?;
            } else {
                megadvc::init(current_dir()?, remote_dir)?;
            }

            Ok(())
        }
        Command::Add { files } => Ok(megadvc::add(files)?),
        Command::Remove { files } => Ok(megadvc::remove(files)?),
        Command::Status => Ok(megadvc::status()?),
        _ => todo!(),
    }
}
