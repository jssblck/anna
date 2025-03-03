use std::path::PathBuf;

use clap::Parser;
use color_eyre::{eyre::Context, Result};
use winlock::Agent;

#[derive(Debug, Parser)]
pub struct AgentArgs {
    /// Branch name for the agent to work on
    #[arg(value_name = "BRANCH")]
    branch: String,
}

pub fn main(project: PathBuf, AgentArgs { branch }: AgentArgs) -> Result<()> {
    eprintln!("Creating session for branch {branch:?}...");

    let agent = Agent::builder()
        .project(&project)
        .branch(&branch)
        .build()
        .context("create agent session")?;
    eprintln!(
        "Session {} at {} for branch {:?}",
        agent.status,
        agent.workspace.display(),
        branch,
    );

    agent.run().context("run agent")?;

    eprintln!();
    eprintln!("Note: If you're done, you can use `anna session ...` commands to clean up.");
    eprintln!("      Otherwise, you can always execute this same command to re-enter the current session.");
    Ok(())
}
