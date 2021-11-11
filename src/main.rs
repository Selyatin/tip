mod types;
mod screens;

use types::{State, Word, Screen, Player};
use std::{
    io::{Write, stdout},
    thread,
    time::{Instant, Duration},
};
use crossterm::{
    queue,
    ExecutableCommand,
    QueueableCommand,
    terminal::{self, Clear, ClearType},
    cursor::MoveTo,
    event::{self, Event, KeyEvent, KeyCode, KeyModifiers, },
    style::{Stylize, style, SetForegroundColor, Color, Print, PrintStyledContent}
};
use rand::{
    Rng,
    thread_rng,
    seq::SliceRandom
};

fn reset_state(state: &mut State){
      state.socket = None;
        state.current_player = 0;
        state.players.clear();
        state.players.push(Player::default());
}

fn main() -> anyhow::Result<()> {

    terminal::enable_raw_mode()?;

    // Get initial terminal size
    let (columns, rows) = terminal::size()?;

    let mut rng = thread_rng();

    // Include the dictionary file as a string
    let dictionary_string = include_str!("../dictionary.txt");

    // Parse dictionary
    let dictionary: Vec<Word> = dictionary_string
        .split("\n")
        .map(|s| Word::new(s, 0, rng.gen_range(0..rows - 1)))
        .collect();

    drop(dictionary_string);

    let mut screen = Screen::Main;

    let mut state = State {
        columns,
        rows,
        dictionary,
        players: vec![],
        instant: Instant::now(),
        last_instant: 0,
        current_player: 0,
        socket: None
    };

    let mut stdout = stdout();

    queue!(stdout, Clear(ClearType::All), MoveTo(columns / 2, rows / 2), Print("Shuffling Dictionary..."))?;

    stdout.flush()?;

    state.dictionary.shuffle(&mut rng);

    loop {
        match screen {
            Screen::Main => screens::main_screen(&mut stdout, &state)?,
            Screen::Single => screens::single_player_screen(&mut stdout, &mut state)?,
            Screen::Join => screens::join_screen(&mut stdout, &mut state)?
        };

        // Render the queued frame
        stdout.flush()?;

        // Sleep at most 16 ms so that we render 60 fps
        if event::poll(Duration::from_millis(16))? {
            match event::read()? {
                Event::Key(KeyEvent{code: KeyCode::Char(c), modifiers: KeyModifiers::NONE}) => {
                    if let Some(player) = state.players.get_mut(state.current_player) {
                        player.input.push(c);
                    }
                },
                Event::Key(KeyEvent{code: KeyCode::Enter, modifiers: KeyModifiers::NONE}) => {
                    if let Some(player) = state.players.get_mut(state.current_player) {
                        player.input.push('\n');
                    }
                },
                Event::Key(KeyEvent{code: KeyCode::Backspace, modifiers: KeyModifiers::NONE}) => {
                    if let Some(player) = state.players.get_mut(state.current_player) {
                        player.input.pop();
                    }
                },
                Event::Key(KeyEvent{code: KeyCode::Esc, modifiers: KeyModifiers::NONE}) => {
                    if screen == Screen::Main {
                        break;
                    }
                    screen = Screen::Main;
                },
                Event::Key(KeyEvent{code: KeyCode::F(1), modifiers: KeyModifiers::NONE}) => {
                    reset_state(&mut state);
                    screen = Screen::Single;
                },
                Event::Key(KeyEvent{code: KeyCode::F(3), modifiers: KeyModifiers::NONE}) => {
                    reset_state(&mut state);
                    screen = Screen::Join;
                },
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
                },
                _ => ()        
            };
        }
    }

    terminal::disable_raw_mode()?;

    Ok(())
}
