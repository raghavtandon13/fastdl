use anyhow::{Result, anyhow};
use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::header::{ACCEPT_RANGES, CONTENT_LENGTH, RANGE};
use std::fs::OpenOptions;
use std::io::{Seek, SeekFrom, Write};
use std::path::Path;
use std::time::Duration;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// URL to download
    #[arg(short, long)]
    url: String,

    /// Number of parallel chunks
    #[arg(short = 'j', long, default_value_t = 4)]
    jobs: usize,

    /// Output file
    #[arg(short, long)]
    output: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let output = args
        .output
        .clone()
        .unwrap_or_else(|| filename_from_url(&args.url));

    println!("Downloading → {}", output);

    let client = reqwest::Client::new();

    // ---------- HEAD ----------
    let head = client.head(&args.url).send().await?.error_for_status()?;

    let total_size = head
        .headers()
        .get(CONTENT_LENGTH)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok())
        .ok_or_else(|| anyhow!("Server did not provide Content-Length"))?;

    let accepts_ranges = head
        .headers()
        .get(ACCEPT_RANGES)
        .and_then(|v| v.to_str().ok())
        .map(|v| v == "bytes")
        .unwrap_or(false);

    // ---------- FILE ----------
    let file = OpenOptions::new().create(true).write(true).open(&output)?;

    file.set_len(total_size)?;

    let bar = ProgressBar::new(total_size);
    bar.set_style(
        ProgressStyle::with_template(
            "[{elapsed_precise}] {bar:40.cyan/blue} {bytes}/{total_bytes} ({bytes_per_sec})",
        )
        .unwrap(),
    );
    bar.enable_steady_tick(Duration::from_millis(500));

    // ---------- DOWNLOAD ----------
    if accepts_ranges && args.jobs > 1 {
        parallel_download(
            client,
            &args.url,
            &output,
            total_size,
            args.jobs,
            bar.clone(),
        )
        .await?;
    } else {
        single_download(client, &args.url, &output, bar.clone()).await?;
    }

    bar.finish_with_message("Download complete");
    Ok(())
}

// SINGLE STREAM
async fn single_download(
    client: reqwest::Client,
    url: &str,
    output: &str,
    bar: ProgressBar,
) -> Result<()> {
    let mut resp = client.get(url).send().await?.error_for_status()?;
    let mut file = OpenOptions::new().write(true).open(output)?;

    while let Some(chunk) = resp.chunk().await? {
        file.write_all(&chunk)?;
        bar.inc(chunk.len() as u64);
    }

    Ok(())
}

// PARALLEL DOWNLOAD
async fn parallel_download(
    client: reqwest::Client,
    url: &str,
    output: &str,
    total_size: u64,
    jobs: usize,
    bar: ProgressBar,
) -> Result<()> {
    let chunk_size = total_size / jobs as u64;
    let mut handles = Vec::new();

    for i in 0..jobs {
        let start = i as u64 * chunk_size;
        let end = if i == jobs - 1 {
            total_size - 1
        } else {
            start + chunk_size - 1
        };

        let client = client.clone();
        let url = url.to_string();
        let output = output.to_string();
        let bar = bar.clone();

        let handle =
            tokio::spawn(async move { download_chunk(client, url, output, start, end, bar).await });

        handles.push(handle);
    }

    for h in handles {
        h.await??;
    }

    Ok(())
}

async fn download_chunk(
    client: reqwest::Client,
    url: String,
    output: String,
    start: u64,
    end: u64,
    bar: ProgressBar,
) -> Result<()> {
    eprintln!("chunk {}–{} started", start, end);
    let mut resp = client
        .get(&url)
        .header(RANGE, format!("bytes={}-{}", start, end))
        .send()
        .await?
        .error_for_status()?;

    let mut file = OpenOptions::new().write(true).open(output)?;
    file.seek(SeekFrom::Start(start))?;

    let mut offset = start;

    while let Some(chunk) = resp.chunk().await? {
        file.seek(SeekFrom::Start(offset))?;
        file.write_all(&chunk)?;

        offset += chunk.len() as u64;
        bar.inc(chunk.len() as u64);
    }

    Ok(())
}

// UTILS
fn filename_from_url(url: &str) -> String {
    Path::new(url)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("download.bin")
        .to_string()
}
