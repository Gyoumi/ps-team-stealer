pub enum Weather {
    Sand,
    Sun,
    Rain,
    Snow,
    Hail,
    Tailwind,
}

pub type WeatherRange = (u32, u32);

pub struct Battle {
    opponent: String,
    highest_turn: u32,
    weather_turns: HashMap<Weather, Vec<WeatherRange>>
}

impl Battle {
    pub fn new() -> Self {
        Self {
            opponent: "".to_string(),
            highest_turn: -1,
            weather_turns: HashMap::new(),
        }
    }

    pub fn set_opponent(&mut self, opponent: String) {
        self.opponent = opponent;
    }

    pub fn update_highest_turn(&mut self, turn: u32) {
        self.highest_turn = self.highest_turn.max(turn);
    }

    pub fn add_weather_start(&mut self, weather: Weather, turn: u32) {
        self.weather_turns.entry(weather).or_insert(Vec::new()).push((turn, turn));
    }

    pub fn update_weather_end(&mut self, weather: Weather, turn: u32) {
        if let Some(ranges) = self.weather_turns.get_mut(&weather) {
            if let Some(last_range) = ranges.last_mut() {
                last_range.1 = turn;
            }
        }
    }

    pub fn get_weather_turns(&self, weather: Weather) -> Option<&Vec<WeatherRange>> {
        self.weather_turns.get(&weather)
    }

    pub fn get_highest_turn(&self) -> u32 {
        self.highest_turn
    }
}