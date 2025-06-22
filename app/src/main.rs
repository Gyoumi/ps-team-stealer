mod video;
mod util;
use video::video::start;
use dotenv::dotenv;
// mod ocr;
// use ocr::read_frame;

mod image;
use image::image_processor;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
    dotenv().ok();
    let url = "https://www.youtube.com/watch?v=5xjQgr8xN9s";

    image_processor::process_image(0).await;

    Ok(())

    // start(url).await

    // let img_file = "./src/frames/frame_50.bmp";
    // let img = image::open(img_file).map(|image| image.into_luma8())?;
    
    // process_image(&img)?;
    // Ok(())

    // let img = image::open(img_file).map(|image| image.into_luma8())?;
    // read_frame(img)
}