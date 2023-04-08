use std::io::{stdin, stdout, Write};
use std::time::Instant;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

mod model;

use model::*;

/// Words per minute are defined as being 5 chars long (not including spaces - we're harsher here)
fn main() {
    let text = std::fs::read_to_string("/home/gilescope/git/type/war_of_the_worlds.txt").unwrap();
    // let lines = text.lines();
    let stdin = stdin();
    //setting up stdout and going into raw mode
    let mut stdout = stdout().into_raw_mode().unwrap();

    //printing welcoming message, clearing the screen and going to left top corner with the cursor
    print!(r#"{}{}"#, termion::cursor::Goto(1, 1), termion::clear::All);

    let page = Page::parse(&text);
    let mut cursor = Cursor::new(&page);
    print!(r#"{}"#, termion::cursor::Goto(1, 3));
    for line in page.lines.iter().take(10) {
        let mut is_first = true;
        for word in line.words.iter() {
            if is_first {
                is_first = false;
            } else {
                print!(" ");
            }
            print!("{}", word.chars.iter().collect::<String>());
        }
        print!("\r\n");
    }

    print!(r#"{}"#, termion::cursor::Goto(1, 3));

    stdout.flush().unwrap();

    let mut cumulative_chars_typed = 0;
    let mut cumulative_time_taken: u128 = 0;

    let mut target: Vec<char> = cursor.current_word().unwrap().chars.clone();
    let mut trial: Vec<char> = vec![];
    let mut start: Option<Instant> = None;

    let mut success: u32 = 0;
    let mut failure: u32 = 0;

    let mut percentage: f32;

    for c in stdin.keys() {
        match c.unwrap() {
            Key::Ctrl('w') | Key::Ctrl('c') => {
                stdout.suspend_raw_mode().unwrap();
                std::process::exit(0)
            }
            Key::Char(' ') => {
                print!(" ");
                cursor.next_char();
                if trial == target {
                    cumulative_chars_typed += target.len();

                    target = cursor
                        .current_word()
                        .unwrap_or(&Word { chars: vec![] })
                        .chars
                        .clone();
                    success += 1;
                    if let Some(start) = start {
                        let duration: u128 = (Instant::now() - start).as_millis();
                        cumulative_time_taken += duration;
                    }
                } else {
                    failure += 1;
                }
                start = None;
                trial.clear();
            }
            Key::Backspace | Key::Delete => {
                let current_word = cursor.word_number();
                let current_char = cursor.char_number();
                cursor.prev_char();

                let c = cursor.current_char().unwrap_or(' ');
                print!(
                    "{}{}{}{}",
                    termion::cursor::Left(1),
                    &termion::color::Fg(termion::color::Blue),
                    c,
                    termion::cursor::Left(1)
                );

                if cursor.word_number() != current_word {
                    cumulative_chars_typed = cumulative_chars_typed.saturating_sub(
                        cursor
                            .current_word()
                            .unwrap_or(&Word { chars: vec![] })
                            .chars
                            .len(),
                    );
                }
                if cursor.char_number() < current_char {
                    if trial.len() > 0 {
                        trial.pop();
                    } else {
                        target = cursor
                            .current_word()
                            .unwrap_or(&Word { chars: vec![] })
                            .chars
                            .clone();
                        trial = target.clone();
                    }
                }
            }
            Key::Char(c) => {
                cursor.next_char();
                if start == None {
                    start = Some(Instant::now());
                }
                trial.push(c);
                let same = trial.len() <= target.len() && target[trial.len() - 1] == c;

                if same {
                    print!("{}{}", &termion::color::Fg(termion::color::Green), c);
                } else {
                    print!("{}{}", &termion::color::Fg(termion::color::Red), c);
                };
            }
            _ => {}
        }

        let wpm = if cumulative_time_taken == 0 {
            0
        } else {
            ((cumulative_chars_typed as f32 / 5.) / (cumulative_time_taken as f32 / 60000.)) as u32
        };

        percentage = if success == 0 {
            0.
        } else {
            100. * ((success as f32) / ((success + failure) as f32))
        };

        print!(
            "{}{}{}:{}:{}     WPM:{} {:.2}%   {}",
            termion::cursor::Save,
            termion::cursor::Goto(1, 1),
            cursor.line_number(),
            cursor.word_number(),
            trial.len(),
            wpm,
            percentage,
            termion::cursor::Restore
        );

        print!(
            r#"{}"#,
            termion::cursor::Goto(cursor.char_number(), cursor.line_number() as u16 + 2)
        );
        stdout.flush().unwrap();
    }
}
