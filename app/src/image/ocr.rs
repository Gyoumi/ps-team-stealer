use std::io::Cursor;
use ollama_rs::{Ollama, generation::{completion::request::GenerationRequest, images::Image}};
use kalosm::vision::*;
use ocrs::{ImageSource, OcrEngine, OcrEngineParams};
use rten::Model;
use std::path::PathBuf;
use ort::session::{Session, builder::GraphOptimizationLevel};
#[allow(unused)]

use crate::util::error::ModelError;
use std::error::Error;
use image::{RgbImage, ImageEncoder, DynamicImage, GenericImageView, RgbaImage};
use itertools::Itertools;
use paddle_ocr_rs::ocr_lite::OcrLite;

pub async fn ocr_segment(image: &RgbImage) -> Result<String, ModelError> {
    //let ocr_result = ollama_ocr(image).await?;
    //Ok(ocr_result)
    //kalosm_ocr(image).await
    ocrs_ocr(image)
    //paddle_crate_ocr(image)
}

pub fn paddle_crate_ocr(image: &RgbImage) -> Result<String, ModelError> {
    let mut ocr = OcrLite::new();

    ocr.init_models(
        "src/image/model/detection.onnx",
        "src/image/model/cls.onnx",
        "src/image/model/rec2.onnx",
        2,
    )?;

    println!("detecting text");
    let result = ocr.detect(
        image,
        50,
        1024,
        0.5,
        0.3,
        1.6,
        false,
        false,
    )?;

    Ok(result.text_blocks.iter().map(|item| item.text.clone()).join("\n"))
}

pub fn ocrs_ocr(img: &RgbImage) -> Result<String, ModelError> {
    let mut detection_path = PathBuf::new();
    let mut recognition_path = PathBuf::new();

    detection_path.push("src/image/model/text-detection.rten");
    recognition_path.push("src/image/model/text-recognition.rten");

    let detection_model = Model::load_file(detection_path)?;
    let recognition_model = Model::load_file(recognition_path)?;

    let engine = OcrEngine::new(OcrEngineParams {
        detection_model: Some(detection_model),
        recognition_model: Some(recognition_model),
        ..Default::default()
    })?;

    let img_source = ImageSource::from_bytes(img.as_raw(), img.dimensions())?;
    let ocr_input = engine.prepare_input(img_source)?;

    let word_rects = engine.detect_words(&ocr_input)?;

    let line_rects = engine.find_text_lines(&ocr_input, &word_rects);

    let line_texts = engine.recognize_text(&ocr_input, &line_rects)?;

    Ok(
        line_texts
            .iter()
            .flatten()
            .filter_map(|text| {
                let s = text.to_string();
                if s.is_empty() { None } else { Some(s) }
            })
            .join("\n")
    )
}

// async fn kalosm_ocr(image: &RgbImage) -> Result<String, ModelError> {
//     println!("Building Kalosm model");
//     let mut model = Ocr::builder().build().await?;
//     let rgba_image: RgbaImage = DynamicImage::ImageRgb8(image.clone()).to_rgba8();
//     println!("ocring image");
//     let ocr_result = model.recognize_text(OcrInferenceSettings::new(rgba_image))?;
//     Ok(ocr_result)
// }


pub async fn ollama_ocr(image: &RgbImage) -> Result<String, ModelError> {
    let ollama = Ollama::default();
    //println!("Ollama connected");
    let model = "hf.co/openbmb/MiniCPM-o-2_6-gguf:Q4_K_M";
    let prompt = "Please fill the contents of this JSON based on the text in the image. 
    {
       name: string,
       nickname: string | null,
       types: string[],
       tera_type: string,
       max_hp: number,
       remaining_hp: number,
       remaining_hp_percentage: number,
       ability: string,
       item: string | null,
       atk: number,
       def: number,
       spa: number,
       spd: number,
       spe: number,
       spe_range: boolean,
       moves: string[],    
    } 
    The 'spe_range' field should be true if 'Spe' is listed as a range between two numbers such as 'Spe {num1} to {num2}'. 
    It should be false if 'Spe' only has a single number. 
    Please ensure that all stats are prior to any modifiers. 
    Please only output the JSON. Please do not hallucinate.";
    //println!("OCRing image");
    let base64_image = image_to_base64(image)?;
    //println!("Got base64 image");
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