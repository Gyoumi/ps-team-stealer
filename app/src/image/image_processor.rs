use image::DynamicImage;
use super::image_segmenter;

pub async fn process_image(id: usize) {
    let segments = image_segmenter::segment_image(id).await;
    match segments {
        Ok(images) => for (i, img) in images.iter().enumerate() {
            let seg_path = format!("./segment/seg_{}.png", i); 
            img.save(seg_path).expect("unable to save image");
        },
        Err(e) => eprintln!("{}", e)
    }
}