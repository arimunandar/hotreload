use std::path::Path;
use std::time::Duration;

use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::sync::mpsc;

/// Start a file watcher that streams file change events
pub fn start_watcher(
    watch_paths: &[String],
    debounce_ms: u64,
    event_tx: mpsc::Sender<Vec<String>>,
) -> anyhow::Result<RecommendedWatcher> {
    let (notify_tx, mut notify_rx) = mpsc::channel::<Event>(256);

    let mut watcher: RecommendedWatcher =
        notify::RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                if let Ok(event) = res {
                    let _ = notify_tx.blocking_send(event);
                }
            },
            Config::default().with_poll_interval(Duration::from_millis(debounce_ms)),
        )?;

    // Register all watch paths
    for path_str in watch_paths {
        let path = Path::new(path_str);
        if path.exists() {
            watcher.watch(path, RecursiveMode::Recursive)?;
            tracing::info!("Watching: {}", path.display());
        } else {
            tracing::warn!("Watch path does not exist: {}", path.display());
        }
    }

    // Debounce and forward events
    let event_tx_clone = event_tx.clone();
    tokio::spawn(async move {
        let debounce = Duration::from_millis(debounce_ms);
        let mut pending: Vec<String> = Vec::new();
        let mut last_event = tokio::time::Instant::now();

        loop {
            let timeout = tokio::time::sleep(debounce);
            tokio::pin!(timeout);

            tokio::select! {
                Some(event) = notify_rx.recv() => {
                    let paths = extract_changed_files(&event);
                    for p in paths {
                        if !pending.contains(&p) {
                            pending.push(p);
                        }
                    }
                    last_event = tokio::time::Instant::now();
                }
                _ = &mut timeout => {
                    if !pending.is_empty() && last_event.elapsed() >= debounce {
                        tracing::debug!("Debounced {} file changes", pending.len());
                        let _ = event_tx_clone.send(pending.clone()).await;
                        pending.clear();
                    }
                }
            }

            // If the channel is closed, stop
            if event_tx_clone.is_closed() {
                break;
            }
        }
    });

    Ok(watcher)
}

/// Extract changed file paths from a notify event
fn extract_changed_files(event: &Event) -> Vec<String> {
    let mut files = Vec::new();

    match &event.kind {
        EventKind::Modify(_) | EventKind::Create(_) => {
            for path in &event.paths {
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                // Only watch .swift files
                if ext == "swift" {
                    if let Some(s) = path.to_str() {
                        files.push(s.to_string());
                    }
                }
            }
        }
        _ => {}
    }

    files
}
