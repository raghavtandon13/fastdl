use anyhow::Result;
use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use std::fs::File;
use std::io::Write;
use std::time::Duration;
use tokio;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long)]
    url: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    println!("Starting download: {}", args.url);

    let client = reqwest::Client::new();
    let resp = client.head(&args.url).send().await?.error_for_status()?;

    let total_size = resp
        .headers()
        .get(reqwest::header::CONTENT_LENGTH)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok())
        .expect("Server did not provide Content-Length");
    // also available as `content_length()`

    let accepts_ranges = resp
        .headers()
        .get(reqwest::header::ACCEPT_RANGES)
        .and_then(|v| v.to_str().ok())
        .map(|v| v == "bytes")
        .unwrap_or(false);

    println!("accepts_ranges: {}", accepts_ranges);

    let mut file = File::create("arch.iso")?;
    println!("Name {:?}", file);

    let bar = ProgressBar::new(total_size);
    bar.set_style(
        ProgressStyle::with_template(
            "[{elapsed_precise}] {bar:40.cyan/blue} {bytes}/{total_bytes} ({bytes_per_sec})",
        )
        .unwrap()
        .progress_chars("##-"),
    );
    bar.enable_steady_tick(Duration::from_millis(1000));

    let mut resp = client.get(&args.url).send().await?.error_for_status()?;
    while let Some(chunk) = resp.chunk().await? {
        file.write_all(&chunk)?;
        bar.inc(chunk.len() as u64);
    }

    bar.finish_with_message("Download complete!");
    Ok(())
}
