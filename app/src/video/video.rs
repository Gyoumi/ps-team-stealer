use tokio::process::Command;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::task::{self, JoinHandle};
use flume::{Sender, Receiver};
use core::str;
use std::error::Error;
use std::process::Stdio;
use std::sync::Arc;
use image::RgbImage;
use indicatif::{ProgressBar, ProgressStyle};
use std::env;

#[derive(Clone)]
struct Resolution {
    width: u32,
    height: u32,
    capacity: usize,
    filesize: Option<u64>, 
}

pub async fn start(url: &str) -> Result<(), Box<dyn std::error::Error>> {
    let resolution = get_resolution(url).await?;
    println!("resolution is {}x{}", resolution.width, resolution.height);
    println!("filesize is {}", resolution.filesize.unwrap());

    let video_stream = stream_youtube_video(url).await?;
    process_video(video_stream, &resolution).await?;
    Ok(())
}

async fn get_resolution(url: &str) -> Result<Resolution, std::io::Error> {
    let yt_dlp_process = Command::new("yt-dlp")
        .arg("--skip-download")
        .arg("--print").arg("%(width)sx%(height)s %(filesize_approx)s %(tbr)s %(duration)s")
        .arg(url)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;

    let output = yt_dlp_process.wait_with_output().await?;
    let stdout = String::from_utf8(output.stdout).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    println!("yt-dlp output: {}", stdout);

    let parts: Vec<&str> = stdout.trim().split_whitespace().collect();

    if parts.len() < 4 {
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "Unexpected yt-dlp output"));
    }

    let (width, height) = {
        let dims: Vec<&str> = parts[0].split('x').collect();

        if dims.len() != 2 {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "Invalid resolution format"));
        }
        (dims[0].parse::<u32>().unwrap(), dims[1].parse::<u32>().unwrap())
    };

    let filesize = parts.get(1).and_then(|s| s.parse::<u64>().ok());

    let filesize = match filesize {
        Some(fs) if fs > 0 => Some(fs),
        _ => {
            let tbr = parts.get(2).and_then(|s| s.parse::<f64>().ok());
            let duration = parts.get(3).and_then(|s| s.parse::<f64>().ok());
            match (tbr, duration) {
                (Some(tbr_kbps), Some(duration_secs)) => {
                    let bitrate_bps = tbr_kbps * 1000.0;
                    let estimated_size = (bitrate_bps * duration_secs) / 8.0; // bytes
                    Some(estimated_size.round() as u64)
                },
                _ => None,
            }
        }
    };

    Ok(Resolution {
        width,
        height,
        capacity: (width * height * 3) as usize,
        filesize,
    })
}

async fn stream_youtube_video(url: &str) -> Result<tokio::process::ChildStdout, std::io::Error> {
    let mut yt_dlp_process = Command::new("yt-dlp")
        .arg("-f").arg("bv")
        .arg("-o").arg("-")
        .arg("--quiet")
        .arg(url)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;

    let stdout = yt_dlp_process.stdout.take().expect("no yt-dlp stdout");

    tokio::spawn(async move {
        let status = yt_dlp_process.wait().await.expect("yt-dlp task error");
        println!("yt-dlp exit status: {}", status);
    });

    Ok(stdout)
}

async fn process_video(mut video_stream: tokio::process::ChildStdout, frame_size: &Resolution) -> Result<Vec<u8>, std::io::Error> {
    let mut ffmpeg_process = Command::new("ffmpeg")
        .arg("-loglevel").arg("error")
        .arg("-i").arg("pipe:0")
        .arg("-vf").arg("fps=18")
        .arg("-f").arg("image2pipe")
        .arg("-pix_fmt").arg("rgb24")
        .arg("-vcodec").arg("rawvideo")
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;

    let mut ffmpeg_stdin = ffmpeg_process.stdin.take().expect("no ffmpeg stdin");
    let mut ffmpeg_stdout = ffmpeg_process.stdout.take().expect("no ffmpeg stdout");

    let pb = frame_size.filesize.map(|size| {
        let pb = ProgressBar::new(size);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({percent}%)")
                .unwrap(),
        );
        pb
    });

    let pb_copy = pb.clone();
    let copy_task = tokio::spawn(async move {
        let mut buf = [0u8; 4096];
        loop {
            let n = video_stream.read(&mut buf).await?;
            if n == 0 { break; }
            ffmpeg_stdin.write_all(&buf[..n]).await?;
            if let Some(pb) = &pb_copy {
                pb.inc(n as u64);
            }
        }
        
        Ok::<(), std::io::Error>(())
    });

    let worker_count = match env::var("WORKER_COUNT") {
        Ok(val) => val.parse::<usize>().unwrap_or(1),
        Err(_) => 1,
    };
    let (tx, rx): (Sender<(Arc<Vec<u8>>, usize)>, Receiver<(Arc<Vec<u8>>, usize)>) = flume::unbounded(); 

    // spsc for each worker
    for id in 0..worker_count {
        let mut rx = rx.clone();        
        let frame_size = frame_size.clone();

        tokio::spawn(worker_task(id, rx, frame_size));
    }

    let mut frame_number = 0;

    loop {
        let mut frame_bytes = vec![0u8; frame_size.capacity];
        match ffmpeg_stdout.read_exact(&mut frame_bytes).await {
            Ok(_) => {
                let frame_data = Arc::new(frame_bytes);
                if tx.send_async((frame_data, frame_number)).await.is_err() {
                    eprintln!("All workers done. Exiting.");
                    break;
                }
                frame_number += 1;
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                println!("Finished reading frames.");
                break;
            }
            Err(e) => return Err(e),
        }
    }

    drop(tx);

    Ok(Vec::new())
}

async fn worker_task(
    id: usize,
    mut rx: Receiver<(Arc<Vec<u8>>, usize)>,
    frame_size: Resolution
) {
    while let Ok((frame_data, frame_number)) = rx.recv_async().await {
        if let Some(img) = RgbImage::from_raw(frame_size.width, frame_size.height, (*frame_data).clone()) {
            if let Err(e) = process_frames(img, frame_number as i32) {
                eprintln!("worker {id} frame {frame_number} failed: {e}");
            }
        }
    }
}


fn process_frames(frame: RgbImage, frame_number: i32) -> Result<(), Box<dyn Error>> {

    let frame_path = format!("./frames/frame_{}.png", frame_number); // Save the first frame
    frame.save(frame_path).expect("unable to save image");

    // read_frame(frame)?;

    // println!("Frame {}", frame_number);
    Ok(())
}
