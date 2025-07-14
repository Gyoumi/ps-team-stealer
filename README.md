# 🎞️ Rust Video Frame Processing & Segmentation Pipeline

An asynchronous, high-performance Rust application that streams and processes video frames directly from YouTube using `yt-dlp` and `ffmpeg`, and performs ONNX-based image segmentation on each frame.

## ✨ Features

- 🔁 **Streaming without file I/O**: Streams video as raw RGB frames via `yt-dlp` → `ffmpeg` → Rust using in-memory pipes.
- ⚙️ **Concurrent frame processing**: Uses an MPMC channel model to process frames with a pool of async worker tasks (configurable via `WORKER_COUNT` env variable).
- 🧠 **ONNX-based image segmentation**: Each frame is segmented using a custom-trained YOLOv8 neural network (exported to ONNX) with custom labeling, and bounding boxes are drawn on detected objects.
- 💾 **Segmented frame output**: Segmented frames are saved as PNGs in the `segment/` directory.
- 📊 **Real-time progress tracking**: Displays terminal progress bar based on estimated video size using `indicatif`.
- 🚧 **Planned: OCR support** (not yet implemented).

## 🧰 Tech Stack

- [Rust](https://www.rust-lang.org/)
- [Tokio](https://tokio.rs/) for async runtime
- [yt-dlp](https://github.com/yt-dlp/yt-dlp) for YouTube video streaming
- [ffmpeg](https://ffmpeg.org/) for video decoding
- [indicatif](https://docs.rs/indicatif/) for terminal progress bars
- [image](https://docs.rs/image/) crate for raw RGB image handling
- [onnxruntime (ort)](https://crates.io/crates/ort) for ONNX model inference
- [ndarray](https://crates.io/crates/ndarray) for tensor operations
- [flume](https://crates.io/crates/flume) for async channels
- [imageproc](https://crates.io/crates/imageproc) for drawing on images

## 📦 Requirements

- Rust 1.70+
- `yt-dlp` and `ffmpeg` installed and available in `$PATH`
- Custom-trained YOLOv8 ONNX model file (with your custom labeling) at `src/image/model/` (see code for expected filename)

## 🚀 Usage

1. Clone the repository and build the app:
   ```bash
   cargo build --release
   ```
2. Ensure you have the ONNX model in `src/image/model/` as required by the code.
3. Run the app (by default, it processes a hardcoded YouTube URL in `main.rs`):
   ```bash
   cargo run --release
   ```
   You can set the number of worker threads with the `WORKER_COUNT` environment variable:
   ```bash
   WORKER_COUNT=4 cargo run --release
   ```
4. Segmented frames will be saved as PNGs in the `segment/` directory (e.g., `segment/segmented_0.png`).

## 📝 Notes

- OCR functionality is planned but not yet implemented.
- You can modify the YouTube URL in `app/src/main.rs` to process a different video.
- The segmentation model and output format are customizable in the code.
