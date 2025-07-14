use image::{RgbImage};
use super::image_segmenter;

pub fn process_image(id: usize, image: RgbImage) {
    println!("Processing image {}", id);
    let _segments = image_segmenter::segment_image(id, image);
    // match segments {
    //     Ok(images) => for (label, img) in &images {
    //         let seg_path = format!("./segment/seg_{}.png", label); 
    //         img.save(seg_path).expect("unable to save image");
    //     },
    //     Err(e) => eprintln!("{}", e)
    // }
}