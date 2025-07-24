# 🎞️ Pokémon Showdown Team Stealer

A fully Rust-based application that automatically extracts Pokémon teams from YouTube Showdown Live videos. It combines video streaming, image segmentation, OCR, and fuzzy data validation to reconstruct structured team data from gameplay footage.

## ✨ Features

- 🎥 **Direct YouTube streaming:** Streams video frames directly from YouTube using `yt-dlp` and `ffmpeg` using 
in-memory pipes.
- ⚡ **Concurrent async processing:** Utilizes a multi-producer, multi-consumer async pipeline for high throughput.
- 🧠 **ONNX-based image segmentation:** Uses a custom YOLOv8 ONNX model to detect and segment relevant UI elements in each frame.
- 🔤 **Multi-backend OCR:** Supports multiple OCR engines (RTen/ocrs, PaddleOCR, and LLM-based Ollama) for robust text extraction from segmented images.
- 🧩 **Fuzzy data validation:** Matches and corrects extracted names, moves, abilities, and items using fuzzy search and live Pokédex data.
- 🏆 **Team and battle reconstruction:** Automatically assembles structured Pokémon teams and battle state from noisy video data.
- 📊 **Real-time progress tracking:** Displays a terminal progress bar and logs detailed processing information.
- 🐳 **Docker Compose deployment:** Easily run the app and all dependencies (including Ollama for LLM OCR) with a single command.
- 🛠️ **Extensible and customizable:** Swap models, adjust pipeline steps, or add new OCR backends and validation logic as needed.

## 🧩 How It Works: Pipeline Overview

1. 🎥 **Video Streaming**
   - Streams YouTube videos directly using [`yt-dlp`](https://github.com/yt-dlp/yt-dlp) and [`ffmpeg`](https://ffmpeg.org/)—no intermediate files, all in-memory.

2. 🖼️ **Frame Extraction**
   - Decodes the video into raw RGB frames using the `image` crate and feeds them into an async processing pipeline.

3. 🧠 **Image Segmentation**
   - Each frame is segmented using a custom-trained YOLOv8 ONNX model (via [`onnxruntime`](https://crates.io/crates/ort`)).
   - Bounding boxes are drawn and relevant regions (e.g., Pokémon sprites, UI elements) are cropped for further analysis.

4. 🔤 **Optical Character Recognition (OCR)**
   - Segmented image regions are processed with multiple OCR backends:
     - 🟦 [`ocrs`](https://crates.io/crates/ocrs) (RTen models)
     - 🟩 [`paddle_ocr_rs`](https://crates.io/crates/paddle-ocr-rs) (PaddleOCR ONNX models)
     - 🤖 [`ollama_rs`](https://crates.io/crates/ollama-rs) (LLM-based, for structured JSON extraction)
   - Extracts Pokémon names, stats, moves, abilities, and more from the video UI.

5. 🧩 **Team Assembly & Fuzzy Validation**
   - Extracted data is validated and enriched using:
     - 📄 Local JSON datasets (Pokémon, moves, abilities, items, natures)
     - 🦊 [`rustemon`](https://crates.io/crates/rustemon) for live Pokédex data
     - 🔍 [`rust_fuzzy_search`](https://crates.io/crates/rust-fuzzy-search) for typo-tolerant matching
   - Assembles a structured representation of each team and battle, even in the presence of OCR errors or UI noise.

6. 💾 **Output**
   - Segmented frames and extracted team data are saved in the `segment/` directory.
   - Progress and results are logged to the terminal.

## 🛠️ Key Tools & Crates

- 🦀 **Rust async runtime:** [`tokio`](https://tokio.rs/)
- 🎥 **Video streaming:** `yt-dlp`, `ffmpeg`
- 🖼️ **Image processing:** [`image`](https://docs.rs/image/), [`imageproc`](https://crates.io/crates/imageproc`)
- 🧠 **ONNX inference:** [`onnxruntime (ort)`](https://crates.io/crates/ort)
- 🔤 **OCR:** [`ocrs`](https://crates.io/crates/ocrs), [`paddle_ocr_rs`](https://crates.io/crates/paddle-ocr-rs), [`ollama_rs`](https://crates.io/crates/ollama-rs)
- 🦊 **Pokémon data:** [`rustemon`](https://crates.io/crates/rustemon), local JSON
- 🔍 **Fuzzy search:** [`rust_fuzzy_search`](https://crates.io/crates/rust-fuzzy-search)
- 🔗 **Async channels:** [`flume`](https://crates.io/crates/flume)
- 📊 **Progress bars:** [`indicatif`](https://docs.rs/indicatif/)
- 🧩 **State management:** [`once_cell`](https://crates.io/crates/once_cell), [`serde`](https://crates.io/crates/serde)

## 🐳 Running with Docker Compose (Recommended)

The easiest way to run the app is with Docker Compose, which sets up both the main application and an Ollama service for LLM-based OCR.

1. 🏗️ From the `app/` directory, build and start all services:
   ```bash
   docker-compose up --build
   ```
   This will build and start two services:
   - 🤖 `ollama`: Runs the Ollama LLM server (using `Dockerfile.ollama` and `start-ollama.sh`)
   - 🦀 `app`: Runs the main Rust application, configured to use the Ollama service for OCR

2. 📂 The `docker-compose.yml` mounts the necessary model and data directories. You can adjust volumes and environment variables as needed in the compose file.

3. 🛑 To stop the services:
   ```bash
   docker-compose down
   ```

> **Tip:** You can set environment variables (e.g., `WORKER_COUNT`, `YOLO_MODEL`, `LABELS`, `OLLAMA_HOST`, `OLLAMA_PORT`) in the `docker-compose.yml` or via an `.env` file.

## 📝 Notes

- 📝 You can modify the YouTube URL in `app/src/main.rs` to process a different video.
- 🧠 The segmentation model, OCR backend, and output format are customizable in the code and via environment variables.
- 🦊 The application validates and enriches extracted data using local JSON datasets and the Rustemon API.
- 📂 For best results, ensure your models are compatible and placed in the correct directory.
