use super::socket::Socket;
use std::{net::TcpStream, time::Instant};

pub struct State {
    pub columns: u16,
    pub rows: u16,
    pub sock_addr: String,
    pub screen: Screen,
    pub players: Vec<Player>,
    pub dictionary: Vec<Word>,
    pub instant: Instant,
    pub last_instant: u128,
    pub current_player: usize,
    pub session_token: Option<u16>,
    pub socket: Option<Socket>,
    pub err: Option<anyhow::Error>,
}

/// Used in Multiplayer to determine what kind of data is received
#[derive(Copy, Clone)]
pub enum Action {
    Input((usize, char)),
    Join(u8),
    Left(usize),
}

#[derive(Eq, PartialEq)]
pub enum Screen {
    Main,
    SinglePlayer,
    MultiPlayer,
    Join,
}

pub struct Word {
    pub value: String,
    pub x: u16,
    pub y: u16,
}

impl Word {
    pub fn new(value: impl Into<String>, x: u16, y: u16) -> Self {
        Self {
            value: value.into(),
            x,
            y,
        }
    }
}

#[derive(Default)]
pub struct Player {
    // Used for ordering the players positions on screen in a multiplayer session
    pub sort_position: u8,
    // Used as an index for Player's Word position in the dictionary
    pub position: usize,
    pub input: String,
    pub current_player: bool,
}
