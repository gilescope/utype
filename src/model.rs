
pub struct Page {
    pub lines: Vec<Line>,
}

pub struct Cursor<'page> {
    page: &'page Page,
    current_line: usize,
    current_word: usize,
    current_char: usize,
}

impl <'page> Cursor<'page> {
    pub fn new(page: &'page Page) -> Cursor<'page> {
        Cursor {
            page,
            current_line: 0,
            current_word: 0,
            current_char: 0,
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

    pub fn current_char(&self) -> Option<char> {
        let mut chars = 0;
        for word in self.current_line().words.iter() {
            for ch in word.chars.iter() {
               
                if chars == self.current_char {
                    return Some(*ch);         
                } 
                chars+=1;
            }
           
           
            if chars == self.current_char {
                    return Some(' ');         
                }
                 chars += 1; // for the space
        }
        None
    }

    pub fn is_current_word_last_in_line(&self) -> bool {
        self.current_word == self.current_line().words.len().saturating_sub(1)
    }

    // pub fn next_word(&mut self) -> Option<&Word> {
    //     self.current_word += 1;
    //     let has_word = self.current_line().words.get(self.current_word).is_some();

    //     if has_word {
    //         self.current_line().words.get(self.current_word)
    //     } else {
    //         self.current_line += 1;
    //         self.current_word = 0;
    //         self.next_word()
    //     }
    // }

    // pub fn prev_word(&mut self) -> Option<&Word> {
    //     if self.current_word == 0 {
    //         self.current_line = self.current_line.saturating_sub(1);
    //         self.current_word = self.current_line().words.len().saturating_sub(1);
    //     } else {
    //         self.current_word -= 1;
    //     }
    //     self.current_line().words.get(self.current_word)
    // }

    pub fn at_start(&self) -> bool {
        self.current_line == 0 && self.current_word == 0 && self.current_char == 0
    }

    pub fn next_char(&mut self) {
        self.current_char += 1;
        if self.current_char > self.current_line().len() {
            self.current_line += 1;
            self.current_word = 0;
            self.current_char = 0;
        }
        self.calc_current_word();
    }

    pub fn prev_char(&mut self) {
        if self.current_char == 0 {
            if self.current_line == 0 {
                return;
            }
            self.current_line = self.current_line.saturating_sub(1);
            self.current_char = self.current_line().len();
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

impl Page {
    pub fn parse(text: &str) -> Self {
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
        Self {
            lines: page
        }
    }

}

pub struct Line {
    pub words: Vec<Word>,
}

impl Line {
    fn len(&self) -> usize {
        self.words
            .iter()
            .fold(0, |acc, word| acc + word.chars.len())
            + self.words.len().saturating_sub(1)
    }
}

#[derive(Debug, Default)]
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
mod test{
    use super::*;

    #[test]
    fn test_parse() {
        let page = Page::parse("The quick brown fox jumps over the lazy dog.");
    
        let mut cursor = Cursor::new(&page);

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
    }
}