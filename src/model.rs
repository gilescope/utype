use std::fmt::Display;
use std::fmt::Formatter;
use std::fs::File;
use std::io::Write;

pub struct Page {
    pub lines: Vec<Line>,
    pub save: bool,
}

pub struct Cursor<'page> {
    page: &'page Page,
    current_line: usize,
    current_word: usize,
    current_char: usize,
    pub window: Window,
}

pub struct Window {
    pub start_line: usize,
    pub size: usize,
}

const WINDOW_MARGIN: usize = 2;

impl<'page> Cursor<'page> {
    pub fn new(page: &'page Page, window: Window) -> Cursor<'page> {
        Cursor {
            page,
            current_line: 0,
            current_word: 0,
            current_char: 0,
            window,
        }
    }

    pub fn line_number(&self) -> u16 {
        self.current_line as u16 + 1
    }

    pub fn word_number(&self) -> u16 {
        self.current_word as u16 + 1
    }

    pub fn char_number(&self) -> u16 {
        self.current_char as u16 + 1
    }

    pub fn current_line(&self) -> &Line {
        self.page.lines.get(self.current_line).unwrap()
    }

    pub fn current_word(&self) -> Option<&Word> {
        self.current_line().words.get(self.current_word)
    }

    pub fn current_word_or_default(&self) -> &Word {
        self.current_word().unwrap_or(EMPTY_WORD)
    }

    /// retuns true if the cursor moved to the next line
    pub fn next_char(&mut self) -> bool {
        let mut moved_to_next_line = false;
        self.current_char += 1;
        if self.current_char > self.current_line().len() {
            self.current_line += 1;
            self.current_word = 0;
            self.current_char = 0;
            moved_to_next_line = true;

            if self.current_line >= self.window.size + self.window.start_line - WINDOW_MARGIN {
                self.window.start_line += 1;
            }
        }
        self.calc_current_word();
        moved_to_next_line
    }

    pub fn prev_char(&mut self) {
        if self.current_char == 0 {
            if self.current_line == 0 {
                return;
            }
            self.current_line = self.current_line.saturating_sub(1);
            self.current_char = self.current_line().len();

            if self.current_line <= self.window.start_line + WINDOW_MARGIN {
                self.window.start_line = self.window.start_line.saturating_sub(1);
            }
        } else {
            self.current_char -= 1;
        }
        self.calc_current_word();
    }

    fn calc_current_word(&mut self) {
        let mut chars = 0;
        for (i, word) in self.current_line().words.iter().enumerate() {
            chars += word.chars.len();
            chars += 1; // for the space
            if chars > self.current_char {
                self.current_word = i;
                break;
            }
        }
    }
}

static EMPTY_WORD: &Word = &Word { chars: vec![] };

impl Page {
    pub fn parse(text: &str, save: bool) -> Self {
        let lines = text.lines();
        let mut page: Vec<Line> = vec![];
        let mut last_line_was_empty = false;
        for line in lines {
            last_line_was_empty = if line.is_empty() {
                if last_line_was_empty {
                    continue;
                }
                true
            } else {
                false
            };

            let mut parsed_line: Vec<Word> = vec![];
            line.split_whitespace().for_each(|word| {
                parsed_line.push(Word::parse(word));
            });
            page.push(Line { words: parsed_line });
        }
        Self { lines: page, save }
    }

    pub(crate) fn last_word(&self) -> Option<&Word> {
        self.lines.last().and_then(|line| line.words.last())
    }
    pub(crate) fn last_word_or_default(&self) -> &Word {
        self.last_word().unwrap_or(EMPTY_WORD)
    }

    pub(crate) fn push(&mut self, arg: char, new_line: bool) {
        if self.lines.is_empty() {
            let line = Line::default();
            self.lines.push(line);
        }
        let last_line = self.lines.last_mut().unwrap();
        if last_line.words.is_empty() {
            last_line.words.push(Word::default());
        }
        let last_word = last_line.words.last_mut().unwrap();
        match arg {
            ' ' => {
                if new_line {
                    let mut line = Line::default();
                    line.words.push(Word::default());
                    self.lines.push(line);
                } else {
                    let skip = if let Some(last_word) = last_line.words.last() {
                        last_word.chars.is_empty()
                    } else {
                        false
                    };
                    if !skip {
                        last_line.words.push(Word::default());
                    }
                }
            }
            _ => {
                last_word.chars.push(arg);
            }
        }
        self.changed();
    }

    pub fn delete(&mut self) {
        if self.lines.is_empty() {
            return;
        }
        let last_line = self.lines.last_mut().unwrap();
        if let Some(last_word) = last_line.words.last_mut() {
            if last_word.chars.pop().is_none() && last_line.words.pop().is_none() {
                self.lines.pop();
            }
        } else {
            self.lines.pop();
        }
        self.changed();
    }

    fn changed(&self) {
        if self.save {
            File::create("war_of_the_worlds.inputs.txt")
                .unwrap()
                .write_all(self.to_string().as_bytes())
                .unwrap();
        }
    }
}

impl Display for Page {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for line in self.lines.iter() {
            for (word_index, word) in line.words.iter().enumerate() {
                if word_index != 0 {
                    write!(f, " ")?;
                }
                for ch in word.chars.iter() {
                    write!(f, "{ch}")?;
                }
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct Line {
    pub words: Vec<Word>,
}

impl Line {
    pub fn len(&self) -> usize {
        self.words
            .iter()
            .fold(0, |acc, word| acc + word.chars.len())
            + self.words.len().saturating_sub(1)
    }
}

#[derive(Debug, Default, Clone)]
pub struct Word {
    pub chars: Vec<char>,
}

impl Word {
    fn parse(text: &str) -> Self {
        Self {
            // Keep the positions the same as the original text
            chars: text
                .chars()
                .filter(|c| c.is_ascii())
                // .map(|c| if c.is_ascii() { c } else { ' ' })
                .collect(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse() {
        let mut page = Page::parse("The quick brown fox jumps over the lazy dog.", false);

        let mut cursor = Cursor::new(
            &page,
            Window {
                start_line: 0,
                size: 1,
            },
        );

        assert_eq!(cursor.current_line().words.len(), 9);
        assert_eq!(cursor.current_word().unwrap().chars.len(), 3);

        cursor.next_char();
        cursor.next_char();
        cursor.next_char();
        assert_eq!(cursor.current_word().unwrap().chars.len(), 3);
        cursor.next_char();
        assert_eq!(cursor.current_word().unwrap().chars.len(), 5);

        cursor.prev_char();
        assert_eq!(cursor.current_word().unwrap().chars.len(), 3);

        assert_eq!(
            page.to_string(),
            "The quick brown fox jumps over the lazy dog.\n"
        );
        page.push(' ', true);
        page.push('r', false);
        assert_eq!(
            page.to_string(),
            "The quick brown fox jumps over the lazy dog.\nr\n"
        );
        page.push(' ', false);
        page.push('c', false);
        assert_eq!(
            page.to_string(),
            "The quick brown fox jumps over the lazy dog.\nr c\n"
        );
    }
}
