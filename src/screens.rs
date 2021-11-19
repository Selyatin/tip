use super::types::{Action, Player, State};
use crossterm::{
    cursor::MoveTo,
    queue,
    style::{style, Attribute, Color, Print, PrintStyledContent, SetForegroundColor, Stylize},
    terminal::{self, Clear, ClearType},
    ExecutableCommand, QueueableCommand,
};
use std::{
    io::{self, Read, Stdout, Write},
    iter,
    net::TcpStream,
    time::{Duration, Instant},
};

pub fn main(stdout: &mut Stdout, state: &State) -> io::Result<()> {
    let (x, y) = ((state.columns as f32 * 0.4) as u16, state.rows as f32);

    queue!(
        stdout,
        Clear(ClearType::All),
        MoveTo(x, (y * 0.34) as u16),
        PrintStyledContent("F1 - Single Player".green().bold()),
        MoveTo(x, (y * 0.37) as u16),
        PrintStyledContent("F2 - Create New Multiplayer Session".yellow().bold()),
        MoveTo(x, (y * 0.4) as u16),
        PrintStyledContent("F3 - Join Session".blue().bold()),
        MoveTo(x, (y * 0.43) as u16),
        PrintStyledContent("ESC - Quit".red().bold())
    )?;

    Ok(())
}

pub fn single_player(stdout: &mut Stdout, state: &mut State) -> io::Result<()> {
    print_help(stdout, &state)?;

    let columns = state.columns;

    let player = &mut state.players[state.current_player];

    let elapsed_millis = state.instant.elapsed().as_millis();

    let should_go_forward: bool = elapsed_millis - state.last_instant > 500;

    let mut add_x: u16 = 4;

    for (i, word) in state.dictionary[player.position..player.position + 4]
        .iter_mut()
        .enumerate()
    {
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

        if correct_chars == word.value.len() || word.x >= columns {
            player.input.clear();
            player.position += 1;
        }

        if should_go_forward {
            word.x += add_x;
            state.last_instant = elapsed_millis;
        }

        add_x -= 1;
    }

    Ok(())
}

pub fn multi_player(stdout: &mut Stdout, state: &mut State) -> io::Result<()> {
    print_help(stdout, state)?;

    let socket = state.socket.as_ref().unwrap();

    for action in socket.actions().drain(..) {
        match action {
            Action::Join(position) => {
                let mut player = Player::default();
                player.sort_position = position;
                state.players.push(player);
                state.players.sort_by(|player_a, player_b| {
                    player_a.sort_position.cmp(&player_b.sort_position)
                });
                for (i, player) in state.players.iter().enumerate() {
                    if player.current_player {
                        state.current_player = i;
                        break;
                    }
                }
            }
            Action::Left(position) => {
                state.players.remove(position);
                state.players.sort_by(|player_a, player_b| {
                    player_a.sort_position.cmp(&player_b.sort_position)
                });
                for (i, player) in state.players.iter().enumerate() {
                    if player.current_player {
                        state.current_player = i;
                        break;
                    }
                }
            }
            Action::Input((position, c)) => {
                if let Some(player) = state.players.get_mut(position) {
                    player.input.push(c);
                }
            }
        };
    }

    let (columns, rows) = (state.columns, state.rows as f32);

    let elapsed_millis = state.instant.elapsed().as_millis();

    let should_go_forward: bool = elapsed_millis - state.last_instant > 500;

    let players_len = state.players.len();

    let space_per_player = (rows / players_len as f32) as u16 - 2;

    // Could further optimize the code by caching this line in the state
    // and only updating it once a Resize occurs, but don't wanna deal
    // with the extra complexity right now, gotta keep it simple.
    let line: String = iter::repeat('-').take(columns.into()).collect();

    // Might use multithreading to calculate each player's section,
    // but that might be overengineering too, so we'll see.
    for (i, player) in state.players.iter_mut().enumerate() {
        let y_start = (i as u16 * space_per_player);
        let y_end = y_start + space_per_player;
        let color = match i {
            0 => Color::Blue,
            1 => Color::Red,
            2 => Color::Green,
            3 => Color::Yellow,
            _ => Color::White,
        };
        queue!(
            stdout,
            MoveTo(0, y_end),
            PrintStyledContent(style(&line).with(color))
        )?;

        let mut add_x: u16 = 4;

        for (j, word) in state.dictionary[player.position..player.position + 4]
            .iter_mut()
            .enumerate()
        {
            let mut correct_chars = 0;

            let word_y = ((word.y as f32 / rows) * y_end as f32 + y_start as f32) as u16 + 1;

            for (n, c) in word.value.chars().enumerate() {
                let mut color = Color::White;
                let mut boldness = Attribute::NormalIntensity;

                if j == 0 {
                    if let Some(d) = player.input.chars().nth(n) {
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
                    MoveTo(word.x + n as u16, word_y),
                    PrintStyledContent(style(c).with(color).attribute(boldness))
                )?;
            }

            if correct_chars == word.value.len() || word.x >= columns {
                player.input.clear();
                player.position += 1;
            }

            if should_go_forward {
                word.x += add_x;
                state.last_instant = elapsed_millis;
            }

            add_x -= 1;
        }
    }

    Ok(())
}

pub fn join(stdout: &mut Stdout, state: &mut State) -> io::Result<()> {
    print_help(stdout, &state)?;

    let (columns, rows) = (state.columns as f32, state.rows as f32);

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

    let x_end = (columns * 0.599) as u16;

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

    let player = state.players.get(state.current_player).unwrap();

    let x_end = x_end - 1;

    let x = x_start + 1;

    let y = (rows * 0.435) as u16;

    for (i, c) in player.input.chars().enumerate() {
        let x = x + i as u16;
        if x > x_end {
            break;
        }
        queue!(stdout, MoveTo(x, y), Print(c))?;
    }

    Ok(())
}

fn print_help(stdout: &mut Stdout, state: &State) -> io::Result<()> {
    queue!(
        stdout,
        MoveTo(0, state.rows),
        PrintStyledContent("ESC - Go Back".yellow().bold())
    )?;

    Ok(())
}
