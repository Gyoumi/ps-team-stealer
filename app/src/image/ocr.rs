use ollama_rs::{Ollama, generation::{completion::request::GenerationRequest, images::Image}};
use image::{RgbImage, ImageEncoder};
use crate::util::error::ModelError;
use std::io::Cursor;
use std::error::Error;

pub async fn ocr_segment(image: &RgbImage) -> Result<String, ModelError> {
    let ollama = Ollama::default();
    println!("Ollama connected");
    let model = "hf.co/openbmb/MiniCPM-o-2_6-gguf:Q4_K_M";
    let prompt = "Please read the text in this image. Please maintain the format in the image including newlines and spaces. Please only include the raw text in the image in your response and not add any additional comments or descriptions. Do not make up any headings or titles. Only include text that is in the image. Please do not hallucinate.";
    println!("OCRing image");
    let base64_image = image_to_base64(image)?;
    println!("Got base64 image");
    match ollama.generate(GenerationRequest::new(String::from(model), prompt).add_image(base64_image)).await {
        Ok(response) => {
            let text = response.response;
            Ok(text)
        }
        Err(e) => {
            eprintln!("Ollama error: {e}");
            let mut source = e.source();
            while let Some(s) = source {
                eprintln!("Caused by: {s}");
                source = s.source();
            }
            Err(ModelError::OllamaError(e.to_string()))
        }
    }
}


fn image_to_base64(img: &RgbImage) -> Result<Image, ModelError> {
    let mut buffer = Cursor::new(Vec::new());
    
    img.write_with_encoder(image::codecs::jpeg::JpegEncoder::new(&mut buffer)).map_err(|e| ModelError::ImageError(e.to_string()))?;
    
    let webp_data = buffer.into_inner();
    
    let base64_string = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, webp_data);
    
    Ok(Image::from_base64(base64_string))
}