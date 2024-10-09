mod video;
use video::start;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
    let url = "https://www.youtube.com/watch?v=5xjQgr8xN9s";

    start(url).await
}