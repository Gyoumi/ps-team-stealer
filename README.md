# 🎞️ Rust Video Frame Processing Pipeline (WIP)

An asynchronous, high-performance Rust application that streams and processes video frames directly from YouTube using `yt-dlp` and `ffmpeg`.

## ✨ Features

- 🔁 **Streaming without file I/O**: Streams video as raw RGB frames via `yt-dlp` → `ffmpeg` → Rust using in-memory pipes.
- ⚙️ **Concurrent processing pipeline**: Uses an MPMC channel model to process frames with a pool of async worker tasks (WIP currently N SCPC).
- 📊 **Real-time progress tracking**: Displays terminal progress bar based on estimated video size using `indicatif`.

## 🧰 Tech Stack

- [Rust](https://www.rust-lang.org/)
- [Tokio](https://tokio.rs/)
- [yt-dlp](https://github.com/yt-dlp/yt-dlp)
- [ffmpeg](https://ffmpeg.org/)
- [indicatif](https://docs.rs/indicatif/) for terminal progress bars
- [image](https://docs.rs/image/) crate for raw RGB image handling

## 📦 Requirements

- Rust 1.70+
- `yt-dlp` and `ffmpeg` installed and available in `$PATH`

```bash
# On Ubuntu
sudo apt install ffmpeg
pip install -U yt-dlp
