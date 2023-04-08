#![feature(associated_type_bounds)]
use std::fs::File;
use std::io::Read;
use std::io::{stdin, stdout, Write};
use std::time::Instant;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
mod model;

use model::*;

const HEADER: u16 = 5;

fn draw_screen(page: &Page, cursor: &Cursor, copy: &Page, clear: bool) {
    if clear {
        print!("{}", termion::clear::All)
    };
    print!(
        r#"{}{}"#,
        termion::cursor::Goto(1, HEADER),
        &termion::color::Fg(termion::color::LightBlue)
    );

    for (line_index, line) in page
        .lines
        .iter()
        .enumerate()
        .skip(cursor.window.start_line)
        .take(cursor.window.size)
    {
        let mut is_first = true;
        if let Some(copy_line) = copy.lines.get(line_index) {
            for (word_index, word) in line.words.iter().enumerate() {
                if is_first {
                    is_first = false;
                } else {
                    print!(" ");
                }

                if let Some(copy_word) = copy_line.words.get(word_index) {
                    for (index, ch) in word.chars.iter().enumerate() {
                        if copy_word.chars.len() > index {
                            if copy_word.chars[index] == *ch {
                                print!("{}", termion::color::Fg(termion::color::Green));
                            } else {
                                print!("{}", termion::color::Fg(termion::color::Red));
                            }
                            print!("{}", copy_word.chars[index]);
                            print!("{}", termion::color::Fg(termion::color::LightBlue));
                        } else {
                            print!("{ch}");
                        }
                    }
                } else {
                    print!("{}", word.chars.iter().collect::<String>());
                }
            }
        } else {
            for word in line.words.iter() {
                if is_first {
                    is_first = false;
                } else {
                    print!(" ");
                }
                print!("{}", word.chars.iter().collect::<String>());
            }
        }
        print!("\r\n");
    }
}

/// read or create a file with the current state of the game
fn read_or_create(path: &str) -> String {
    let mut file = File::options()
        .create(true)
        .read(true)
        .append(true)
        .open(path)
        .unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    contents
}

