use image::{RgbImage};
use itertools::Itertools;
use super::image_segmenter;
use super::ocr;
use std::time::Instant;
use once_cell::sync::Lazy;
use crate::text::{pokemon::Pokemon, team::Team, text_processor::TEAMS};
use crate::util::load_flag::{AsyncLoadFlag, FALSE, LOADING, TRUE};
use rust_fuzzy_search::fuzzy_search;
use ordered_float::OrderedFloat;

pub static IN_BATTLE: Lazy<AsyncLoadFlag> = Lazy::new(|| AsyncLoadFlag::new());

pub async fn process_image(id: usize, image: RgbImage) {
    let segments = image_segmenter::segment_image(id, image);
    
    match segments {
        Ok(images) => {
            // check hover first
            if let Some((label, img_vec)) = images.iter().find(|(key,_)| key.ends_with("hover")) {
                if !images.contains_key("pokemon_hover") {
                    return; // only care about player's mons for now
                }

                IN_BATTLE.wait_until_loaded().await; // no point loading mon if team is not ready

                //let start = Instant::now();
                //let result = ocr::ocr_segment(img.get(0).unwrap()).await;
                //let duration = start.elapsed();

                //println!("ocr_segment() took {:?}", duration);

                let label = label.clone(); // clone the key (String)
                let img = img_vec.get(0).unwrap().clone(); // clone the image

                let h1 = tokio::spawn({
                    let label = label.clone();
                    let img = img.clone();
                    async move {
                        let result2 = ocr::ollama_ocr(&img).await;
                        match result2 {
                            Ok(text) => println!("OLLAMA result for {}: {}", label, text),
                            Err(e) => eprintln!("Error OCRing image: {}", e)
                        }
                    }
                });

                let h2 = tokio::spawn({
                    let label = label.clone();
                    let img = img.clone();
                    async move {
                        let result = ocr::ocrs_ocr(&img);
                        match result {
                            Ok(text) => println!("OCRS result for {}: {}", label, text),
                            Err(e) => eprintln!("Error OCRing image: {}", e)
                        }
                    }
                });

                tokio::join!(h1, h2);
                return;
            }

            if id % 1000 == 0 {
                // read chat and update battle every 1000 frames
                // if let Some((chat_label, chat_img)) = images.get_key_value("chat") {
                //     let result = ocr::paddle_crate_ocr(chat_img.get(0).unwrap());
                //     match result {
                //         Ok(text) => {
                //             let lines = text.lines().collect::<Vec<&str>>();
                //         }
                //     }
                // }
            }

            if images.contains_key("battle") && images.contains_key("chat") && !images.contains_key("turn_log") && (BATTLES.read().await.is_empty() || BATTLES.read().await.last().unwrap().get_highest_turn() > 0) {
                IN_BATTLE.set_state(FALSE); // new battle started
            }

            // get team for each new battle
            if IN_BATTLE.get_state() == FALSE && images.contains_key("battle") {
                IN_BATTLE.set_state(LOADING);
                let (label, img) = images.get_key_value("battle").unwrap();
                let result = ocr::paddle_crate_ocr(img.get(0).unwrap());
                match result {
                    Ok(text) => {
                        let players = text.lines().fold(Vec::new(), |mut acc, line| {
                            if line.trim().is_empty() {
                                if acc.last().map_or(false, |s: &String| !s.trim().is_empty()) {
                                    acc.push(String::new());
                                }
                            } else {
                                if acc.is_empty() {
                                    acc.push(line.trim().to_string());
                                } else {
                                    acc.last_mut().unwrap().push_str(&line.trim());
                                }
                            }
                            acc
                        }).into_iter().filter(|s| !s.is_empty()).collect::<Vec<String>>();

                        if let Some(opponent) = players.get(0) {
                            let mut battle = Battle::new(opponent);
                            BATTLES.write().await.push(battle);
                        } else {
                            eprintln!("No opponent found");
                            IN_BATTLE.set_state(FALSE);
                            return;
                        }

                        if let Some(player) = players.get(1) { // player name is in bottom row

                        
                            println!("Players name is: {}", player);

                            if let Some((chat_label, chat_img)) = images.get_key_value("chat") {
                                let result = ocr::paddle_crate_ocr(chat_img.get(0).unwrap());
                                match result {
                                    Ok(text) => {
                                        let lines = text.lines().collect::<Vec<&str>>();
                                        let pattern = format!("{}'s team", player);

                                        let scores =  fuzzy_search(&pattern, &lines);
                                        let lines = lines.clone();
                                        let sorted_lines = lines.into_iter().enumerate().skip(1)
                                        .map(|(i, line)| (line, OrderedFloat(scores[i-1].1)))
                                        .sorted_by_key(|&(_line, prev_score)| -prev_score)
                                        .map(|(line, _)| line).collect::<Vec<&str>>();

                                        let team = sorted_lines.first().unwrap();

                                        // get player team
                                        let mons: Vec<Pokemon> = team.split('/')
                                            .take(6)
                                            .map(|name| Pokemon::new(name.trim()))
                                            .collect::<Vec<Pokemon>>();
                                        let player_team = Team::new(mons);
                                        let mut team_exists = false;
                                        for team in TEAMS.read().await.iter() {
                                            if team == &player_team {
                                                team_exists = true;
                                                break;
                                            }
                                        }

                                        println!("added team");
                                        if !team_exists {
                                            TEAMS.write().await.push(player_team);

                                            println!("printing team names: ");
                                            for mon in &TEAMS.read().await[0].pokemon {
                                                println!("{}", mon.get_name());
                                            }
                                        }

                                        IN_BATTLE.set_state(TRUE);
                                    },
                                    Err(e) => {
                                        eprintln!("Error OCRing image: {}", e);
                                        IN_BATTLE.set_state(FALSE);
                                    }
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