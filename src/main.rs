mod video;

use actix_web::{post,  App, HttpResponse, HttpServer};
// use futures_core::stream::Stream;
// use futures_util::stream::StreamExt;
// use actix_multipart::Multipart;
// use std::error::Error;
//use futures_util::stream::StreamExt as _;
//use std::io::Cursor;
use std::process::{Command, Stdio};
use std::io::Write;
// use std::pin::Pin;
// use video_rs::decode::Decoder;
// use video_rs::Url;

use video::video;


// #[post("/upload")]
// async fn upload_video(mut payload: Multipart) -> Result<HttpResponse, actix_web::Error> {

//     let mut video_data: Vec<u8> = Vec::new();

//     while let Some(item) = payload.next().await {
//         let mut field = item?;

//         while let Some(chunk) = field.next().await {
//             video_data.extend_from_slice(&chunk?);
//         }
//     }

//     match analyze_video(&video_data).await {
//         Ok(_) => Ok(HttpResponse::Ok().body("Video analyzed successfully")),
//         Err(e) => Ok(HttpResponse::InternalServerError().body(format!("Failed to analyze video: {}", e)))
//     }
// }

#[post("/upload")]
async fn upload_video(youtube_url: String) -> Result<HttpResponse, actix_web::Error> {
    //let video_link = video(&youtube_url).await;

    let video_data = video(&youtube_url).await.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    //let video_buffer: &[u8] = &get_buffer(video_stream).await.map_err(actix_web::Error::from)?;


    match analyze_video(&video_data).await {
        Ok(_) => Ok(HttpResponse::Ok().body("Video analyzed successfully")),
        Err(e) => Ok(HttpResponse::InternalServerError().body(format!("Failed to analyze video: {}", e)))
    }
}

// async fn get_buffer<S>(mut stream: S) -> Result<Vec<u8>, std::io::Error> 
// where
//     S: Stream<Item = Result<Vec<u8>, io::Error>> + Unpin
//     {
//         let mut buffer = Vec::new();

//         while let Some(stream_chunk) = stream.next().await {
//             let chunk = stream_chunk?;
//             buffer.extend_from_slice(&chunk);
//         }
//     Ok(buffer)
// }

// async fn analyze_video(video_url: &str) -> Result<(), Box<dyn std::error::Error>> {
//     video_rs::init().unwrap();

//     let source = video_url.parse::<Url>().unwrap();
//     println!("{}", source);
//     let mut decoder = Decoder::new(source).expect("failed to create decoder");

//     decoder.decode_iter().take_while(Result::is_ok).map(Result::unwrap).for_each(|(_ts, frame)| {
//         let rgb = frame.slice(ndarray::s![0, 0, ..]).to_slice().unwrap();
//         println!("pixel at 0, 0: {}, {}, {}", rgb[0], rgb[1], rgb[2],);
//     });
    
//     Ok(())
// }

async fn analyze_video(video_data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    let mut ffmpeg_process = Command::new("ffmpeg")
        .arg("-i")      // Input from stdin
        .arg("pipe:0")  // Use pipe for input
        .arg("-vf")
        .arg("fps=1")   // Extract 1 frame per second (you can adjust this)
        .arg("-f")
        .arg("image2pipe")
        .arg("pipe:1")  // Output to stdout (image data)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    // Write video data to FFmpeg's stdin
    if let Some(ref mut stdin) = ffmpeg_process.stdin {
        stdin.write_all(video_data)?;
    }

    // Read frame data from FFmpeg's stdout
    let output = ffmpeg_process.wait_with_output()?;

    if output.status.success() {
        println!("Frames extracted successfully.");
        // You can process the frame data here (output.stdout contains the frame data)
    } else {
        println!("FFmpeg failed: {:?}", output.stderr);
        return Err("FFmpeg error".into());
    }
    Ok(())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let url = "https://www.youtube.com/watch?v=B93Zcv6cHts";
    let video_data = video(url).await;
    //let video_url = video("https://www.youtube.com/watch?v=MYfLLoX0AnU").await;
    match analyze_video(&video_data.expect("video not found")).await {
        Ok(_) => println!("closed successfully"),
        Err(err) => println!("error {}", err)
    }

    //video::download_video(url).await;

    HttpServer::new(|| {
        App::new()
            .service(upload_video)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}