use std::collections::{HashMap, HashSet};
use crate::text::pokeapi::{ABILITIES, ITEMS, MOVES, NATURES, TYPES, POKEMON_SPECIES, PokemonBaseStat,Stat, get_stat_enum};

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

    pub fn set_nickname(&mut self, nickname: String) {
        self.nickname = Some(nickname);
    }

    pub fn set_level(&mut self, level: u8) {
        self.level = Some(level);
    }

    pub fn set_ability(&mut self, ability: String) {
        if !ABILITIES.get().unwrap().contains(&ability) {
            return;
        }
        self.ability = Some(ability);
    }

    pub fn set_item(&mut self, item: String) {
        if !ITEMS.get().unwrap().contains(&item) {
            return;
        }
        self.item = Some(item);
    }

    pub fn set_nature(&mut self, nature: &str) {
        self.nature = Some(nature.to_string());
    }

    pub fn set_tera(&mut self, tera: String) {
        if !TYPES.get().unwrap().contains(&tera) {
            return;
        }
        self.tera = Some(tera);
    }

    pub fn set_hp(&mut self, raw_hp: u32) {
        let base_hp = POKEMON_SPECIES.get().unwrap().get(self.name.as_str()).unwrap().base_stats.get(&Stat::HP).unwrap();
        if raw_hp < min_hp(base_hp) || raw_hp > max_hp(&base_hp) {
            return;
        }
        if raw_hp > *base_hp {
            let evs: u16 = ((raw_hp - *base_hp) * 4).try_into().unwrap_or(0);
            self.hp = evs as u8;
            self.remaining_evs -= evs as u16;
        } else {
            self.hp_iv = (*base_hp - raw_hp).try_into().unwrap_or(0);
        }
    }

    pub fn set_attack(&mut self, raw_attack: u32) {
        let base_attack = POKEMON_SPECIES.get().unwrap().get(self.name.as_str()).unwrap().base_stats.get(&Stat::Attack).unwrap();
        if raw_attack < min_stat(&base_attack) || raw_attack > max_stat(&base_attack) {
            return; // value misread
        }

        if raw_attack > *base_attack {
            let evs: u16 = ((raw_attack - *base_attack) * 4).try_into().unwrap_or(0);
            if evs > u16::min(252, self.remaining_evs) {
                let evs = ((raw_attack as f32 / 1.1).ceil() as u16 - *base_attack as u16) * 4;
                self.pos_nature = Some("attack".to_string());

                if let Some(neg_nature) = self.neg_nature.as_ref() {
                    let dec_stat = get_stat_enum(neg_nature);
                    let nature = NATURES.get().unwrap().iter()
                    .find(|n| n.increased_stat == Stat::Attack && n.decreased_stat == dec_stat);
                    if let Some(nature) = nature {
                        self.set_nature(nature.name.as_str());
                    }
                }
            }
            self.attack = evs as u8;
            self.remaining_evs -= evs as u16;
        } else {
            self.attack_iv = (*base_attack - raw_attack).try_into().unwrap_or(0)    ;
        }
    }

    pub fn set_defense(&mut self, raw_defense: u8) {
        self.defense = raw_defense;
    }

    pub fn set_special_attack(&mut self, raw_spatk: u8) {
        self.special_attack = raw_spatk;
    }

    pub fn set_special_defense(&mut self, raw_spdef: u8) {
        self.special_defense = raw_spdef;
    }

    pub fn set_speed(&mut self, raw_speed: u8) {
        self.speed = raw_speed;
    }

    pub fn set_moves(&mut self, moves: Vec<String>) {
        for mv in moves {
            if MOVES.get().unwrap().contains(&mv) {
                self.moves.push(mv);
            }
        }
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
        self.item.is_some() &&
        self.nature.is_some() &&
        self.remaining_evs != 508 &&
        !self.moves.is_empty()
    }

    pub fn to_import_string(&self) -> String {
        let name_segment = 
        match &self.nickname {
            Some(nickname) => format!("{} ({})", nickname, self.name),
            None => format!("{}", self.name),
        };

        let item_segment = 
        match &self.item {
            Some(item) => format!(" @ {}", item),
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
                Some(tera) => format!("Tera Type: {}\n", tera),
                None => String::new(),
            };
        let ev_line = 
            match self.remaining_evs {
                508 => String::new(),
                _ => format!("EVs: {}\n", self.get_ev_spread()),
            };
        let nature_line = 
            match & self.nature {
                Some(nature) => format!("{} Nature\n", nature),
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
            res.push_str(&format!("{} HP /", self.hp));
        }

        if self.attack > 0 {
            res.push_str(&format!("{} Atk /", self.attack));
        }

        if self.defense > 0 {
            res.push_str(&format!("{} Def /", self.defense));
        }

        if self.special_attack > 0 {
            res.push_str(&format!("{} SpA /", self.special_attack));
        }

        if self.special_defense > 0 {
            res.push_str(&format!("{} SpD /", self.special_defense));
        }

        if self.speed > 0 {
            res.push_str(&format!("{} Spe /", self.speed));
        }

        res.pop();
        res
    }

    fn get_iv_spread(&self) -> Option<String> {
        let mut res = String::new();

        if self.hp_iv != 31 {
            res.push_str(&format!("{} HP /", self.hp_iv));
        }

        if self.attack_iv != 31 {
            res.push_str(&format!("{} Atk /", self.attack_iv));
        }

        if self.defense_iv != 31 {
            res.push_str(&format!("{} Def /", self.defense_iv));
        }

        if self.special_attack_iv != 31 {
            res.push_str(&format!("{} SpA /", self.special_attack_iv));
        }

        if self.special_defense_iv != 31 {
            res.push_str(&format!("{} SpD /", self.special_defense_iv));
        }

        if self.speed_iv != 31 {
            res.push_str(&format!("{} Spe /", self.speed_iv));
        }

        if res.is_empty() {
            return None;
        }

        res.pop();
        Some(res)
    }
}

fn min_stat(stat: &u32) -> u32 {
    (*stat as f32 * 0.9).floor() as u32
}

fn min_hp(hp: &u32) -> u32 {
    *hp - 31
}

fn max_stat(stat: &u32) -> u32 {
    (*stat as f32 + (252.0 / 4.0) * 1.1).floor() as u32
}

fn max_hp(hp: &u32) -> u32 {
    *hp + (252 / 4)
}