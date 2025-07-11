use ort::{environment::Environment, session::{Session, builder::GraphOptimizationLevel, output::SessionOutputs}, value::Value};
use image::{imageops::{resize, FilterType}, ImageBuffer, Rgb, RgbImage};
use ndarray::{Array, IxDyn, CowArray, Ix4, s, Axis};
use crate::util::error::ModelError;
use std::sync::Arc;
use imageproc::drawing::draw_text_mut;

use ndarray::ArrayViewD;

struct BoundingBox {
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    class_id: usize,
    confidence: f32,
}

/// Convert an RgbImage to a CHW float32 tensor normalized to 0-1.
fn rgb_image_to_tensor(image: RgbImage, width: u32, height: u32) -> Result<(Array<f32, Ix4>, u32, u32), ModelError> {
    let (og_width, og_height) = (image.width(), image.height());
    let resized = resize(&image, width, height, FilterType::CatmullRom);

    let mut tensor = Array::zeros((1, 3, height as usize, width as usize));
    for (x, y, pixel) in resized.enumerate_pixels() {
        let x = x as usize;
        let y = y as usize;
        let r = pixel[0] as f32 / 255.0;
        let g = pixel[1] as f32 / 255.0;
        let b = pixel[2] as f32 / 255.0;
        tensor[[0, 0, y, x]] = r;
        tensor[[0, 1, y, x]] = g;
        tensor[[0, 2, y, x]] = b;
    }

    Ok((tensor, og_width, og_height))
}

/// Draw bounding boxes on an RgbImage.
fn draw_boxes(mut img: RgbImage, boxes: &[BoundingBox]) -> RgbImage {
    for bbox in boxes {
        let color = Rgb([255, 0, 0]);
        for x in bbox.x1 as u32..bbox.x2 as u32 {
            if (bbox.y1 as u32) >= 0 && (bbox.y1 as u32) < img.height() {
                img.put_pixel(x, bbox.y1 as u32, color);
                img.put_pixel(x, bbox.y2 as u32, color);
            }
        }
        for y in bbox.y1 as u32..bbox.y2 as u32 {
            if (bbox.x1 as u32) >= 0 && (bbox.x1 as u32) < img.width() {
                img.put_pixel(bbox.x1 as u32, y, color);
                img.put_pixel(bbox.x2 as u32, y, color);
            }
        }
    }
    img
}

/// Simple postprocessing: convert YOLOv8 output to boxes.
fn postprocess_output(output: &ArrayViewD<f32>, img_width: u32, img_height: u32) -> Vec<BoundingBox> { //(x1, y1, x2, y2, class_id, confidence)
    // let shape = output.shape();
    // let num_classes = shape[1] - 5;
    // let num_anchors = shape[2];
    // let mut boxes = Vec::new();
    // let confidence_threshold = 0.05;

    // for anchor in 0..num_anchors {
    //     let x = output[[0, 0, anchor]];
    //     let y = output[[0, 1, anchor]];
    //     let w = output[[0, 2, anchor]];
    //     let h = output[[0, 3, anchor]];
    //     let obj_conf = output[[0, 4, anchor]];

    //     // Class scores
    //     let mut best_class = 0;
    //     let mut best_score = 0.0;
    //     for class_idx in 0..num_classes {
    //         let score = output[[0, 5 + class_idx, anchor]];
    //         if score > best_score {
    //             best_score = score;
    //             best_class = class_idx;
    //         }
    //     }

    //     let confidence = obj_conf * best_score;

    //     if confidence > confidence_threshold {
    //         let x1 = x - w / 2.0;
    //         let y1 = y - h / 2.0;
    //         let x2 = x + w / 2.0;
    //         let y2 = y + h / 2.0;
    //         boxes.push((x1, y1, x2, y2, best_class, confidence));
    //     }
    // }

    // boxes
    let mut boxes = Vec::new();
        let output = output.slice(s![.., .., 0]);

        for row in output.axis_iter(Axis(0)) {
            let row: Vec<_> = row.iter().copied().collect();
            let (class_id, prob) = row
                .iter()
                .skip(4)
                .enumerate()
                .map(|(index, value)| (index, *value))
                .reduce(|accum, row| if row.1 > accum.1 { row } else { accum })
                .unwrap();

            if prob < 0.25 {
                continue;
            }

            let xc = row[0] / 640. * (img_width as f32);
            let yc = row[1] / 640. * (img_height as f32);
            let w = row[2] / 640. * (img_width as f32);
            let h = row[3] / 640. * (img_height as f32);

            boxes.push(BoundingBox {
                x1: xc - w / 2.,
                y1: yc - h / 2.,
                x2: xc + w / 2.,
                y2: yc + h / 2.,
                class_id: class_id.try_into().unwrap(),
                confidence: prob,
            });
        }

    boxes
}


pub fn segment_image(id: usize) -> Result<RgbImage, ModelError> { //Result<HashMap<String, RgbImage>, ModelError> {
    let image = image::open("frame_50.png")?.to_rgb8();


    let (input_tensor, og_width, og_height) = rgb_image_to_tensor(image, 640, 640)?;
    let session = Session::builder()?
    .with_optimization_level(GraphOptimizationLevel::Level3)?
    .commit_from_file("src/image/model/yolo_custom.onnx")?;

    let input = ort::inputs![input_tensor.view()]?;
    let outputs: SessionOutputs = session.run(input)?;

    // Print raw model output
    let output_array = outputs["output0"].try_extract_tensor::<f32>().unwrap().t().into_owned();
    println!("Shape: {:?}", output_array.view().shape());


    // Postprocess boxes
    let boxes = postprocess_output(&output_array.view(), og_width, og_height);
    println!("Detected {} boxes", boxes.len());

    // Draw and save
    let og_image = image::open("frame_50.png")?.to_rgb8();
    let output_img = draw_boxes(og_image, &boxes);
    output_img.save("segment/segmented.png")?;

    Ok(output_img)
}