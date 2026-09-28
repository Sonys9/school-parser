use std::{cmp::Ordering::Less, fmt::Debug, io::Cursor, mem::transmute};

use office_oxide::{Document, DocumentFormat};
use tracing::info;

use crate::{errors::Error, parsers::table};

const CLASSES_COUNT: usize = 38;
const CLASSES: [&'static str; CLASSES_COUNT] = [
    "5а", "5б", "5в", "5г", "6а", "6б", "6в", "6г", "7а", "7б", "7в", "8а", "8б", "8в", "8г", "9а",
    "9б", "9в", "9г", "10а", "10б", "10в", "11а", "11б", "1а", "1б", "1в", "2а", "2б", "2в", "3а",
    "3б", "3в", "2г", "4а", "4б", "4в", "4г",
];
const LESSONS_COUNT: usize = 10;
const STUDY_DAYS_COUNT: usize = 6;

pub struct Parser {
    pub lessons: [[[Lesson; LESSONS_COUNT]; CLASSES_COUNT]; STUDY_DAYS_COUNT],
}

impl Parser {
    pub fn new() -> Self {
        Self {
            lessons: [[[Lesson::default(); LESSONS_COUNT]; CLASSES_COUNT]; STUDY_DAYS_COUNT],
        }
    }

    pub unsafe fn reparse(&mut self, content: &[u8]) -> Result<(), Error> {
        let static_content: &'static str = unsafe { transmute(content) };
        let document =
            Document::from_reader(Cursor::new(static_content), DocumentFormat::Xlsx)?.plain_text();
        info!("content parsed len: {}", document.len());
        self.lessons(document);
        Ok(())
    }

    fn lessons(&mut self, document: String) {
        for (i, line) in document.lines().skip(3).enumerate() {
            if line.starts_with("\"Шко") {
                // \"Школьный диспетчер\"
                info!("sum: {}", i);
                return;
            };
            self.row(line.split("\t").skip(1), i);
        }
    }

    pub fn get_by_class(&self, class: &str) -> Option<Vec<&str>> {
        let Some(class_index) = CLASSES.iter().position(|&pclass| pclass == class) else {
            return None;
        };
        for (i, lessons) in self.lessons.iter().enumerate() {
            info!("=== {}th day of the week ===", i + 1);
            for (i, lesson) in lessons[class_index].iter().enumerate() {
                if lesson.name_id == 0 && lesson.class_id == 0 {
                    continue;
                };
                info!("[{}] {}: {}", i + 1, table::name(lesson.name_id).unwrap(), table::name(lesson.class_id).unwrap());
            }
        }
        None
    }

    fn row<'b, I>(&mut self, mut row: I, index: usize)
    where
        I: Iterator<Item = &'b str> + Debug,
    {
        let day_index = index / (LESSONS_COUNT + 1);
        let offset = index % (LESSONS_COUNT + 1);
        if offset == 0 {
            return;
        };
        let mut class_number = 0;
        let mut i = 0;
        while class_number < CLASSES_COUNT {
            let Some(lesson_name) = row.next().map(|name| name.trim()) else { return };
            let Some(lesson_class) = row.next().map(|name| name.trim()) else { return };
            i += 1;
            class_number += 1;
            if !(lesson_name.is_empty() && lesson_class.is_empty()) {
                self.lessons[day_index][class_number - 1][offset - 1] = Lesson {
                    name_id: table::id(lesson_name),
                    class_id: table::id(lesson_class),
                };
            };
            if i == 6 {
                for _ in 0..2 {
                    row.next();
                }
                i = 0;
            }
        }
    }
}

#[derive(Default, Copy, Clone, Debug)]
pub struct Lesson {
    pub name_id: u8,
    pub class_id: u8,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse() {
        let content = include_bytes!("../../mock_file.xls");
        let mut lessons = Parser::new();
        unsafe { lessons.reparse(content).unwrap() };
        let lessons = lessons.get_by_class("5а").unwrap();

        println!("lessons: {:?}", lessons);
        assert_eq!(lessons.len(), 999);
    }
}
