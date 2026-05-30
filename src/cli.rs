use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "hotreload", about = "Hot reload for iOS apps on the simulator")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Path to the project root (defaults to current directory)
    #[arg(short, long, global = true)]
    pub path: Option<String>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Detect Xcode project, validate build settings, create .hotreload/config.toml
    Init {
        /// Force re-initialization even if config already exists
        #[arg(long)]
        force: bool,
    },
    /// Watch Swift files, recompile -> serve dylib -> inject into simulator
    Watch {
        /// Port for the dylib HTTP server (default: 9876)
        #[arg(long, default_value = "9876")]
        http_port: u16,

        /// TCP port on the iOS app to send injection commands (default: 8899)
        #[arg(long, default_value = "8899")]
        app_port: u16,

        /// Simulator host IP (default: 127.0.0.1)
        #[arg(long, default_value = "127.0.0.1")]
        app_host: String,
    },
    /// One-shot compile + inject a specific Swift file
    Inject {
        /// Swift file to compile and inject
        file: String,

        /// Port for the dylib HTTP server (default: 9876)
        #[arg(long, default_value = "9876")]
        http_port: u16,

        /// TCP port on the iOS app to send injection commands (default: 8899)
        #[arg(long, default_value = "8899")]
        app_port: u16,

        /// Simulator host IP (default: 127.0.0.1)
        #[arg(long, default_value = "127.0.0.1")]
        app_host: String,
    },
    /// Show or stream simulator logs for com.hotreload
    Log {
        /// Stream logs continuously instead of showing recent entries
        #[arg(long)]
        stream: bool,

        /// How far back to show logs, e.g. "5m", "1h" (only for show mode)
        #[arg(long, default_value = "2m")]
        last: String,

        /// Simulator device ID or "booted"
        #[arg(long, default_value = "booted")]
        simulator: String,
    },
    /// Ping the app's HotReloadServer to check connection
    Status {
        /// TCP port on the iOS app to ping (default: 8899)
        #[arg(long, default_value = "8899")]
        app_port: u16,

        /// Simulator host IP (default: 127.0.0.1)
        #[arg(long, default_value = "127.0.0.1")]
        app_host: String,
    },
}

impl Cli {
    pub fn exec(&self) -> anyhow::Result<()> {
        let project_root = self
            .path
            .clone()
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| std::env::current_dir().unwrap());

        match &self.command {
            Commands::Init { force } => crate::commands::init::run(&project_root, *force),
            Commands::Watch {
                http_port,
                app_port,
                app_host,
            } => crate::commands::watch::run(&project_root, *http_port, *app_port, app_host),
            Commands::Inject {
                file,
                http_port,
                app_port,
                app_host,
            } => crate::commands::inject::run(&project_root, file, *http_port, *app_port, app_host),
            Commands::Log {
                stream,
                last,
                simulator,
            } => crate::commands::log::run(*stream, last, simulator),
            Commands::Status { app_port, app_host } => {
                crate::commands::status::run(*app_port, app_host)
            }
        }
    }
}
