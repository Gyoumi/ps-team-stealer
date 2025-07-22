use image::{RgbImage};
use itertools::Itertools;
use super::image_segmenter;
use super::ocr;
use std::time::Instant;
use once_cell::sync::Lazy;
use crate::text::{pokemon::Pokemon, team::Team, text_processor::TEAMS};
use crate::util::load_flag::{AsyncLoadFlag, FALSE, LOADING, TRUE};

pub static IN_BATTLE: Lazy<AsyncLoadFlag> = Lazy::new(|| AsyncLoadFlag::new());

pub async fn process_image(id: usize, image: RgbImage) {
    let segments = image_segmenter::segment_image(id, image);
    
    match segments {
        Ok(images) => {
            // check hover first
            if let Some((label, img)) = images.iter().find(|(key,_)| key.ends_with("hover")) {
                if !images.contains_key("player_hover") {
                    return; // only care about player's mons for now
                }

                IN_BATTLE.wait_until_loaded().await; // no point loading mon if team is not ready

                //let start = Instant::now();
                //let result = ocr::ocr_segment(img.get(0).unwrap()).await;
                //let duration = start.elapsed();

                //println!("ocr_segment() took {:?}", duration);
                let result = ocr::ocrs_ocr(img.get(0).unwrap());
                match result {
                    Ok(text) => println!("OCRS result for {}: {}", label, text),
                    Err(e) => eprintln!("Error OCRing image: {}", e)
                }

                let result2 = ocr::ollama_ocr(img.get(0).unwrap()).await;
                match result2 {
                    Ok(text) => println!("OLLAMA result for {}: {}", label, text),
                    Err(e) => eprintln!("Error OCRing image: {}", e)
                }
                return;
            }

            // get team for each new battle
            if IN_BATTLE.get_state() == FALSE && images.contains_key("battle") {
                IN_BATTLE.set_state(LOADING);
                let (label, img) = images.get_key_value("battle").unwrap();
                let result = ocr::ocrs_ocr(img.get(0).unwrap());
                match result {
                    Ok(text) => {
                        println!("OCRS result for {}: {}", label, text);
                        
                        // get player name

                        if let Some((battle_label, battle_img)) = images.get_key_value("battle") {
                            let result = ocr::ocrs_ocr(battle_img.get(0).unwrap());
                            match result {
                                Ok(text) => {
                                    println!("OCRS result for {}: {}", battle_label, text);
                                    
                                    // get player team
                                    let mons: Vec<Pokemon> = text
                                        .split_whitespace()
                                        .take(6)
                                        .map(|name| Pokemon::new(name))
                                        .collect();
                                    let player_team = Team::new(mons);
                                    let mut team_exists = false;
                                    for team in TEAMS.read().await.iter() {
                                        if team == &player_team {
                                            team_exists = true;
                                            break;
                                        }
                                    }
                                    if !team_exists {
                                        TEAMS.write().await.push(player_team);
                                    }

                                    IN_BATTLE.set_state(TRUE);
                                },
                                Err(e) => {
                                    eprintln!("Error OCRing image: {}", e);
                                    IN_BATTLE.set_state(FALSE);
                                }
                            }
                        }
                    },
                    Err(e) => {
                        eprintln!("Error OCRing image: {}", e);
                        IN_BATTLE.set_state(FALSE);
                    }
                }
                
            }
            // for (label, img) in &images { (for the future)
            //     ocr::ocr_segment(img);
            // }
        },
        Err(e) => eprintln!("Error segmenting image: {}", e)
    }
}