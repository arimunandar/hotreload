use std::process::Command;

pub fn run(stream: bool, last: &str, simulator: &str) -> anyhow::Result<()> {
    let mut args = vec![
        "simctl",
        "spawn",
        simulator,
        "log",
    ];

    if stream {
        args.push("stream");
    } else {
        args.push("show");
    }

    args.extend(&[
        "--predicate",
        "subsystem == \"com.hotreload\"",
        "--style",
        "compact",
    ]);

    if !stream {
        args.push("--last");
        args.push(last);
    }

    let status = Command::new("xcrun")
        .args(&args)
        .status()?;

    if !status.success() {
        anyhow::bail!(
            "xcrun simctl log {} exited with status: {}",
            if stream { "stream" } else { "show" },
            status
        );
    }

    Ok(())
}
