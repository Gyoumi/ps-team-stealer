use std::cmp::max;
use rust_fuzzy_search::fuzzy_compare;
use crate::text::{pokeapi::{get_stat_enum, get_stat_string, Stat, NATURES, POKEMON_SPECIES}, text_processor::{validate_type, validate_ability, validate_item, validate_move, capitalise_first}};

#[derive(Default, Debug)]
pub struct Pokemon {
    name: String,
    nickname: Option<String>,
    level: Option<u8>,
    ability: Option<String>,
    item: Option<String>,
    nature: Option<String>,
    pos_nature: Option<String>,
    neg_nature: Option<String>,
    tera: Option<String>,
    hp: u8,
    attack: u8,
    defense: u8,
    special_attack: u8,
    special_defense: u8,
    speed: u8,
    moves: Vec<String>,
    remaining_evs: u16,
    hp_iv: u8,
    attack_iv: u8,
    defense_iv: u8,
    special_attack_iv: u8,
    special_defense_iv: u8,
    speed_iv: u8,
}

impl Pokemon {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            nickname: None,
            level: None,
            ability: None,
            item: None,
            nature: None,
            pos_nature: None,
            neg_nature: None,
            tera: None,
            hp: 0,
            attack: 0,
            defense: 0,
            special_attack: 0,
            special_defense: 0,
            speed: 0,
            moves: Vec::new(),
            remaining_evs: 508,
            hp_iv: 31,
            attack_iv: 31,
            defense_iv: 31,
            special_attack_iv: 31,
            special_defense_iv: 31,
            speed_iv: 31,
        }
    }

    pub fn set_nickname(&mut self, name1: &str, name2: Option<&str>) {
        if let Some(name2) = name2 {
            let compare = fuzzy_compare(name2, name1);
            if compare < 0.7 {
                let compare1 = fuzzy_compare(name1, self.name.as_str());
                let compare2 = fuzzy_compare(name2, self.name.as_str());
                if compare1 > compare2 {
                    self.nickname = Some(name2.to_string());
                } else {
                    self.nickname = Some(name1.to_string());
                }
            }
        } 
    }

    pub fn set_ability(&mut self, ability: &str) -> bool {
        let validated = validate_ability(ability);
        if validated.is_none() {
            return false;
        }
        self.ability = Some(validated.unwrap());
        true
    }

    pub fn set_item(&mut self, item: Option<&str>) -> bool {
        if let Some(item) = item {
            let validated = validate_item(item);
            if validated.is_none() {
                return false;
            }
            self.item = Some(validated.unwrap());
        }
        true
    }

    pub fn set_tera(&mut self, tera: Option<&str>) -> bool {
        if let Some(tera) = tera {
            let validated = validate_type(tera);
            if validated.is_none() {
                return false;           // tera type exists but is invalid. i.e. value misread
            }
            self.tera = Some(validated.unwrap());
        }
        true
    }

    pub fn set_hp(&mut self, hp1: u32, hp2: u32) -> bool {
        let raw_hp = max(hp1, hp2);
        let base_stats = match POKEMON_SPECIES.get() {
            Some(stats) => stats,
            None => {
                eprintln!("[set_hp] POKEMON_SPECIES not initialized");
                return false;
            }
        };
        let species = match base_stats.get(self.name.to_lowercase().as_str()) {
            Some(species) => species,
            None => {
                eprintln!("[set_hp] Species '{}' not found in POKEMON_SPECIES", self.name);
                return false;
            }
        };
        let base_hp = match species.base_stats.get(&Stat::HP) {
            Some(hp) => hp,
            None => {
                eprintln!("[set_hp] Stat::HP not found for species '{}'", self.name);
                return false;
            }
        };
        println!("raw_hp: {}, base_hp: {}, min_hp: {}, max_hp: {}", raw_hp, base_hp, min_hp(base_hp), max_hp(&base_hp));
        if raw_hp < min_hp(base_hp) || raw_hp > max_hp(&base_hp) {
            return false;
        }
        if raw_hp > *base_hp {
            let evs: u16 = ((raw_hp - *base_hp) * 4).try_into().unwrap_or(0);
            self.hp = evs as u8;
            self.remaining_evs -= evs as u16;
        } else {
            self.hp_iv = (31 - *base_hp + raw_hp).try_into().unwrap_or(0); // assuming no evs if iv is lower than 31
        }
        true
    }

    fn set_stat(&mut self, stat: Stat, raw_value: u32) -> bool {
        let base_stats = match POKEMON_SPECIES.get() {
            Some(stats) => stats,
            None => {
                println!("[set_stat] POKEMON_SPECIES not initialized");
                return false;
            }
        };
        let species = match base_stats.get(self.name.to_lowercase().as_str()) {
            Some(species) => species,
            None => {
                println!("[set_stat] Species '{}' not found in POKEMON_SPECIES", self.name);
                return false;
            }
        };
        let base_stat = match species.base_stats.get(&stat) {
            Some(val) => val,
            None => {
                println!("[set_stat] Stat {:?} not found for species '{}'", stat, self.name);
                return false;
            }
        };

        let raw_stat = self.remove_modifer(raw_value, stat.clone());
        //println!("stat: {:?}, raw_value: {}, raw_stat: {}, base_stat: {}, min_stat: {}, max_stat: {}", stat, raw_value, raw_stat, base_stat, min_stat(base_stat), max_stat(base_stat));
        if raw_stat < min_stat(base_stat) || raw_stat > max_stat(base_stat) {
            return false;
        }
        
        if raw_stat > *base_stat {
            let mut evs: u16 = ((raw_stat - *base_stat) * 4).try_into().unwrap_or(0);
            if evs > u16::min(252, self.remaining_evs) { // positive nature
                evs = ((raw_stat as f32 / 1.1).ceil() as u16 - *base_stat as u16) * 4;
                self.pos_nature = Some(get_stat_string(stat.clone()));
            }
            match stat {
                Stat::Attack => self.attack = evs as u8,
                Stat::Defense => self.defense = evs as u8,
                Stat::SpecialAttack => self.special_attack = evs as u8,
                Stat::SpecialDefense => self.special_defense = evs as u8,
                Stat::Speed => self.speed = evs as u8,
                _ => {}
            }
            self.remaining_evs -= evs as u16;
        } else {
            if let Some(_) = self.neg_nature.as_ref() {
                self.update_iv(*base_stat, raw_stat, stat);
            } else {
                let neg_stat = (*base_stat as f32 * 0.9).floor() as u32;

                match neg_stat {
                    _ if neg_stat == raw_stat => {
                        self.neg_nature = Some(get_stat_string(stat.clone()));
                    }
                    _ if neg_stat > raw_stat => {
                        self.neg_nature = Some(get_stat_string(stat.clone())); 
                        self.update_iv(*base_stat, raw_stat, stat);
                    }
                    _ => {
                        self.update_iv(*base_stat, raw_stat, stat);
                    }
                }
            }
        }

        if let Some(pos_nature) = self.pos_nature.as_ref() {
            if let Some(neg_nature) = self.neg_nature.as_ref() {
                let dec_stat = get_stat_enum(neg_nature);
                let inc_stat = get_stat_enum(pos_nature);
                let nature = NATURES.get().unwrap().iter()
                    .find(|n| n.increased_stat == inc_stat && n.decreased_stat == dec_stat);
                if let Some(nature) = nature {
                    self.nature = Some(nature.name.clone());
                } else {
                    return false;
                }
            }
        }
        true
    }

    fn update_iv(&mut self, base_stat: u32, raw_stat: u32, stat: Stat) {
        let iv = (31 - base_stat + raw_stat).try_into().unwrap_or(0);
        match stat {
            Stat::Attack => self.attack_iv = iv,
            Stat::Defense => self.defense_iv = iv,
            Stat::SpecialAttack => self.special_attack_iv = iv,
            Stat::SpecialDefense => self.special_defense_iv = iv,
            Stat::Speed => self.speed_iv = iv,
            _ => {}
        }
    }

    pub fn set_attack(&mut self, raw_attack: u32) -> bool {
        self.set_stat(Stat::Attack, raw_attack)
    }

    pub fn set_defense(&mut self, raw_defense: u32) -> bool {
        self.set_stat(Stat::Defense, raw_defense)
    }

    pub fn set_special_attack(&mut self, raw_spatk: u32) -> bool {
        self.set_stat(Stat::SpecialAttack, raw_spatk)
    }

    pub fn set_special_defense(&mut self, raw_spdef: u32) -> bool {
        self.set_stat(Stat::SpecialDefense, raw_spdef)
    }

    pub fn set_speed(&mut self, raw_speed: u32) -> bool {
        self.set_stat(Stat::Speed, raw_speed)
    }

    pub fn set_moves(&mut self, moves: Vec<String>) -> bool {
        let mut completed = true;
        for mv in moves {
            let validated = validate_move(&mv);
            if let Some(validated) = validated {
                if !self.moves.contains(&validated) && self.moves.len() < 4 {
                    self.moves.push(validated);
                }
            } else {
                completed = false;
            }
        }
        completed
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_nickname(&self) -> Option<&str> {
        self.nickname.as_deref()
    }

    pub fn get_level(&self) -> Option<u8> {
        self.level
    }
    
    pub fn get_ability(&self) -> Option<&str> {
        self.ability.as_deref()
    }

    pub fn get_item(&self) -> Option<&str> {
        self.item.as_deref()
    }   

    pub fn get_nature(&self) -> Option<&str> {
        self.nature.as_deref()
    }

    pub fn get_pos_nature(&self) -> Option<&str> {
        self.pos_nature.as_deref()
    }

    pub fn get_neg_nature(&self) -> Option<&str> {
        self.neg_nature.as_deref()
    }

    pub fn get_tera(&self) -> Option<&str> {    
        self.tera.as_deref()
    }

    pub fn get_hp(&self) -> u8 {
        self.hp
    }           

    pub fn get_attack(&self) -> u8 {
        self.attack
    }

    pub fn get_defense(&self) -> u8 {   
        self.defense
    }

    pub fn get_special_attack(&self) -> u8 {
        self.special_attack
    }       

    pub fn get_special_defense(&self) -> u8 {
        self.special_defense
    }

    pub fn get_speed(&self) -> u8 { 
        self.speed
    }

    pub fn get_moves(&self) -> &Vec<String> {
        &self.moves
    } 

    pub fn complete(&self) -> bool {
        self.ability.is_some() &&
        self.nature.is_some() &&
        self.remaining_evs != 508 &&
        !self.moves.is_empty()
    }

    pub fn to_import_string(&self) -> String {
        let name_segment = 
        match &self.nickname {
            Some(nickname) => format!("{} ({})", nickname, capitalise_first(&self.name)),
            None => format!("{}", capitalise_first(&self.name)),
        };

        let item_segment = 
        match &self.item {
            Some(item) => format!("@ {}", item),
            None => String::new(),
        };

        let first_line = format!("{} {}\n", name_segment, item_segment);
        let ability_line = match &self.ability {
            Some(ability) => format!("Ability: {}\n", ability),
            None => String::new(),
        };
        let level_line: String = match &self.level {
            Some(level) => format!("Level: {}\n", level),
            None => String::new(),
        };
        let tera_line = 
            match &self.tera {
                Some(tera) => format!("Tera Type: {}\n", capitalise_first(&tera)),
                None => String::new(),
            };
        let ev_line = 
            match self.remaining_evs {
                508 => String::new(),
                _ => format!("EVs: {}\n", self.get_ev_spread()),
            };
        let nature_line = 
            match & self.nature {
                Some(nature) => format!("{} Nature\n", capitalise_first(nature)),
                None => String::new(),
            };
        let iv_line = 
            match self.get_iv_spread() {
                Some(ivs) => format!("IVs: {}\n", ivs),
                None => String::new(),
            };
        let moves_lines = self.moves.iter().map(|mv| format!("- {}\n", mv)).collect::<Vec<String>>();

        format!("{}{}{}{}{}{}{}{}\n", first_line, ability_line, level_line, tera_line, ev_line, nature_line, iv_line, moves_lines.join(""))
    }

    fn get_ev_spread(&self) -> String {
        let mut res = String::new();

        if self.hp > 0 {
            res.push_str(&format!("{} HP / ", self.hp));
        }

        if self.attack > 0 {
            res.push_str(&format!("{} Atk / ", self.attack));
        }

        if self.defense > 0 {
            res.push_str(&format!("{} Def / ", self.defense));
        }

        if self.special_attack > 0 {
            res.push_str(&format!("{} SpA / ", self.special_attack));
        }

        if self.special_defense > 0 {
            res.push_str(&format!("{} SpD / ", self.special_defense));
        }

        if self.speed > 0 {
            res.push_str(&format!("{} Spe / ", self.speed));
        }

        res.pop();
        res.pop();
        res
    }

    fn get_iv_spread(&self) -> Option<String> {
        let mut res = String::new();

        if self.hp_iv != 31 {
            res.push_str(&format!("{} HP / ", self.hp_iv));
        }

        if self.attack_iv != 31 {
            res.push_str(&format!("{} Atk / ", self.attack_iv));
        }

        if self.defense_iv != 31 {
            res.push_str(&format!("{} Def / ", self.defense_iv));
        }

        if self.special_attack_iv != 31 {
            res.push_str(&format!("{} SpA / ", self.special_attack_iv));
        }

        if self.special_defense_iv != 31 {
            res.push_str(&format!("{} SpD / ", self.special_defense_iv));
        }

        if self.speed_iv != 31 {
            res.push_str(&format!("{} Spe / ", self.speed_iv));
        }

        if res.is_empty() {
            return None;
        }

        res.pop();
        res.pop();
        Some(res)
    }

    fn remove_modifer(&self, stat_value: u32, stat_type: Stat) -> u32 {
        let item =  match self.item.as_ref() {
            Some(item) => Some(item.to_lowercase().trim().split_whitespace().collect::<Vec<&str>>().join("-")),
            None => None,
        };
        match stat_type {
            Stat::Attack => {
                if let Some(item) = item {
                    if item == "choice-band" {
                        return stat_value / 1.5 as u32;
                    }
                }
                stat_value
            },
            Stat::Defense => {
                let mut updated_stat = stat_value;
                if let Some(item) = item {
                    if item == "eviolite" {     // assuming all eviolite holder can still evolve (would fail if item was tricked)
                        updated_stat = stat_value / 1.5 as u32;
                    }
                }
                // if self.types.contains("ice") {  // TODO: add types to pokemon struct
                    // let weathers = battle::get_weathers_for_turn(turn); // TODO: add battle and turn as input values
                    // if weathers.contains(weather::Snow) {
                    //     updated_stat = updated_stat / 1.5 as u16;
                    // }
                // }
                    updated_stat
            },
            Stat::SpecialAttack => {
                if let Some(item) = item {
                    if item == "choice-specs" {
                        return stat_value / 1.5 as u32;
                    }
                }
                stat_value
            },
            Stat::SpecialDefense => {
                let mut updated_stat = stat_value;
                if let Some(item) = item {
                    if item == "assault-vest" || item == "eviolite" {
                        updated_stat = stat_value / 1.5 as u32;
                    }
                }
                // if self.types.contains("rock") {  // TODO: add types to pokemon struct
                    // let weathers = battle::get_weathers_for_turn(turn); // TODO: add battle and turn as input values
                    // if weathers.contains(weather::Sand) {
                    //     updated_stat = updated_stat / 1.5 as u16;
                    // }
                // }
                updated_stat
            },
            Stat::Speed => {
                let mut updated_stat = stat_value;
                if let Some(item) = item {
                    if item == "choice-scarf" {
                        return stat_value / 1.5 as u32;
                    }
                }
                // let ability = self.ability.to_lowercase().trim().split_whitespace().collect::<Vec<&str>>().join("-");
                // let weathers = battle::get_weathers_for_turn(turn); // TODO: add battle and turn as input values
                // if ability == "chlorophyll" && weathers.contains(weather::Sun) {
                    // updated_stat = updated_stat / 2 as u16;
                // }
                // if ability == "swift-swim" && weathers.contains(weather::Rain) {
                    // updated_stat = updated_stat / 2 as u16;
                // }
                // if ability == "slush-rush" && (weathers.contains(weather::Hail) || weathers.contains(weather::Snow)) {
                    // updated_stat = updated_stat / 2 as u16;
                // }
                // if ability == "sand-rush" && weathers.contains(weather::Sand) {
                    // updated_stat = updated_stat / 2 as u16;
                // }
                // if weathers.contains(weather::Tailwind) {
                    // updated_stat = updated_stat * 2 as u16;
                // }
                updated_stat
            },
            _ => stat_value,
        }
    }
}

fn min_stat(stat: &u32) -> u32 {
    ((*stat as f32 - 31.0) * 0.9).floor() as u32
}

fn min_hp(hp: &u32) -> u32 {
    *hp - 31
}

fn max_stat(stat: &u32) -> u32 {
    ((*stat as f32 + (252.0 / 4.0)) * 1.1).floor() as u32
}

fn max_hp(hp: &u32) -> u32 {
    *hp + (252 / 4)
}

