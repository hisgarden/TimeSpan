use clap::Parser;
use toggl::cli::{Cli, TogglApp};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let app = TogglApp::new(cli.database.clone())?;
    
    if let Err(e) = app.run(cli).await {
        std::process::exit(1);
    }
    
    Ok(())
}
