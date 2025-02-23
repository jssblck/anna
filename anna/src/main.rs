use color_eyre::Result;
use winlock::Agent;

fn main() -> Result<()> {
    color_eyre::install()?;

    let _agent = Agent::new();

    Ok(())
}
