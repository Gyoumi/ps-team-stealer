use once_cell::sync::OnceCell;
use std::collections::{HashMap, HashSet};
use rustemon::{client::RustemonClient, pokemon, moves, items};
use futures::future;
use itertools::Itertools;
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

pub static NATURES: OnceCell<Vec<Nature>> = OnceCell::new();
pub static ABILITIES: OnceCell<HashSet<String>> = OnceCell::new();
pub static ITEMS: OnceCell<HashSet<String>> = OnceCell::new();
pub static MOVES: OnceCell<HashSet<String>> = OnceCell::new();
pub static TYPES: OnceCell<HashSet<String>> = OnceCell::new();
pub static POKEMON_SPECIES: OnceCell<HashMap<String, PokemonBaseStat>> = OnceCell::new();

pub static RUSTEMON_CLIENT: OnceCell<RustemonClient> = OnceCell::new();

#[derive(PartialEq, Eq, Hash, Debug, serde::Serialize, serde::Deserialize, Clone)]
pub enum Stat {
    HP,
    Attack,
    Defense,
    SpecialAttack,
    SpecialDefense,
    Speed,
    None,
}

#[derive(PartialEq, Eq, Hash, Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct Nature {
    pub name: String,
    pub decreased_stat: Stat,
    pub increased_stat: Stat
}

#[derive(PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct PokemonBaseStat {
    pub base_stats: HashMap<Stat, u32>
}

pub async fn init_pokemon_data() {
    let pokemon_species = get_all_pokemon_species();
    let natures = get_all_natures();
    let abilities = get_all_abilities();
    let items = get_all_items();
    let moves = get_all_moves();
    let types = get_all_types();

    tokio::join!(pokemon_species, natures, abilities, items, moves, types);
}

async fn get_all_pokemon_species() {
    let data_path = "src/text/data/pokemon_species.json";
    let species: Option<HashMap<String, PokemonBaseStat>> = read_data(data_path);
    if let Some(species) = species {
        POKEMON_SPECIES.set(species).unwrap();
        println!("Loaded pokemon species from file");
    } else {
        let species = fetch_all_pokemon_species().await;
        let _ = write_data(data_path, &species);
        POKEMON_SPECIES.set(species).unwrap();
        println!("Loaded pokemon species from API");
    }
}

async fn fetch_all_pokemon_species() -> HashMap<String, PokemonBaseStat> {
    let client = RUSTEMON_CLIENT.get_or_init(|| RustemonClient::default());
    let result = pokemon::pokemon::get_all_entries(&client).await;
    if let Ok(species) = result {
        let species_futures = species.into_iter().map(|species| {
            let name = species.name;
            async move {
                let mut formes = Vec::new();

                let res = pokemon::pokemon::get_by_name(&name, &client).await;
                if let Ok(mon) = res {
                    let base_stats = mon.stats.into_iter().map(|stat| {
                        let stat_name = get_stat_enum(&stat.stat.name);
                        let stat_value = match stat_name {
                            Stat::HP => match stat.base_stat {
                                1 => 1, // shedinja case
                                _ => stat.base_stat*2+141
                            }
                            _ => stat.base_stat*2+36
                        };
                        (stat_name, stat_value as u32)
                    }).collect::<HashMap<Stat, u32>>();
                    formes.push((name, PokemonBaseStat {
                        base_stats: base_stats.clone()
                    }));

                    let alts = mon.forms.into_iter().for_each(|form| formes.push((form.name, PokemonBaseStat { base_stats: base_stats.clone() })));
                } else {
                    formes.push((name, PokemonBaseStat {
                        base_stats: HashMap::new(),
                    }));
                }

                formes
            }
        });
        future::join_all(species_futures).await.into_iter().flatten().collect::<HashMap<String, PokemonBaseStat>>()
    } else {
        eprintln!("Failed to get all pokemon names");
        HashMap::new()
    }
}

async fn get_all_natures() {
    let data_path = "src/text/data/natures.json";
    let natures: Option<Vec<Nature>> = read_data(data_path);
    if let Some(natures) = natures {
        NATURES.set(natures).unwrap();
        println!("Loaded natures from file");
    } else {
        let natures = fetch_all_natures().await;
        let _ = write_data(data_path, &natures);
        NATURES.set(natures).unwrap();
        println!("Loaded natures from API");
    }
}

async fn fetch_all_natures() -> Vec<Nature> {
    let client = RUSTEMON_CLIENT.get_or_init(|| RustemonClient::default());
    let result = pokemon::nature::get_all_entries(&client).await;
    if let Ok(natures_res) = result {
        let natures_futures = natures_res.into_iter().map(|nature| {
            let name = nature.name.clone();
            async move {
                let nature_res = match pokemon::nature::get_by_name(&name, &client).await {
                    Ok(nature) => nature,
                    Err(_) => {
                        return Nature {
                            name,
                            decreased_stat: Stat::Attack,
                            increased_stat: Stat::Attack,
                        }
                    }
                };
                let (dec_field, inc_field) = match (nature_res.decreased_stat, nature_res.increased_stat) {
                    (Some(dec), Some(inc)) => (dec, inc),
                    _ => {
                        return Nature {
                            name,
                            decreased_stat: Stat::Attack,
                            increased_stat: Stat::Attack,
                        }
                    }
                };
                let decreased_stat = match pokemon::stat::get_by_name(&dec_field.name, &client).await {
                    Ok(stat) => match stat.name.as_str() {
                        "attack" => Stat::Attack,
                        "defense" => Stat::Defense,
                        "special-attack" => Stat::SpecialAttack,
                        "special-defense" => Stat::SpecialDefense,
                        "speed" => Stat::Speed,
                        _ => Stat::Attack,
                    },
                    Err(_) => Stat::Attack,
                };
                let increased_stat = match pokemon::stat::get_by_name(&inc_field.name, &client).await {
                    Ok(stat) => match stat.name.as_str() {
                        "attack" => Stat::Attack,
                        "defense" => Stat::Defense,
                        "special-attack" => Stat::SpecialAttack,
                        "special-defense" => Stat::SpecialDefense,
                        "speed" => Stat::Speed,
                        _ => Stat::Attack,
                    },
                    Err(_) => Stat::Attack,
                };
                Nature {
                    name,
                    decreased_stat,
                    increased_stat,
                }
            }
        });
        future::join_all(natures_futures).await
    } else {
        eprintln!("Failed to get all natures");
        Vec::new()
    }
}

