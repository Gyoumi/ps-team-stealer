use once_cell::sync::Lazy;
use std::sync::{Mutex, Arc};
use tokio::sync::RwLock;
use std::collections::HashMap;
use crate::text::{team::Team, battle::Battle, pokeapi::{ABILITIES, ITEMS, MOVES, NATURES, TYPES, POKEMON_SPECIES}};
use rust_fuzzy_search::{fuzzy_search_best_n};

pub static TEAMS: Lazy<Arc<RwLock<Vec<Team>>>> = Lazy::new(|| Arc::new(RwLock::new(Vec::new())));
pub static BATTLES: Lazy<Arc<RwLock<Vec<Battle>>>> = Lazy::new(|| Arc::new(RwLock::new(Vec::new())));

static OCR_RESULTS: Lazy<Mutex<HashMap<String, Vec<String>>>> = Lazy::new(|| Mutex::new(HashMap::new()));

pub fn process_text(image_id: String, pokemon_name: String) {
    let mut results = OCR_RESULTS.lock().unwrap();
    results.entry(image_id).or_insert_with(Vec::new).push(pokemon_name);
}

fn validate_name<'a, I, F>(
    input: &str,
    all_names: I,
    exact_check: F,
) -> Option<String>
where
    I: IntoIterator<Item = &'a str>,
    F: Fn(&str) -> bool,
{
    let iter = input.trim().to_lowercase().split_whitespace().collect::<Vec<&str>>().join("-");

    if exact_check(&iter) {
        return Some(capitalise_first(&iter));
    }

    let names_vec: Vec<&str> = all_names.into_iter().collect();
    let best_match = fuzzy_search_best_n(&iter, &names_vec, 1);

    match best_match.first() {
        Some((name, score)) if *score >= 0.6 => Some(capitalise_first(name)),
        _ => None,
    }
}

fn capitalise_first(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut caps_next = true;
    for c in s.chars() {
        if caps_next && c.is_ascii_alphabetic() {
            for up in c.to_uppercase() {
                result.push(up);
            }
            caps_next = false;
        } else {
            result.push(c);
            caps_next = c == '-';
        }
    }
    result
}

pub fn validate_pokemon(pokemon_name: &str) -> Option<String> {
    let species = POKEMON_SPECIES.get()?;
    validate_name(
        pokemon_name,
        species.keys().map(|s| s.as_str()),
        |name| species.contains_key(name),
    )
}

pub fn validate_move(move_name: &str) -> Option<String> {
    let moves = MOVES.get()?;
    validate_name(
        move_name,
        moves.iter().map(|s| s.as_str()),
        |name| moves.contains(name),
    )
}

pub fn validate_ability(ability_name: &str) -> Option<String> {
    let abilities = ABILITIES.get()?;
    validate_name(
        ability_name,
        abilities.iter().map(|s| s.as_str()),
        |name| abilities.contains(name),
    )
}

pub fn validate_items(item_name: &str) -> Option<String> {
    let items = ITEMS.get()?;
    validate_name(
        item_name,
        items.iter().map(|s| s.as_str()),
        |name| items.contains(name),
    )
}

pub fn validate_types(type_name: &str) -> Option<String> {
    let types = TYPES.get()?;
    validate_name(
        type_name,
        types.iter().map(|s| s.as_str()),
        |name| types.contains(name),
    )
}

pub fn validate_nature(nature_name: &str) -> Option<String> {
    let natures = NATURES.get()?;
    validate_name(
        nature_name,
        natures.iter().map(|n| n.name.as_str()),
        |name| natures.iter().any(|n| n.name == name),
    )
}
