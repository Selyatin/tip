mod word;

use word::Word;
use lazy_static::lazy_static;
use std::{
    io::{Write, stdout},
    thread,
    time::{Instant, Duration},
    sync::{
        mpsc::{channel, Sender, Receiver}
    }
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

fn main() {

    terminal::enable_raw_mode().unwrap();

    // Get initial terminal size
    let (mut columns, mut rows) = terminal::size().expect("Couldn't get Terminal Size");

    let mut rng = thread_rng();

    // Include the dictionary file as a string
    let dictionary_string = include_str!("../dictionary.txt");

    let mut dictionary: Vec<Word> = dictionary_string
        .split("\n")
        .map(|s| Word::new(s, 0, rng.gen_range(0..rows)))
        .collect();

    let mut stdout = stdout();

    let instant = Instant::now();

    let mut last_instant = 0;

    let mut input: Vec<char> = vec![];

    queue!(stdout, Clear(ClearType::All), MoveTo(columns / 2, rows / 2), Print("Shuffling Dictionary...")).unwrap();

    stdout.flush().unwrap();

    dictionary.shuffle(&mut rng);

    loop {
        queue!(stdout, Clear(ClearType::All)).unwrap();

        let elapsed_millis = instant.elapsed().as_millis();

        let should_go_forward: bool = elapsed_millis - last_instant > 500;

        let mut i = dictionary.len() - 1;
        
        let mut add_x: u16 = 4;

        while i > dictionary.len() - 5 { 
            let dictionary_len = dictionary.len();
            
            let word = &mut dictionary[i];

            let mut correct_chars = 0;

            for (j, c) in word.value.chars().enumerate() {
                let mut color = Color::White;
                if let Some(d) = input.get(j) {
                    if c == *d && i == dictionary_len - 1{
                        color = Color::Green;
                        correct_chars += 1;
                    } else if i == dictionary_len - 1{
                        color = Color::Red;
                    }
                }
                queue!(stdout, MoveTo(word.x + j as u16, word.y), PrintStyledContent(style(c).with(color))).unwrap();
            }

            i -= 1;

            if correct_chars == word.value.len() {
                drop(word);
                dictionary.pop();
                input.clear();
                continue;
            }

            if should_go_forward {
                word.x += add_x;
            } 
            
            add_x -= 1;

            if word.x >= columns {
                break;
            }

            if should_go_forward{
                last_instant = elapsed_millis;
            }

            queue!(stdout, MoveTo(columns, rows)).unwrap();

            // Render the queued frame
            stdout.flush().unwrap();
        }

        // Sleep 16 millis so that we render 60 fps
        if event::poll(Duration::from_millis(16)).unwrap() {
            match event::read().unwrap() {
                Event::Key(KeyEvent{code: KeyCode::Char(c), modifiers: KeyModifiers::NONE}) => {
                    input.push(c);
                },
                Event::Key(KeyEvent{code: KeyCode::Backspace, modifiers: KeyModifiers::NONE}) => {
                    input.pop();
                },
                Event::Key(KeyEvent{code: KeyCode::Char('c'), modifiers: KeyModifiers::CONTROL}) => {
                    break;
                },
                Event::Resize(new_columns, new_rows) => {
                    columns = new_columns;
                    rows = new_rows;

                    for word in &mut dictionary {
                        word.y = rng.gen_range(0..rows);
                    }
                },
                _ => ()        
            };
        }
    }

    terminal::disable_raw_mode().unwrap();
}
