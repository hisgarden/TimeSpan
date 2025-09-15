use clap::Parser;
use timespan::cli::{Cli, TimeSpanApp};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let app = TimeSpanApp::new(cli.database.clone())?;
    
    if let Err(_e) = app.run(cli).await {
        std::process::exit(1);
    }
    
    Ok(())
}
