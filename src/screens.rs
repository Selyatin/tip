use super::types::{Action, Player, Screen, State};
use crossterm::{
    cursor::MoveTo,
    queue,
    style::{style, Attribute, Color, Print, PrintStyledContent, Stylize},
    terminal::{Clear, ClearType},
};
use std::{
    io::{self, Stdout},
    iter,
};

pub fn main(stdout: &mut Stdout, state: &State) -> io::Result<()> {
    let (x, y) = (
        (state.columns as f32 * 0.4) as u16,
        (state.rows as f32 * 0.41) as u16,
    );

    queue!(
        stdout,
        Clear(ClearType::All),
        MoveTo(x, y),
        PrintStyledContent("F1 - Single Player".green().bold()),
        MoveTo(x, y + 1),
        PrintStyledContent("F2 - Create New Multiplayer Session".yellow().bold()),
        MoveTo(x, y + 2),
        PrintStyledContent("F3 - Join Session".blue().bold()),
        MoveTo(x, y + 3),
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

    let mut should_go_forward = false;

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
                    if c == '-' {
                        player.input.pop();
                    } else {
                        player.input.push(c);
                    }
                }
            }
            Action::Forward => should_go_forward = true,
        };
    }

    let (columns, rows) = (state.columns, state.rows as f32);

    let players_len = state.players.len();

    let space_per_player = ((rows - 2.0) / players_len as f32) as u16;

    let line: String = iter::repeat('-').take(columns.into()).collect();

    // Might use multithreading to calculate each player's section,
    // but that might be overengineering too, so we'll see.
    for (i, player) in state.players.iter_mut().enumerate() {
        let y_start = i as u16 * space_per_player;

        let y_end = y_start + space_per_player;

        let color = match i {
            0 => Color::Blue,
            1 => Color::Red,
            2 => Color::Green,
            3 => Color::Yellow,
            _ => Color::White,
        };

        let print_you = if player.current_player { " (You)" } else { "" };

        queue!(
            stdout,
            MoveTo(0, y_end),
            PrintStyledContent(style(&line).with(color)),
            MoveTo(5, y_end),
            PrintStyledContent("Player ".with(color)),
            PrintStyledContent(style(i + 1).with(color)),
            PrintStyledContent(style(print_you).with(color))
        )?;

        let mut add_x: u16 = 4;

        for (j, word) in state.dictionary[player.position..player.position + 4]
            .iter_mut()
            .enumerate()
        {
            let mut correct_chars = 0;
            
            let (y_start, y_end) = (y_start + 1, y_end - 1);

            let word_y = ((word.y as f32 / rows) * (y_end - y_start) as f32) as u16 + y_start;

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
                add_x -= 1;
            }
        }
    }

    Ok(())
}

pub fn loading(stdout: &mut Stdout, state: &mut State) -> io::Result<()> {
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
            Action::Forward => {
                state.screen = Screen::MultiPlayer;
                return Ok(());
            }
            _ => (),
        };
    }

    let (columns, rows) = (
        (state.columns as f32 * 0.35) as u16,
        (state.rows as f32 * 0.4) as u16,
    );

    queue!(
        stdout,
        MoveTo(columns, rows),
        PrintStyledContent(
            "Waiting 10 seconds for other players to join."
                .green()
                .bold()
        ),
        MoveTo(columns, rows + 2)
    )?;

    for (i, player) in state.players.iter().enumerate() {
        let color = match i {
            0 => Color::Blue,
            1 => Color::Red,
            2 => Color::Green,
            3 => Color::Yellow,
            _ => Color::White,
        };

        let print_you = if player.current_player { " (You)" } else { "" };

        queue!(
            stdout,
            PrintStyledContent("Player ".with(color).bold()),
            PrintStyledContent(style(i + 1).with(color).bold()),
            PrintStyledContent(style(print_you).with(color).bold()),
            Print(' ')
        )?;
    }

    Ok(())
}

pub fn join(stdout: &mut Stdout, state: &mut State) -> io::Result<()> {
    print_help(stdout, &state)?;

    let (columns, rows) = (state.columns as f32, state.rows as f32);

    let (mut x_start, mut x_end) = ((columns * 0.4) as u16, (columns * 0.6) as u16);

    let mut y_start = (rows * 0.4) as u16;
    let y_end = y_start + 2;

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

    x_end -= 1;
    y_start += 1;

    queue!(
        stdout,
        MoveTo(x_start, y_start),
        Print('|'),
        MoveTo(x_end, y_start),
        Print('|')
    )?;

    let player = state.players.get(state.current_player).unwrap();

    x_end = x_end - 1;

    x_start += 1;

    for (i, c) in player.input.chars().enumerate() {
        let x = x_start + i as u16;
        if x > x_end {
            break;
        }
        queue!(stdout, MoveTo(x, y_start), Print(c))?;
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
