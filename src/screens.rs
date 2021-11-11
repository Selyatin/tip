use super::types::State;
use std::{
    io::{self, Write, Read, Stdout},
    time::{Instant, Duration},
    net::TcpStream
};
use crossterm::{
    queue,
    ExecutableCommand,
    QueueableCommand,
    terminal::{self, Clear, ClearType},
    cursor::MoveTo,
    style::{Attribute, Stylize, style, SetForegroundColor, Color, Print, PrintStyledContent}
};

pub fn main_screen(stdout: &mut Stdout, state: &State) -> io::Result<()> {
    let x = state.columns / 2 - 10;
    let y = (state.rows / 2) as f32;

    queue!(
        stdout,
        Clear(ClearType::All),
        MoveTo(x, (y * 0.75) as u16), 
        PrintStyledContent("F1 - Single Player".green().bold()),
        MoveTo(x, (y * 0.8) as u16),
        PrintStyledContent("F2 - Create New Multiplayer Session".yellow().bold()),
        MoveTo(x, (y * 0.85) as u16),
        PrintStyledContent("F3 - Join Session".blue().bold()),
        MoveTo(x, (y * 0.9) as u16),
        PrintStyledContent("ESC - Quit".red().bold())
    )?; 

    Ok(())
}

pub fn single_player_screen(stdout: &mut Stdout, state: &mut State) -> io::Result<()> {
    queue!(stdout, Clear(ClearType::All))?;

    print_help(stdout, &state)?;

    let (columns, rows) = (state.columns, state.rows);

    let player = &mut state.players[state.current_player];

    let elapsed_millis = state.instant.elapsed().as_millis();

    let should_go_forward: bool = elapsed_millis - state.last_instant > 500;

    let mut add_x: u16 = 4;

    for (i, word) in state.dictionary[player.position..player.position + 4].iter_mut().enumerate() {

        let mut correct_chars = 0;

        for (j, c) in word.value.chars().enumerate() {
            let mut color = Color::White;
            let mut boldness = Attribute::NormalIntensity; 

            if i == 0 {
                if let Some(d) = player.input.chars().nth(j) {
                    if d == c {
                        color = Color::Green;
                        correct_chars += 1;
                        boldness = Attribute::Bold;
                    } else {
                        color = Color::Red;
                        boldness = Attribute::Bold;
                    }
                }
            }

            queue!(
                stdout, 
                MoveTo(word.x + j as u16, word.y),
                PrintStyledContent(style(c).with(color).attribute(boldness))
            )?;


        }

        if correct_chars == word.value.len() || word.x >= columns{
            player.input.clear();
            player.position += 1;
        }

        if should_go_forward {
            word.x += add_x;
            state.last_instant = elapsed_millis;
        }

        add_x -= 1;

        queue!(stdout, MoveTo(columns, rows))?;

    }

    Ok(())
}

pub fn join_screen(stdout: &mut Stdout, state: &mut State) -> io::Result<()> {
    let (columns, rows) = (state.columns as f32, state.rows as f32); 

    queue!(stdout, Clear(ClearType::All))?;

    print_help(stdout, &state);

    let (x_start, x_end) = ((columns * 0.4) as u16, (columns * 0.6) as u16);

    let (y_start, y_end) = ((rows * 0.4) as u16, (rows * 0.45) as u16);

    queue!(
        stdout,
        MoveTo(x_start, y_start - 1),
        PrintStyledContent("Enter Session Code".bold()),
    )?;

    for x in x_start..x_end {
        queue!(
            stdout, 
            MoveTo(x, y_start), 
            Print('-'), 
            MoveTo(x, y_end), 
            Print('-')
        )?;
    }

    let x_end = (columns * 0.59) as u16;

    let (y_start, y_end) = ((rows * 0.42) as u16, (rows * 0.45) as u16);

    for y in y_start..y_end {
        queue!(
            stdout, 
            MoveTo(x_start, y), 
            Print('|'), 
            MoveTo(x_end, y), 
            Print('|')
        )?;
    }

    Ok(()) 
}

pub fn print_error(
    stdout: &mut Stdout, 
    state: &State, 
    err: impl std::fmt::Display
) -> io::Result<()> {
    let (columns, rows) = (state.columns, state.rows);

    queue!(
        stdout,
        MoveTo(columns / 2, rows),
        PrintStyledContent(style(err).red().bold())
    )?;

    Ok(())
}

pub fn print_help(stdout: &mut Stdout, state: &State) -> io::Result<()> {
    queue!(
        stdout,
        MoveTo(0, state.rows),
        PrintStyledContent("ESC - Go Back".yellow().bold())
    )?;

    Ok(())
}