/// Words per minute are defined as being 5 chars long (not including spaces - we're harsher here)
fn main() {
    let (_terminal_width, terminal_height) = termion::terminal_size().unwrap();
    let text = std::fs::read_to_string("/home/gilescope/git/type/war_of_the_worlds.txt").unwrap();
    let input_text = read_or_create("/home/gilescope/git/type/war_of_the_worlds.inputs.txt");
    let stdin = stdin();
    //setting up stdout and going into raw mode
    let mut stdout = stdout().into_raw_mode().unwrap();

    //clear the screen and go to top left corner with the cursor
    print!(r#"{}{}"#, termion::cursor::Goto(1, 1), termion::clear::All);
    let window = Window {
        start_line: 0,
        size: (terminal_height as usize).saturating_sub(HEADER as usize),
    };

    let page = Page::parse(&text, false);
    let mut copy = Page::parse("", true);
    let mut cursor = Cursor::new(&page, window);
    draw_screen(&page, &cursor, &copy, true);

    print!(r#"{}"#, termion::cursor::Goto(1, HEADER));

    stdout.flush().unwrap();

    let mut start: Option<Instant> = None;

    let mut analytics = Analytics {
        cumulative_chars_typed: 0,
        cumulative_time_taken: 0,
        success: 0,
        failure: 0,
        percentage: 0.0,
    };

    for mut c in input_text.chars() {
        if c == '\n' {
            c = ' '
        }
        process_key(
            &page,
            &mut copy,
            Key::Char(c),
            &mut stdout,
            &mut cursor,
            &mut analytics,
            &mut start,
        );
    }
    for c in stdin.keys() {
        process_key(
            &page,
            &mut copy,
            c.unwrap(),
            &mut stdout,
            &mut cursor,
            &mut analytics,
            &mut start,
        );
    }
}

struct Analytics {
    cumulative_chars_typed: usize,
    cumulative_time_taken: u128,
    success: u32,
    failure: u32,
    percentage: f32,
}

fn process_key<T: Write>(
    page: &Page,
    copy: &mut Page,
    c: Key,
    stdout: &mut termion::raw::RawTerminal<T>,
    cursor: &mut Cursor,
    ctx: &mut Analytics,
    start: &mut Option<Instant>,
) {
    let mut clear = false;
    match c {
        Key::Ctrl('c') => {
            stdout.suspend_raw_mode().unwrap();
            std::process::exit(0)
        }
        Key::Backspace | Key::Delete => {
            copy.delete();
            let current_word = cursor.word_number();
            cursor.prev_char();

            if cursor.word_number() != current_word {
                ctx.cumulative_chars_typed = ctx.cumulative_chars_typed.saturating_sub(
                    cursor
                        .current_word()
                        .unwrap_or(&Word { chars: vec![] })
                        .chars
                        .len(),
                );
            }
        }
        Key::Char(c) => {
            //if meant to be a space but we got it wrong put an _ in there.
            let current_word = cursor.current_word_or_default().clone();
            let last_word = copy.last_word_or_default();
            if c == ' ' || last_word.chars.len() == current_word.chars.len() {
                let current_word = cursor.current_word_or_default().clone();
                let trial = copy.last_word_or_default().clone();
                let is_short = trial.chars.len() < current_word.chars.len();

                let new_line = cursor.next_char();
                if new_line {
                    clear = true;
                }
                let expected = *current_word.chars.get(trial.chars.len()).unwrap_or(&'X');
                if is_short {
                    let ascii_case_bit = 0b00100000;
                    let is_alpha = expected.is_ascii_alphabetic();
                    let c = expected as u32;
                    let c = if is_alpha {
                        char::from_u32(c ^ ascii_case_bit).unwrap()
                    } else {
                        'X'
                    };
                    // trial.push(c);
                    copy.push(c, false);
                } else {
                    copy.push(' ', new_line);
                }
                if *trial.chars == current_word.chars {
                    ctx.cumulative_chars_typed += current_word.chars.len();

                    ctx.success += 1;
                    if let Some(start) = start {
                        let duration: u128 = (Instant::now() - *start).as_millis();
                        ctx.cumulative_time_taken += duration;
                    }
                } else {
                    ctx.failure += 1;
                }

                if !is_short {
                    *start = None;
                }
            } else {
                cursor.next_char();
                copy.push(c, false);

                if start.is_none() {
                    *start = Some(Instant::now());
                }
            }
        }
        _ => {}
    }

    draw_screen(page, cursor, copy, clear);
    let wpm = if ctx.cumulative_time_taken == 0 {
        0
    } else {
        ((ctx.cumulative_chars_typed as f32 / 5.) / (ctx.cumulative_time_taken as f32 / 60000.))
            as u32
    };

    ctx.percentage = if ctx.success == 0 {
        0.
    } else {
        100. * ((ctx.success as f32) / ((ctx.success + ctx.failure) as f32))
    };

    print!(
        "{}{}{}:{}:{}     WPM:{} {:.2}%   copy:{:?}  page:{:?}  {}", // copy:{:?}  page:{:?}
        termion::cursor::Save,
        termion::cursor::Goto(1, 1),
        cursor.line_number(),
        cursor.word_number(),
        copy.last_word_or_default().chars.len(),
        wpm,
        ctx.percentage,
        copy.lines.first().unwrap_or(&Line { words: vec![] }),
        page.lines.first().unwrap_or(&Line { words: vec![] }),
        termion::cursor::Restore
    );

    print!(
        r#"{}"#,
        termion::cursor::Goto(
            cursor.char_number(),
            cursor.line_number() + HEADER - 1 - cursor.window.start_line as u16
        )
    );
    stdout.flush().unwrap();
}
