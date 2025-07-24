use once_cell::sync::OnceCell;
use std::collections::{HashMap, HashSet};
use rustemon::{client::RustemonClient, pokemon, moves, items};
use futures::future;
use itertools::Itertools;

pub static NATURES: OnceCell<Vec<Nature>> = OnceCell::new();
pub static ABILITIES: OnceCell<HashSet<String>> = OnceCell::new();
pub static ITEMS: OnceCell<HashSet<String>> = OnceCell::new();
pub static MOVES: OnceCell<HashSet<String>> = OnceCell::new();
pub static TYPES: OnceCell<HashSet<String>> = OnceCell::new();
pub static POKEMON_SPECIES: OnceCell<HashMap<String, PokemonBaseStat>> = OnceCell::new();

pub static RUSTEMON_CLIENT: OnceCell<RustemonClient> = OnceCell::new();

#[derive(PartialEq, Eq, Hash, Debug)]
pub enum Stat {
    HP,
    Attack,
    Defense,
    SpecialAttack,
    SpecialDefense,
    Speed,
    None,
}

#[derive(PartialEq, Eq, Hash, Debug)]
pub struct Nature {
    pub name: String,
    pub decreased_stat: Stat,
    pub increased_stat: Stat
}

#[derive(PartialEq, Eq, Debug)]
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
    let client = RUSTEMON_CLIENT.get_or_init(|| RustemonClient::default());
    let result = pokemon::pokemon::get_all_entries(&client).await;
    
    if let Ok(species) = result {
        let species_futures = species.into_iter().map(|species| {
            let name = species.name;
            async move {
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
                (name, PokemonBaseStat {
                    base_stats
                })
            } else {
                (name, PokemonBaseStat {
                    base_stats: HashMap::new(), // TODO: set better default value (maybe from a file)
                })
            }
        }
        });
        let species = future::join_all(species_futures).await.into_iter().collect::<HashMap<String, PokemonBaseStat>>();
        println!("Got {} pokemon species", species.len());
        POKEMON_SPECIES.set(species).unwrap();
    } else {
        eprintln!("Failed to get all pokemon names");
    }
}

async fn get_all_natures() {
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

        let natures = future::join_all(natures_futures).await;
        println!("Got {} natures", natures.len());
        NATURES.set(natures).unwrap();
    } else {
        eprintln!("Failed to get all natures");
    }
}

async fn get_all_abilities() {
    let client = RUSTEMON_CLIENT.get_or_init(|| RustemonClient::default());
    let result = pokemon::ability::get_all_entries(&client).await;

    if let Ok(abilities) = result {
        let abilities = abilities.into_iter().map(|ability| ability.name).collect::<HashSet<String>>();
        println!("Got {} abilities", abilities.len());
        ABILITIES.set(abilities).unwrap();
    } else {
        eprintln!("Failed to get all abilities");
    }
}

async fn get_all_items() {
    let client = RUSTEMON_CLIENT.get_or_init(|| RustemonClient::default());
    let result = items::item::get_all_entries(&client).await;

    if let Ok(items) = result {
        let items = items.into_iter().map(|item| item.name.to_string().split("-").join(" ")).collect::<HashSet<String>>();
        println!("Got {} items", items.len());
        ITEMS.set(items).unwrap();
    } else {
        eprintln!("Failed to get all items");
    }
} 

async fn get_all_moves() {
    let client = RUSTEMON_CLIENT.get_or_init(|| RustemonClient::default());
    let result = moves::move_::get_all_entries(&client).await;

    if let Ok(moves) = result {
        let moves = moves.into_iter().map(|mv| mv.name.to_string().split("-").join(" ")).collect::<HashSet<String>>();
        println!("Got {} moves", moves.len());
        MOVES.set(moves).unwrap();
    } else {        
        eprintln!("Failed to get all moves");
    }
}

async fn get_all_types() {
    let client = RUSTEMON_CLIENT.get_or_init(|| RustemonClient::default());
    let result = pokemon::type_::get_all_entries(&client).await;

    if let Ok(types) = result {
        let types = types.into_iter().map(|tp| tp.name).collect::<HashSet<String>>();
        println!("Got {} types", types.len());
        TYPES.set(types).unwrap();
    } else {
        eprintln!("Failed to get all types");
    }
}

pub fn get_stat_enum(name: &str) -> Stat {
    match name {
        "attack" => Stat::Attack,
        "defense" => Stat::Defense,
        "special-attack" => Stat::SpecialAttack,
        "special-defense" => Stat::SpecialDefense,
        "speed" => Stat::Speed,
        _ => Stat::None,
    }
}