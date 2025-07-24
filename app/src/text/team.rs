use crate::text::pokemon::Pokemon;

pub struct Team {   
    pub pokemon: Vec<Pokemon>,
}

impl Team {
    pub fn new(pokemon: Vec<Pokemon>) -> Self {
        Self { pokemon }
    }

    pub fn to_import_string(&self) -> String {
        self.pokemon.iter().map(|pkmn| pkmn.to_import_string()).collect::<Vec<String>>().join("\n")
    }

    pub fn print_names(&self) {
        println!("{}", self.pokemon.iter().map(|pkmn| pkmn.get_name()).collect::<Vec<&str>>().join(", "));
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