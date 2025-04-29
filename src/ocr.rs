// use std::{error::Error, path::Path};

// use image::GrayImage;
// use ocrs::{ImageSource, OcrEngine, OcrEngineParams};
// use rten::Model;

// // pub fn read_frame(frame: GrayImage) -> Result <(), Box<dyn Error>> {

// //     let detection_model_path = Path::new("./src/models/detection-model.rten");
// //     let recognition_model_path = Path::new("./src/models/recognition-model.rten");

// //     let detection_model = Model::load_file(detection_model_path)?;
// //     let recognition_model = Model::load_file(recognition_model_path)?;

// //     let engine_params = OcrEngineParams {
// //                                                     detection_model: Some(detection_model), 
// //                                                     recognition_model: Some(recognition_model),
// //                                                     ..Default::default()
// //                                                     };
// //     let engine = OcrEngine::new(engine_params)?;
// //     let img_source = ImageSource::from_bytes(frame.as_raw(), frame.dimensions())?;
// //     let ocr_input = engine.prepare_input(img_source)?;

// //     let word_rects = engine.detect_words(&ocr_input)?;

// //     let line_rects = engine.find_text_lines(&ocr_input, &word_rects);

// //     let line_texts = engine.recognize_text(&ocr_input, &line_rects)?;

// //     println!("line_texts is of size: {}", line_texts.len());

// //     for line in line_texts
// //         .iter()
// //         .flatten()
// //         .filter(|l| l.to_string().len() > 1) {
// //             println!("{}", line);
// //         }
// //     Ok(())
// // }

// pub fn read_frame(frame: GrayImage) -> Result <(), Box<dyn Error>> {

// }