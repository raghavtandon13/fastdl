# fastdl ⚡

A fast, minimal HTTP downloader written in **Rust**, supporting **parallel chunked downloads** via HTTP `Range` requests.

This project began as a learning exercise and evolved into a real, working download manager — focused on correctness, performance, and simplicity.

---

## ✨ Features

- 🚀 **Parallel downloads** using HTTP `Range` headers
- 📦 Automatic fallback to single-stream download when ranges aren’t supported
- 📊 Real-time progress bar with speed reporting
- 🧠 Smart use of `HEAD` requests to probe server capabilities
- 💾 Constant-memory streaming (no full-file buffering)
- 🛠 Simple, readable codebase (no unsafe code)

---

## 📦 How It Works

1. Sends an HTTP `HEAD` request to determine:
   - File size (`Content-Length`)
   - Whether the server supports byte ranges (`Accept-Ranges`)
2. Pre-allocates the output file to the full size
3. Chooses download strategy:
   - **Single stream** for small files or servers without range support
   - **Parallel chunked downloads** for large files with range support
4. Streams data directly to disk while updating a shared progress bar

Each chunk is downloaded independently and written to a fixed byte range in the output file — no locks, no overlaps.

---

## 🖥 Usage

### Build

```bash
cargo build --release
```

### Download a file (single stream)

```bash
fastdl -u https://example.com/file.iso
```

### Download with parallel chunks

```bash
fastdl -u https://example.com/file.iso -j 4
```

### Specify output file

```bash
fastdl -u https://example.com/file.iso -o myfile.iso
```

---

## 🧪 Recommended Test Files

For reliable testing of parallel downloads:

- http://ipv4.download.thinkbroadband.com/1GB.zip

(Some CDNs throttle per-IP bandwidth, so not all servers show speedups with parallelism.)

---

## ⚠️ Notes & Limitations

- HTTPS testing depends on system clock and certificate validity
- Parallel downloads may not improve speed on all servers
- Currently blocks on file I/O (uses `std::fs::File`)
- No resume support yet (see roadmap)

---

## 🛣 Roadmap / Future Features

Planned improvements:

- ⏯ Resume support (continue partially downloaded files)
- 🔁 Per-chunk retry with exponential backoff
- 📏 Adaptive chunk sizing based on file size
- 🧾 Filename detection via `Content-Disposition`
- 🧵 Async file I/O using `tokio::fs`
- 🔐 Optional checksum verification (SHA256)
- 🧪 Benchmark mode for testing throughput
- 🌐 (Long-term) BitTorrent support

---

## 🧠 Why This Project Exists

This project was built to understand how real download managers work under the hood:

- HTTP semantics
- Streaming I/O
- Concurrency with correctness
- Real-world CDN behavior

It is intentionally minimal, readable, and dependency-light.

---

## 🦀 Built With

- Rust
- tokio
- reqwest
- clap
- indicatif
- anyhow

---

## 📜 License

MIT (or pick your preferred license).

---

## 🙌 Acknowledgements

Inspired by tools like wget, aria2, and curl, but built from scratch to learn how they actually work.

