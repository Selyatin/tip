use std::time::Instant;

pub struct State {
    pub columns: u16,
    pub rows: u16,
    pub players: Vec<Player>,
    pub dictionary: Vec<Word>,
    pub instant: Instant,
    pub last_instant: u128,
    pub current_player: usize
}

pub enum Screen {
    Main,
    Single
}

pub struct Word {
    pub value: String,
    pub x: u16,
    pub y: u16
}

impl Word {
    pub fn new(value: impl Into<String>, x: u16, y: u16) -> Self {
        Self {
            value: value.into(),
            x,
            y
        }
    }
}

#[derive(Default)]
pub struct Player {
    pub position: usize,
    pub input: Vec<char>
}
