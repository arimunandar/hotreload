use crate::injector;

pub fn run(app_port: u16, app_host: &str) -> anyhow::Result<()> {
    println!("Pinging HotReloadServer at {}:{}...", app_host, app_port);

    match injector::ping(app_host, app_port) {
        Ok(response) => {
            println!("✅ Connected! Response: {}", response);
            Ok(())
        }
        Err(e) => {
            println!(
                "❌ Could not connect to {}:{} — is the app running?",
                app_host, app_port
            );
            println!("   Error: {}", e);
            // Return Ok since this is a diagnostic command
            Ok(())
        }
    }
}
