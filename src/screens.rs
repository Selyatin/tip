use super::types::State;
use std::{
    io::{self, Write, Stdout},
    time::{Instant, Duration}
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

pub fn print_help(stdout: &mut Stdout, state: &State) -> io::Result<()> {
    queue!(
        stdout,
        MoveTo(0, state.rows),
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
            if let Some(d) = player.input.get(j) {
                if c == *d && i == 0 {
                    color = Color::Green;
                    correct_chars += 1;
                    boldness = Attribute::Bold;
                } else if i == 0 {
                    color = Color::Red;
                    boldness = Attribute::Bold;
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
