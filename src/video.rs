use tokio::process::Command;
use tokio::io::AsyncReadExt;
use core::str;
use std::process::Stdio;
use image::ImageBuffer;

struct Resolution {
    width: u32,
    height: u32,
    capacity: usize
}

type FrameBuffer = ImageBuffer<image::Rgb<u8>, Vec<u8>>;

pub async fn start(url: &str) -> Result<(), Box<dyn std::error::Error>>{
    let resolution = get_resolution(url).await?;
    println!("resolution is {}x{}", resolution.width, resolution.height);

    let video_stream = stream_youtube_video(url)?;
    process_video(video_stream, resolution).await?;
    Ok(())
}

async fn get_resolution(url: &str) -> Result<Resolution, std::io::Error> {
    let yt_dlp_process = Command::new("yt-dlp")
        .arg("-f")
        .arg("bv")
        .arg("-o")
        .arg("-")
        .arg(url)
        .arg("--skip-download")
        .arg("--print")
        .arg("resolution")
        .stderr(Stdio::piped())
        .spawn()?;

    let output = yt_dlp_process.wait_with_output().await.expect("error yt-dlp ffmpeg task");
    let stdout = String::from_utf8(output.stderr).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?; 
    println!("exit status was: {}", stdout);
    let parts: Vec<&str> = stdout.trim().split('x').collect();

    let width: u32 = parts[0].parse().unwrap();
    let height: u32 = parts[1].parse().unwrap();

    let capacity: usize =  (width * height * 3).try_into().unwrap();
    
    Ok(Resolution {
        width, height, capacity
    })
}

fn stream_youtube_video(url: &str) -> Result<Stdio, std::io::Error> {
    let mut yt_dlp_process = Command::new("yt-dlp")
        .arg("-f")
        .arg("bv")
        .arg("-o")
        .arg("-")
        .arg(url)
        .stdout(Stdio::piped())
        .spawn()?;

    let stdout: Stdio = yt_dlp_process.stdout.take().unwrap().try_into().unwrap();

    tokio::spawn(async move {
        let status = yt_dlp_process.wait().await.expect("error yt-dlp ffmpeg task");
        println!("exit status was: {}", status);
    });


    Ok(stdout)
}

async fn process_video(video_stream: Stdio, frame_size: Resolution) -> Result<Vec<u8>, std::io::Error> {
    let mut ffmpeg_process = Command::new("ffmpeg")
    .arg("-i")
    .arg("-")
    .arg("-vf")
    .arg("fps=18")
    .arg("-f")
    .arg("image2pipe")
    .arg("-pix_fmt")
    .arg("rgb24")
    .arg("-vcodec")
    .arg("rawvideo")
    .arg("-")
    .stdin(video_stream)
    .stdout(Stdio::piped())
    .spawn()?;
    let mut ffmpeg_stdout = ffmpeg_process.stdout.take().unwrap(); 

    tokio::spawn(async move {
        let status = ffmpeg_process.wait().await.expect("error finishing ffmpeg task");
        println!("exit status was: {}", status);
    });

    let mut buffer = Vec::new();
    let mut buffer_idx = 0;
    let mut remainder = 0;

    let mut frame_number = 0;

    loop {
        let mut line = [0; 131972];
        let n = ffmpeg_stdout.read(&mut line).await?;

        if n == 0 {
            println!("end of file reached! buffer size is {}", buffer.len());
            break;
        }

        buffer.extend_from_slice(&line[..n]);      
        if remainder + n >= frame_size.capacity {
            let curr_bytes = buffer[buffer_idx..buffer_idx+frame_size.capacity].to_vec();
            
            tokio::spawn(async move {
                let img = FrameBuffer::from_raw(frame_size.width, frame_size.height, curr_bytes).unwrap();
                process_frames(img, frame_number).expect("image processing failed");
            });

            frame_number+=1;  

            buffer_idx += frame_size.capacity;
            remainder -= std::cmp::min(frame_size.capacity, remainder);
        } else {
            remainder += n;
        }        
    }

    Ok(Vec::new())
}

fn process_frames(frame: FrameBuffer, frame_number: i32) -> Result <(), std::io::Error> {

    let frame_path = format!("./frames/frame_{}.bmp", frame_number); // Save the first frame


    frame.save(frame_path).expect("unable to save image");

    println!("Frame {}", frame_number);
    Ok(())
}