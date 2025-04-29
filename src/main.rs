mod video;
mod error;
use video::start;

// mod ocr;
// use ocr::read_frame;

mod image_processor;
use image_processor::process_image;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
    let url = "https://www.youtube.com/watch?v=5xjQgr8xN9s";

    start(url).await

    // let img_file = "./src/frames/frame_50.bmp";
    // let img = image::open(img_file).map(|image| image.into_luma8())?;
    
    // process_image(&img)?;
    // Ok(())

    // let img = image::open(img_file).map(|image| image.into_luma8())?;
    // read_frame(img)
}