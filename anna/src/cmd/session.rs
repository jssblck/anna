use std::path::PathBuf;

use clap::{Parser, Subcommand};
use color_eyre::Result;
use winlock::Sessions;

#[derive(Debug, Subcommand)]
pub enum SessionCommands {
    /// List all available sessions
    ListAll,

    /// List sessions for for the current project
    List,

    /// Remove a session for the current project
    Remove(SessionRemoveArgs),
}

#[derive(Debug, Parser)]
pub struct SessionRemoveArgs {
    /// Branch name
    #[arg(value_name = "BRANCH")]
    branch: String,
}

pub fn main(project: PathBuf, command: SessionCommands) -> Result<()> {
    match command {
        SessionCommands::ListAll => main_list_all(),
        SessionCommands::List => main_list(project),
        SessionCommands::Remove(args) => main_remove(project, args),
    }
}

fn main_list_all() -> Result<()> {
    let sessions = Sessions::list_all()?;
    if sessions.is_empty() {
        eprintln!("No sessions found");
        return Ok(());
    }

    for session in sessions {
        eprintln!(
            "Project: {}\n\tBranch: {}\n\tWorkspace: {}",
            session.project.display(),
            session.branch,
            session.workspace.display()
        );
    }

    Ok(())
}

fn main_list(project: PathBuf) -> Result<()> {
    let sessions = Sessions::list(&project)?;
    if sessions.is_empty() {
        eprintln!("No sessions found for project: {}", project.display());
        return Ok(());
    }

    for session in sessions {
        eprintln!(
            "Branch: {}\n\tWorkspace: {}",
            session.branch,
            session.workspace.display()
        );
    }

    Ok(())
}

fn main_remove(project: PathBuf, args: SessionRemoveArgs) -> Result<()> {
    Sessions::remove(&project, &args.branch)?;
    eprintln!("Session removed successfully");
    Ok(())
}
