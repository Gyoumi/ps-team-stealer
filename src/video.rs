use rustube::{Error, Id, Stream, Video, VideoDescrambler, VideoFetcher};
use rustube::fetcher::{recommended_cookies, recommended_headers};
// use reqwest::Client;
// use tokio_stream::StreamExt;


pub type Result<T, E = std::io::Error> = core::result::Result<T, E>;

// pub async fn download_video(video_url: &str) {
//     println!("starting download for {video_url}");
//     println!("downloaded video to {:?}", rustube::download_best_quality(&video_url).await.unwrap());
// }

pub async fn video(video_url: &str) -> Result<Vec<u8>> {
    let id = Id::from_raw(video_url).unwrap();

    // let cookie_jar: reqwest::cookie::Jar = recommended_cookies();
    // let headers: reqwest::header::HeaderMap = recommended_headers();

    // let client = Client::builder()
    //         .default_headers(headers)
    //         .cookie_provider(std::sync::Arc::new(cookie_jar))
    //         .build().map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("error building client: {e}")))?;

    let fetcher: VideoFetcher = VideoFetcher::from_id(id.into_owned()).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("error descrambling video: {e}")))?;
    let descrambler: VideoDescrambler = fetcher.fetch().await.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("error fetching descrambler: {e}")))?;
    let video: Video = descrambler.descramble().map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("error descrambling video: {e}")))?;


    let video_path = video
        .streams()
        .iter()
        .filter(|stream| stream.includes_video_track && stream.quality == rustube::video_info::player_response::streaming_data::Quality::Hd720)
        .max_by_key(|stream| stream.quality_label)
        .unwrap()
        .download()
        .await.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("error downloading video: {e}")))?;
    //let stream: &Stream = video.best_video().unwrap();
    // let video_info = descrambler.video_info();
    // println!("title of video: {:?}", video_info);
    // let path = stream.download().await.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("error downloading video: {e}")))?;
    // println!("downloaded to {:?}", path);

    //Ok(String::from(stream.signature_cipher.url.as_str()))

    // let res = get(client, &stream.signature_cipher.url).await.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("error fetching video url: {e}")))?;
    // let bytes = get_stream_bytes(res.bytes_stream()).await?;
    // Ok(bytes)

    Ok(Vec::new())
}


// // copied from rustube repo
// async fn get(client: Client, url: &url::Url) -> Result<reqwest::Response, Error> {
//     Ok(
//         client
//             .get(url.as_str())
//             .send()
//             .await?
//             .error_for_status()?
//     )
// }

// async fn get_stream_bytes<S>(mut stream: S) -> Result<Vec<u8>>
// where
//     S: tokio_stream::Stream<Item=reqwest::Result<bytes::Bytes>> + Unpin
//     {
//         let mut buffer = Vec::new();

//         while let Some(chunk) = stream.next().await {
//             let chunk = chunk.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("error retrieving chunk: {e}")))?;
//             buffer.extend_from_slice(&chunk);
//         }
//         Ok(buffer)
// }