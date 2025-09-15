use clap::Parser;
use timespan::cli::{Cli, TimeSpanApp};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use timespan::cli::sanitize_error_message;
    
    let cli = Cli::parse();
    
    let app = match TimeSpanApp::new(cli.database.clone()) {
        Ok(app) => app,
        Err(e) => {
            eprintln!("Error: {}", sanitize_error_message(&e));
            std::process::exit(1);
        }
    };
    
    if let Err(e) = app.run(cli).await {
        // Error messages are already sanitized in the CLI handlers
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
    
    Ok(())
}
