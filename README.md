# fastdl ⚡

A fast, minimal HTTP downloader written in **Rust**, with support for **parallel chunked downloads** using HTTP `Range` requests.

This project started as a learning exercise and turned into a real, working download manager — focused on correctness, performance, and simplicity.

---

## ✨ Features

- 🚀 **Parallel downloads** using HTTP `Range` headers
- 📦 Automatic fallback to single-stream download when ranges aren’t supported
- 📊 Real-time progress bar with speed reporting
- 🧠 Smart use of `HEAD` requests to probe server capabilities
- 💾 Constant-memory streaming (no full-file buffering)
- 🛠 Simple, readable codebase (no unsafe code)

---

## 📦 How It Works (High Level)

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
