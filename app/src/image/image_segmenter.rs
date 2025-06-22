use image::DynamicImage;
use kalosm::vision::*;
use std::{any::Any, env};
use crate::util::error::ModelError;
use tokio::sync::{OnceCell, RwLock};


struct SegmentModel {
    model: OnceCell<Option<SegmentAnything>>
}

impl SegmentModel {
    pub fn new() -> SegmentModel {
        SegmentModel {
            model: OnceCell::new()
        }
    }
    pub async fn get_segmentation_model(&self) -> &Option<SegmentAnything> {
        self.model.get_or_init(|| async {
            SegmentAnything::builder().build().ok()
        }).await
    }
    pub async fn set_segmentation_model(&mut self) {
        if self.get_segmentation_model().await.is_none() {
            self.model.take();
            self.get_segmentation_model().await;
        }
    }
}
pub struct Models {
    seg_models: Vec<SegmentModel>
}

impl Models {
    pub async fn create(size: usize) -> Models {
        let mut seg_models = Vec::new();
        for _ in 0..size {
            let model = SegmentModel::new();
            model.get_segmentation_model().await;
            seg_models.push(model);
        }
        Self {
            seg_models: seg_models
        }
    }

    pub async fn get_segmentation_model(&self, id: usize) -> Result<&SegmentAnything, ModelError> {
        self.seg_models[id].get_segmentation_model().await.as_ref().ok_or(ModelError::SegmentModelLoadError)
    }
}   

static MODELS: OnceCell<RwLock<Models>> = OnceCell::const_new();

async fn get_models() -> &'static RwLock<Models> {
    MODELS.get_or_init(|| async {
    let worker_count = match env::var("WORKER_COUNT") {
        Ok(val) => val.parse::<usize>().unwrap_or(1),
        Err(_) => 1,
    };
    let models = Models::create(worker_count).await;
    RwLock::new(models)
    }).await
}

pub async fn segment_image(id: usize) -> Result<Vec<DynamicImage>, ModelError>{
    // let lock = get_models().await;

    // let mut writer = lock.write().await;
    // let model = &mut writer.seg_models[id];
    // model.set_segmentation_model().await;
    // drop(writer);
    
    // let reader = lock.read().await;
    // let seg_model = reader.get_segmentation_model(id).await?;
    let seg_model = SegmentAnything::builder().build().unwrap();
    
    let image = image::open("./frame_50.png").unwrap();
    use std::time::Instant;
    println!("starting timer...");
    let start = Instant::now();
    let images = seg_model.segment_everything(image)?;
    let duration = start.elapsed();
    println!("get the images!. Took {:?}", duration);
    Ok(images)
}