async fn get_all_abilities() {
    let data_path = "src/text/data/abilities.json";
    let abilities: Option<HashSet<String>> = read_data(data_path);
    if let Some(abilities) = abilities {
        ABILITIES.set(abilities).unwrap();
        println!("Loaded abilities from file");
    } else {
        let abilities = fetch_all_abilities().await;
        let _ = write_data(data_path, &abilities);
        ABILITIES.set(abilities).unwrap();
        println!("Loaded abilities from API");
    }
}

async fn fetch_all_abilities() -> HashSet<String> {
    let client = RUSTEMON_CLIENT.get_or_init(|| RustemonClient::default());
    let result = pokemon::ability::get_all_entries(&client).await;
    if let Ok(abilities) = result {
        abilities.into_iter().map(|ability| ability.name.to_string().split("-").join(" ")).collect::<HashSet<String>>()
    } else {
        eprintln!("Failed to get all abilities");
        HashSet::new()
    }
}

async fn get_all_items() {
    let data_path = "src/text/data/items.json";
    let items: Option<HashSet<String>> = read_data(data_path);
    if let Some(items) = items {
        ITEMS.set(items).unwrap();
        println!("Loaded items from file");
    } else {
        let items = fetch_all_items().await;
        let _ = write_data(data_path, &items);
        ITEMS.set(items).unwrap();
        println!("Loaded items from API");
    }
}

async fn fetch_all_items() -> HashSet<String> {
    let client = RUSTEMON_CLIENT.get_or_init(|| RustemonClient::default());
    let result = items::item::get_all_entries(&client).await;
    if let Ok(items) = result {
        items.into_iter().map(|item| item.name.to_string().split("-").join(" ")).collect::<HashSet<String>>()
    } else {
        eprintln!("Failed to get all items");
        HashSet::new()
    }
}

async fn get_all_moves() {
    let data_path = "src/text/data/moves.json";
    let moves: Option<HashSet<String>> = read_data(data_path);
    if let Some(moves) = moves {
        MOVES.set(moves).unwrap();
        println!("Loaded moves from file");
    } else {
        let moves = fetch_all_moves().await;
        let _ = write_data(data_path, &moves);
        MOVES.set(moves).unwrap();
        println!("Loaded moves from API");
    }
}

async fn fetch_all_moves() -> HashSet<String> {
    let client = RUSTEMON_CLIENT.get_or_init(|| RustemonClient::default());
    let result = moves::move_::get_all_entries(&client).await;
    if let Ok(moves) = result {
        moves.into_iter().map(|mv| mv.name.to_string().split("-").join(" ")).collect::<HashSet<String>>()
    } else {
        eprintln!("Failed to get all moves");
        HashSet::new()
    }
}

async fn get_all_types() {
    let data_path = "src/text/data/types.json";
    let types: Option<HashSet<String>> = read_data(data_path);
    if let Some(types) = types {
        TYPES.set(types).unwrap();
        println!("Loaded types from file");
        } else {
        let types = fetch_all_types().await;
        let _ = write_data(data_path, &types);
        TYPES.set(types).unwrap();
        println!("Loaded types from API");
    }
}

async fn fetch_all_types() -> HashSet<String> {
    let client = RUSTEMON_CLIENT.get_or_init(|| RustemonClient::default());
    let result = pokemon::type_::get_all_entries(&client).await;
    if let Ok(types) = result {
        types.into_iter().map(|tp| tp.name).collect::<HashSet<String>>()
    } else {
        eprintln!("Failed to get all types");
        HashSet::new()
    }
}

fn read_data<T: serde::de::DeserializeOwned>(path: &str) -> Option<T> {
    if let Ok(mut file) = fs::File::open(path) {
        let mut contents = String::new();
        if file.read_to_string(&mut contents).is_ok() && !contents.trim().is_empty() {
            serde_json::from_str(&contents).ok()
        } else {
            None
        }
    } else {
        None
    }
}

fn write_data<T: serde::Serialize>(path: &str, data: &T) -> io::Result<()> {
    let serialized = serde_json::to_string(data).unwrap();
    let mut file = fs::File::create(path)?;
    file.write_all(serialized.as_bytes())
}

pub fn get_stat_enum(name: &str) -> Stat {
    match name {
        "hp" => Stat::HP,
        "attack" => Stat::Attack,
        "defense" => Stat::Defense,
        "special-attack" => Stat::SpecialAttack,
        "special-defense" => Stat::SpecialDefense,
        "speed" => Stat::Speed,
        _ => Stat::None,
    }
}