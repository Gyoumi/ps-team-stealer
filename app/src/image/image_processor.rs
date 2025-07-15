use image::{RgbImage};
use super::image_segmenter;
use super::ocr;

pub async fn process_image(id: usize, image: RgbImage) {
    let segments = image_segmenter::segment_image(id, image);
    
    match segments {
        Ok(images) => {
            if let Some((label, img)) = images.iter().find(|(key,_)| key.ends_with("pokemon_hover")) { // only care about player's mons for now
                let result = ocr::ocr_segment(img.get(0).unwrap()).await;
                match result {
                    Ok(text) => println!("OCR result for {}: {}", label, text),
                    Err(e) => eprintln!("Error OCRing image: {}", e)
                }
                return;
            }
            // for (label, img) in &images { (for the future)
            //     ocr::ocr_segment(img);
            // }
        },
        Err(e) => eprintln!("Error segmenting image: {}", e)
    }
}