#[macro_use]
extern crate lazy_static;

mod screens;
mod socket;
mod types;

use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute, queue,
    style::{style, Print, PrintStyledContent, Stylize},
    terminal::{self, Clear, ClearType},
};
use socket::Socket;
use std::{
    env,
    io::{self, stdout, Stdout, Write},
    time::{Duration, Instant},
};
use types::{Player, Screen, State, Word};

lazy_static! {
    static ref DICTIONARY: Vec<Word> = include_str!("../dictionary.txt")
        .split("\n")
        .map(|s| Word::new(s, 0, 0))
        .collect();
}

fn reset_state(state: &mut State) {
    state.socket = None;
    state.err = None;
    state.session_token = None;
    state.current_player = 0;
    state.players.clear();
    let mut player = Player::default();
    player.current_player = true;
    state.players.push(player);
}

fn main_loop(stdout: &mut Stdout, state: &mut State) -> io::Result<()> {
    // Clear the previous frame
    queue!(stdout, Clear(ClearType::All))?;

    match state.screen {
        Screen::Main => screens::main(stdout, state)?,
        Screen::SinglePlayer => screens::single_player(stdout, state)?,
        Screen::Join => screens::join(stdout, state)?,
        Screen::MultiPlayer => screens::multi_player(stdout, state)?,
        Screen::Loading => screens::loading(stdout, state)?,
    };

    if let Some(err) = &state.err {
        queue!(
            stdout,
            MoveTo((state.columns as f32 * 0.4) as u16, state.rows),
            PrintStyledContent("Error: ".red().bold()),
            PrintStyledContent(style(err).red().bold())
        )?;
    }

    if let Some(session_token) = &state.session_token {
        queue!(
            stdout,
            MoveTo((state.columns as f32 * 0.1) as u16, state.rows),
            PrintStyledContent("Session Token: ".green().bold()),
            PrintStyledContent(style(session_token).green().bold())
        )?;
    }

    // Render the queued frame
    stdout.flush()?;

    if event::poll(Duration::from_millis(16))? {
        match event::read()? {
            Event::Key(KeyEvent {
                code: KeyCode::Char(c),
                modifiers: KeyModifiers::NONE,
            }) => {
                if let Some(player) = state.players.get_mut(state.current_player) {
                    player.input.push(c);
                    if let Some(socket) = &mut state.socket {
                        socket.send_input(c)?;
                    }
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                modifiers: KeyModifiers::NONE,
            }) => match state.screen {
                Screen::Join => {
                    let player = state.players.get_mut(state.current_player).unwrap();

                    state.socket = Some(Socket::new(&state.sock_addr)?);

                    let session_token: u16 = player
                        .input
                        .parse()
                        .map_err(|_| io::Error::new(io::ErrorKind::Other, "Invalid Number."))?;

                    player.sort_position =
                        state.socket.as_mut().unwrap().join_session(session_token)?;

                    state.session_token = Some(session_token);

                    state.socket.as_ref().unwrap().init_reader()?;

                    state.dictionary = DICTIONARY.clone();

                    let rng = fastrand::Rng::with_seed(state.session_token.unwrap().into());

                    rng.shuffle(&mut state.dictionary);

                    let mut prev_y: u16 = 0;

                    for word in &mut state.dictionary {
                        let mut y = rng.u16(0..state.rows - 1);
                        if y == prev_y {
                            y += 2;
                            if y > state.rows - 1 {
                                y -= 3;
                            }
                        }
                        prev_y = y;
                        word.y = y;
                    }

                    player.input.clear();

                    state.screen = Screen::Loading;
                }
                _ => (),
            },

            Event::Key(KeyEvent {
                code: KeyCode::Backspace,
                modifiers: KeyModifiers::NONE,
            }) => {
                if let Some(player) = state.players.get_mut(state.current_player) {
                    player.input.pop();
                    if let Some(socket) = &mut state.socket {
                        socket.send_input('-')?;
                    }
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Esc,
                modifiers: KeyModifiers::NONE,
            }) => {
                if state.screen == Screen::Main {
                    execute!(stdout, Clear(ClearType::All), Show)?;
                    terminal::disable_raw_mode()?;
                    std::process::exit(0);
                }
                reset_state(state);
                state.screen = Screen::Main;
            }
            Event::Key(KeyEvent {
                code: KeyCode::F(1),
                modifiers: KeyModifiers::NONE,
            }) => {
                reset_state(state);

                let rng = fastrand::Rng::new();

                rng.shuffle(&mut state.dictionary);

                let mut prev_y: u16 = 0;

                for word in &mut state.dictionary {
                    let mut y = rng.u16(0..state.rows - 1);
                    if y == prev_y {
                        y += 2;
                        if y > state.rows - 1 {
                            y -= 3;
                        }
                    }
                    prev_y = y;
                    word.y = y;
                }

                state.screen = Screen::SinglePlayer;
            }
            Event::Key(KeyEvent {
                code: KeyCode::F(2),
                modifiers: KeyModifiers::NONE,
            }) => {
                reset_state(state);

                state.socket = Some(Socket::new(&state.sock_addr)?);

                state.session_token = Some(state.socket.as_mut().unwrap().create_session()?);

                state.socket.as_ref().unwrap().init_reader()?;

                state.dictionary = DICTIONARY.clone();

                let rng = fastrand::Rng::with_seed(state.session_token.unwrap().into());

                rng.shuffle(&mut state.dictionary);

                let mut prev_y: u16 = 0;

                for word in &mut state.dictionary {
                    let mut y = rng.u16(0..state.rows - 1);
                    if y == prev_y {
                        y += 2;
                        if y > state.rows - 1 {
                            y -= 3;
                        }
                    }
                    prev_y = y;
                    word.y = y;
                }

                state.screen = Screen::Loading;
            }
            Event::Key(KeyEvent {
                code: KeyCode::F(3),
                modifiers: KeyModifiers::NONE,
            }) => {
                reset_state(state);
                state.screen = Screen::Join;
            }
            Event::Resize(new_columns, new_rows) => {
                // Using nearest-neighbor interpolation to scale the frame up/down
                let scale_x = new_columns as f32 / state.columns as f32;
                let scale_y = new_rows as f32 / state.rows as f32;
                for word in &mut state.dictionary {
                    word.x = (word.x as f32 * scale_x) as u16;
                    word.y = (word.y as f32 * scale_y) as u16;
                }
                state.columns = new_columns;
                state.rows = new_rows;
            }
            _ => (),
        };
    }

    Ok(())
}

fn main() -> io::Result<()> {
    let sock_addr = env::args().nth(1).unwrap_or("127.0.0.1:8080".to_owned());

    terminal::enable_raw_mode()?;

    // Get initial terminal size
    let (columns, rows) = terminal::size()?;

    let mut state = State {
        columns,
        rows,
        dictionary: DICTIONARY.clone(),
        sock_addr,
        screen: Screen::Main,
        players: vec![],
        instant: Instant::now(),
        last_instant: 0,
        current_player: 0,
        session_token: None,
        socket: None,
        err: None,
    };

    let mut stdout = stdout();

    queue!(
        stdout,
        Hide,
        Clear(ClearType::All),
        MoveTo(columns / 2, rows / 2),
        Print("Shuffling Dictionary...")
    )?;

    stdout.flush()?;

    loop {
        if let Err(err) = main_loop(&mut stdout, &mut state) {
            state.err = Some(Box::new(err));
        }
    }
}
