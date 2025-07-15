mod video;
mod util;
use video::video::start;
use dotenv::dotenv;

// mod ocr;
// use ocr::read_frame;

mod image;
use image::image_processor;

mod text;
use text::text_processor;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
    dotenv().ok();
    let url = "https://www.youtube.com/watch?v=5xjQgr8xN9s";
    let image = ::image::open("./frame_50.png").unwrap().into_rgb8();
    image_processor::process_image(50, image).await;

    Ok(())

    // start(url).await?;
    // Ok(())

    // let img_file = "./src/frames/frame_50.bmp";
    // let img = image::open(img_file).map(|image| image.into_luma8())?;
    
    // process_image(&img)?;
    // Ok(())

    // let img = image::open(img_file).map(|image| image.into_luma8())?;
    // read_frame(img)
}