// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use clap::{Parser, Subcommand};
use cmd::{agent::AgentArgs, session::SessionCommands};
use color_eyre::{eyre::Context, Result};

mod cmd;

#[derive(Parser)]
#[command(version, about)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create an agent session and run the agent.
    Agent(AgentArgs),

    /// Manage agent sessions
    Session {
        #[command(subcommand)]
        command: SessionCommands,
    },
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let cli = Args::parse();
    let project = std::env::current_dir().context("get working directory")?;

    match cli.command {
        Commands::Agent(args) => cmd::agent::main(project, args),
        Commands::Session { command } => cmd::session::main(project, command),
    }
}
