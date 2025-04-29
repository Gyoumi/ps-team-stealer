use std::{error::Error, fmt};
use image::{DynamicImage, Frame, GrayImage, Luma, Pixel, Rgb};

#[derive(Debug)]
pub enum FrameError {
    LiveCheckFailed(u8, u8),
    BattleHeightNotFound,
    StartHeightNotFound,
}

impl fmt::Display for FrameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FrameError::LiveCheckFailed(left, mid) => write!(f, "Failed to complete live check. left_intensity: {left}, mid_intensity: {mid}"),
            FrameError::BattleHeightNotFound => write!(f, "Unable to determine battle frame height."),
            FrameError::StartHeightNotFound => write!(f, "Unable to determine battle frame start height."),
        }
    }
}

impl Error for FrameError{}

fn dark_check(pixel: &Luma<u8>) -> bool {
        let intensity = pixel[0];
        let Rgb([r, g, b]) = pixel.to_rgb();

        intensity <= 100 && (r == g && g == b) 
}

fn light_check(pixel: &Luma<u8>) -> bool {
    let intensity = pixel[0];
    
    intensity >= 230
}

fn check_live<F>(frame: &GrayImage, mode_check: F) -> Result<bool, FrameError> 
where 
    F: Fn(&Luma<u8>) -> bool {

    let height = frame.height();
    let width = frame.width();

    let mid = frame.get_pixel(width / 2, height -1);
    let mid_intensity = mid[0];

    let left = frame.get_pixel(1, height -1);
    let left_intensity = left[0];

    if !mode_check(mid)  || !mode_check(left) {
        return Ok(false);
    }

    if left_intensity != mid_intensity {
        return Err(FrameError::LiveCheckFailed(left_intensity, mid_intensity));
    }

    Ok(true)
}

fn find_battle_height(frame: &GrayImage) -> Result<u32, FrameError>{
    let height = frame.height()-1;
    let bg_intensity = frame.get_pixel(1, height)[0];

    let mut battle_height: Option<u32> = None;

    while height > height / 2 {     // if anyone uploads a showdown video with the battle screen taking up less than half the frame, they are a psychopath.
        let intensity = frame.get_pixel(1, height)[0];

        if battle_height.is_none() && intensity != bg_intensity {
            battle_height = Some(height);
        } else if battle_height.is_some() && intensity == bg_intensity {       // case of left side cropped to first mon in party at far left of screen
            battle_height = None;
        }
    }

    battle_height.ok_or(FrameError::BattleHeightNotFound)
}

fn find_battle_start(frame: &GrayImage) -> Result<u32, FrameError> {
    let top_intensity = frame.get_pixel(0, 0)[0];
    let height = frame.height() / 2;
    
    let top_bar = 8;     // height of top bar is 8 pixels

    let mut start = None;

    for i in 1..height {
        let intensity = frame.get_pixel(0, i)[0];

        if start.is_none() && intensity != top_intensity {
            start = Some(height);
        } else if start.is_some() && intensity == top_intensity {
            start = None;
        }
    }

    match start {
        Some(start_height) => Ok(start_height + top_bar),
        None => Err(FrameError::StartHeightNotFound),
    }
}

fn find_battle_width(frame: &GrayImage, top_height: u32) -> Result<u32, FrameError> {
    let width = frame.width()-1;
    let bg_intensity = frame.get_pixel(width, top_height)[0];

    Ok(0)
}

pub fn process_image(image: &GrayImage) -> Result<(), FrameError> {
    // 1. check to see if video is a live or replay
    let is_live = check_live(image, dark_check)? || check_live(image, light_check)?;

    if is_live {
        // 2. identify height/width of battle
        let frame_height = find_battle_height(image)?;
        let start_height = find_battle_start(image)?;
    } 

    Ok(())
}