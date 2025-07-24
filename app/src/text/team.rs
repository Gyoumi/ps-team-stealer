use crate::image::ocr::OllamaOcrResult;
use crate::text::pokemon::Pokemon;

pub struct Team {   
    pub pokemon: Vec<Pokemon>,
}

impl Team {
    pub fn new(pokemon: Vec<Pokemon>) -> Self {
        Self { pokemon }
    }

    pub fn add_pokemon(&mut self, pkmn_name: &str) -> bool {
        if self.pokemon.len() >= 6 {
            return false;
        }

        self.pokemon.push(Pokemon::new(pkmn_name));
        true
    }

    pub fn update_pokemon(&mut self, pkmn_name: &str, json_string: &str) -> bool {
        let pkmn = self.pokemon.iter_mut().find(|pkmn| pkmn.get_name() == pkmn_name);
        if pkmn.is_none() {
            return false;
        }
        let mon = pkmn.unwrap();

        let json_string = json_string.trim().trim_start_matches("```json")
        .trim_start().trim_start_matches("'''json").trim_start().trim_end_matches("'''").
        trim_end().trim_end_matches("```").trim_end();

        println!("converting json string to object");

        let json = serde_json::from_str::<OllamaOcrResult>(json_string);
        if json.is_err() {
            return false;
        }
        let ollama_pkmn = json.unwrap();
        
        match ollama_pkmn.spe_range {
            true => return false,
            false => (),
        }

        mon.set_nickname(&ollama_pkmn.name, ollama_pkmn.nickname.as_deref());
        let tera_res = mon.set_tera(ollama_pkmn.tera_type.as_deref());
        let moves_res = mon.set_moves(ollama_pkmn.moves);
        let ability_res = mon.set_ability(&ollama_pkmn.ability);
        let item_res = mon.set_item(ollama_pkmn.item.as_deref());
        let hp_res = mon.set_hp(ollama_pkmn.remaining_hp, ollama_pkmn.max_hp);

        let mut stat_setters: Vec<(u32, Box<dyn FnMut(&mut Pokemon, u32) -> bool>)> = vec![
            (ollama_pkmn.atk, Box::new(|mon, val| mon.set_attack(val))),
            (ollama_pkmn.def, Box::new(|mon, val| mon.set_defense(val))),
            (ollama_pkmn.spa, Box::new(|mon, val| mon.set_special_attack(val))),
            (ollama_pkmn.spd, Box::new(|mon, val| mon.set_special_defense(val))),
            (ollama_pkmn.spe, Box::new(|mon, val| mon.set_speed(val))),
        ];

        stat_setters.sort_by(|a, b| b.0.cmp(&a.0));

        let mut stat_res = true;
        for (val, setter) in &mut stat_setters {
            let setter_res = setter(mon, *val);
            stat_res = stat_res && setter_res;
        }

        println!("mon ready: {:?}", mon);
        println!("export format: {}", mon.to_import_string());

        tera_res && moves_res && ability_res && item_res && hp_res && stat_res
    }

    pub fn to_import_string(&self) -> String {
        self.pokemon.iter().map(|pkmn| pkmn.to_import_string()).collect::<Vec<String>>().join("\n")
    }

    pub fn print_names(&self) {
        println!("{}", self.pokemon.iter().map(|pkmn| pkmn.get_name()).collect::<Vec<&str>>().join(", "));
    }

    pub fn exists_in_team(&self, pokemon_name: &str) -> bool {
        self.pokemon.iter().any(|pkmn| pkmn.get_name() == pokemon_name)
    }
}

impl PartialEq for Team {
    fn eq(&self, other: &Self) -> bool { // if all names are same in same order, then teams are the same
        for (i, mon) in self.pokemon.iter().enumerate() {
            if mon.get_name() != other.pokemon[i].get_name() || mon.get_nickname() != other.pokemon[i].get_nickname() {
                return false;
            }
        }
        true
    }
}