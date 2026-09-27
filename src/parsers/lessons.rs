use std::{io::Cursor, mem::transmute};

use office_oxide::{Document, DocumentFormat};
use tracing::info;

use crate::errors::Error;

const CLASSES_COUNT: usize = 38;
const CLASSES: [&'static str; CLASSES_COUNT] = [
    "5а", "5б", "5в", "5г", "6а", "6б", "6в", "6г", "7а", "7б", "7в", "8а", "8б", "8в", "8г", "9а",
    "9б", "9в", "9г", "10а", "10б", "10в", "11а", "11б", "1а", "1б", "1в", "2а", "2б", "2в", "3а",
    "3б", "3в", "2г", "4а", "4б", "4в", "4г",
];
const LESSONS_COUNT: usize = 10;
const STUDY_DAYS_COUNT: usize = 6;

pub struct Parser {
    pub lessons: [[[u8; LESSONS_COUNT]; CLASSES_COUNT]; STUDY_DAYS_COUNT],
}

impl Parser {
    pub fn new() -> Self {
        Self {
            lessons: [[[0u8; LESSONS_COUNT]; CLASSES_COUNT]; STUDY_DAYS_COUNT],
        }
    }

    pub unsafe fn reparse(&mut self, content: &[u8]) -> Result<(), Error> {
        let static_content: &'static str = unsafe { transmute(content) };
        let document =
            Document::from_reader(Cursor::new(static_content), DocumentFormat::Xlsx)?.plain_text();
        let third_line: &'static str =
            unsafe { transmute(document.lines().nth(2).ok_or_else(|| Error::Document)?) };
        self.extract_classes(third_line);
        self.lessons(document);
        Ok(())
    }

    fn lessons(&mut self, document: String) {
        for (i, line) in document.lines().skip(3).enumerate() {
            if line.starts_with("\"Шко") {
                // \"Школьный диспетчер\"
                return;
            };
            self.row(line.split("\t"), i);
        }
    }

    pub fn get_by_class(&self, class: &str) -> Option<Vec<&str>> {
        if !self.classes.contains(&class) {
            return None;
        };

        None
    }

    fn row<'b>(&mut self, row: impl Iterator<Item = &'b str>, index: usize) {
        let day_index = index / LESSONS_COUNT + 1;
        if index % (LESSONS_COUNT + 1) == 0 {
            return;
        };
        for block in row {}
    }

    fn extract_classes(&mut self, line: &'a str) {
        let mut i = 0;
        for data in line.split("\t") {
            if data == "№" || data.is_empty() {
                continue;
            };
            self.classes[i] = data;
            i += 1;
            if i == CLASSES_COUNT {
                return;
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse() {
        let content = include_bytes!("../../mock_file.xls");
        let mut lessons = Parser::new();
        unsafe { lessons.reparse(content).unwrap() };
        lessons.get_by_class("5а");

        println!("Classes: {:?}", lessons.classes);
        assert_eq!(
            lessons
                .classes
                .iter()
                .filter(|class| !class.is_empty())
                .collect::<Vec<&&str>>()
                .len(),
            CLASSES_COUNT
        )
    }
}
