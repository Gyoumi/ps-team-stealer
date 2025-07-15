use ort::{environment::Environment, session::{Session, builder::GraphOptimizationLevel, output::SessionOutputs}, value::Value};
use image::{imageops::{resize, FilterType}, ImageBuffer, Rgb, RgbImage, GenericImageView};
use ndarray::{Array, IxDyn, CowArray, Ix4, s, Axis, ArrayViewD};
use crate::util::error::ModelError;
use std::{sync::Arc, env, collections::HashMap};
use imageproc::drawing::draw_text_mut;
use once_cell::sync::Lazy;

pub static MODEL_PATH: Lazy<String> = Lazy::new(|| {
    let model = env::var("YOLO_MODEL").unwrap_or_else(|_| String::from("yolo_custom"));
    format!("src/image/model/{}.onnx", model)
});

pub static LABELS: Lazy<Vec<String>> = Lazy::new(|| {
    let raw = env::var("LABELS").unwrap_or_default();
    println!("LABELS raw from env: {:?}", raw);
    raw.split(',')
        .map(|s| s.trim().to_string())
        .collect()
});

fn get_model_path() -> &'static str {
    MODEL_PATH.as_str()
}

fn get_labels() -> Vec<String> {
    LABELS.to_vec()
}

#[derive(Clone, Debug, Copy)]
struct BoundingBox {
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    class_id: usize,
    confidence: f32,
}
impl BoundingBox {
    fn new(x1: f32, y1: f32, x2: f32, y2: f32, class_id: usize, confidence: f32) -> Self {
        Self { x1, y1, x2, y2, class_id, confidence }
    }
    fn intersection(&self, bx: &BoundingBox) -> f32 {
        (self.x2.min(bx.x2) - self.x1.max(bx.x1)) * (self.y2.min(bx.y2) - self.y1.max(bx.y1))
    }
    
    fn union(&self, bx: &BoundingBox) -> f32 {
        ((self.x2 - self.x1) * (self.y2 - self.y1)) + ((bx.x2 - bx.x1) * (bx.y2 - bx.y1))
            - self.intersection(bx)
    }
}

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

fn draw_boxes(mut img: RgbImage, boxes: &[BoundingBox], height: u32, width: u32) -> RgbImage {
    for bbox in boxes {
        let color = Rgb([255, 0, 0]);
        for x in bbox.x1 as u32..bbox.x2 as u32 {
            if x >= 0 && x < width && (bbox.y1 as u32) >= 0 && (bbox.y1 as u32) < height && (bbox.y2 as u32) >= 0 && (bbox.y2 as u32) < height {
                img.put_pixel(x, bbox.y1 as u32, color);
                img.put_pixel(x, bbox.y2 as u32, color);
            }
        }

        for y in bbox.y1 as u32..bbox.y2 as u32 {
            if y >= 0 && y < height && (bbox.x1 as u32) >= 0 && (bbox.x1 as u32) < width && (bbox.x2 as u32) >= 0 && (bbox.x2 as u32) < width {
                img.put_pixel(bbox.x1 as u32, y, color);
                img.put_pixel(bbox.x2 as u32, y, color);
            }
        }
    }
    img
}

pub fn crop_segments_by_label(
    image: &RgbImage,
    boxes: &[BoundingBox],
    labels: Vec<String>,
) -> HashMap<String, Vec<RgbImage>> {
    let mut map: HashMap<String, Vec<RgbImage>> = HashMap::new();

    for bbox in boxes {
        let x1 = bbox.x1.max(0.0).floor() as u32;
        let y1 = bbox.y1.max(0.0).floor() as u32;
        let x2 = bbox.x2.min(image.width() as f32).ceil() as u32;
        let y2 = bbox.y2.min(image.height() as f32).ceil() as u32;

        if x2 <= x1 || y2 <= y1 {
            continue; 
        }

        let sub_image = image.view(x1, y1, x2 - x1, y2 - y1).to_image();
        let label = labels.get(bbox.class_id).map(|s| s.as_str()).unwrap_or("unknown").to_string();


        map.entry(label).or_default().push(sub_image);
    }

    map
}

fn postprocess_output(output: &ArrayViewD<f32>, img_width: u32, img_height: u32) -> Vec<BoundingBox> { 
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

            if prob < 0.4 {
                continue;
            }

            let xc = row[0] / 640. * (img_width as f32);
            let yc = row[1] / 640. * (img_height as f32);
            let w = row[2] / 640. * (img_width as f32);
            let h = row[3] / 640. * (img_height as f32);

            boxes.push(BoundingBox::new(
                xc - w / 2.,
                yc - h / 2.,
                xc + w / 2.,
                yc + h / 2.,
                class_id.try_into().unwrap(),
                prob,
            ));
        }

        let mut sorted = boxes.clone();
        sorted.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
        let mut res = Vec::new();
        
        while !sorted.is_empty() {
            let best = sorted.remove(0);
            res.push(best);
        
            sorted.retain(|b| {
                if b.class_id != best.class_id {
                    return true;
                }
        
                let inter = best.intersection(b);
                let union = best.union(b);
        
                if inter <= 0.0 || union <= 0.0 {
                    return true; 
                }
        
                let iou = inter / union;
        
                iou < 0.7
            });
        }
        
        res
        
}

pub fn segment_image(id: usize, image: RgbImage) -> Result<HashMap<String, Vec<RgbImage>>, ModelError> {
    println!("Segmenting image {} with model {}", id, MODEL_PATH.as_str());
    let og_image = image.clone();
    let og_image_2 = image.clone();
    let (input_tensor, og_width, og_height) = rgb_image_to_tensor(image, 640, 640)?;
    let session = Session::builder()?
    .with_optimization_level(GraphOptimizationLevel::Level3)?
    .commit_from_file(get_model_path())?;

    let input = ort::inputs![input_tensor.view()]?;
    let outputs: SessionOutputs = session.run(input)?;

    let output_array = outputs["output0"].try_extract_tensor::<f32>().unwrap().t().into_owned();
    println!("Shape: {:?}", output_array.view().shape());


    let boxes = postprocess_output(&output_array.view(), og_width, og_height);
    println!("Detected {} boxes", boxes.len());

    let output_img = draw_boxes(og_image, &boxes, og_height, og_width);
    output_img.save(format!("segment/segmented_{}.png", id))?;

    let segments = crop_segments_by_label(&og_image_2, &boxes, get_labels());
    for (label, segment) in segments.clone() {
        println!("Segmented {} segments for label {}", segment.len(), label);
        for (_, s) in segment.iter().enumerate() {
            s.save(format!("segment/segmented_{}_{}.png", id, label))?;
        }
    }

    Ok(segments)
}