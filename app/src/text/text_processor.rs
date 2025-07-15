use once_cell::sync::Lazy;
use std::sync::Mutex;
use std::collections::HashMap;
use image::RgbImage;
use std::io::Cursor;

static OCR_RESULTS: Lazy<Mutex<HashMap<String, Vec<String>>>> = Lazy::new(|| Mutex::new(HashMap::new()));

pub fn process_text(image_id: String, pokemon_name: String) {
    let mut results = OCR_RESULTS.lock().unwrap();
    results.entry(image_id).or_insert_with(Vec::new).push(pokemon_name);
}

// /// Convert an RgbImage to a base64-encoded PNG string
// pub fn image_to_base64(image: &RgbImage) -> Result<String, Box<dyn std::error::Error>> {
//     // Create a buffer to hold the PNG data
//     let mut buffer = Cursor::new(Vec::new());
    
//     // Encode the image as PNG to the buffer
//     image.write_with_encoder(image::codecs::PngEncoder::new(&mut buffer))?;
    
//     // Get the PNG data as bytes
//     let png_data = buffer.into_inner();
    
//     // Encode to base64
//     let base64_string = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, png_data);
    
//     Ok(base64_string)
// }

// /// Convert an RgbImage to a base64-encoded JPEG string (smaller file size)
// pub fn image_to_base64_jpeg(image: &RgbImage, quality: u8) -> Result<String, Box<dyn std::error::Error>> {
//     // Create a buffer to hold the JPEG data
//     let mut buffer = Cursor::new(Vec::new());
    
//     // Encode the image as JPEG to the buffer
//     image.write_with_encoder(image::codecs::JpegEncoder::new_with_quality(&mut buffer, quality))?;
    
//     // Get the JPEG data as bytes
//     let jpeg_data = buffer.into_inner();
    
//     // Encode to base64
//     let base64_string = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, jpeg_data);
    
//     Ok(base64_string)
// }

// /// Convert an RgbImage to a base64-encoded WebP string (smallest file size)
// pub fn image_to_base64_webp(image: &RgbImage, quality: f32) -> Result<String, Box<dyn std::error::Error>> {
//     // Create a buffer to hold the WebP data
//     let mut buffer = Cursor::new(Vec::new());
    
//     // Encode the image as WebP to the buffer
//     image.write_with_encoder(image::codecs::WebPEncoder::new_with_quality(&mut buffer, quality))?;
    
//     // Get the WebP data as bytes
//     let webp_data = buffer.into_inner();
    
//     // Encode to base64
//     let base64_string = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, webp_data);
    
//     Ok(base64_string)
// }
