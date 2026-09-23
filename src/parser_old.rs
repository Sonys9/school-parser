use std::{borrow::Cow, mem::transmute};

use tl::ParserOptions;

pub unsafe fn parse_changes<'h>(document: &'h str) -> Vec<(Change<'h, 'h, 'h, 'h, 'h, 'h>, usize)> {
    let dom = tl::parse(&document, ParserOptions::default()).unwrap();
    
    let mut changes: Vec<(Change, usize)> = Vec::new();
    for node in dom.nodes() {
        let Some(element) = node.as_tag() else {
            continue;
        };
        let attributes = element.attributes();
        let (Some(Some(class)), Some(Some(style))) = (attributes.get("class"), attributes.get("style")) else {
            continue
        };
        if style.as_utf8_str().contains("width:") || !(class == "xl68" || class == "xl70") {
            continue;
        };
        let raw_text = match element.inner_text(dom.parser()) {
            Cow::Borrowed("&nbsp;") => None,
            text if text.trim().is_empty() => None,
            text => Some(text)
        };
        let text = unsafe { transmute(raw_text) };
        match class.as_utf8_str() {
            Cow::Borrowed("xl70") => {
                let Some(class_name) = text else {
                    continue;
                };
                changes.push((Change::new(class_name), 0))
            },
            Cow::Borrowed("xl68") => {
                let Some((change, index)) = changes.last_mut() else {
                    continue;
                };
                match index {
                    0 => change.lesson_id = text,
                    1 => change.lesson = text,
                    2 => change.class = text,
                    3 => change.changed_lesson = text,
                    4 => change.comment = text,
                    _ => {}
                };
                *index += 1;
            },
            _ => {},
        };
    }

    changes
}

#[derive(Debug)]
pub struct Change<'a, 'b, 'c, 'd, 'e, 'f> {
    pub class_name: Cow<'e, str>,
    pub lesson_id: Option<Cow<'f, str>>,
    pub lesson: Option<Cow<'a, str>>,
    pub class: Option<Cow<'b, str>>,
    pub changed_lesson: Option<Cow<'c, str>>,
    pub comment: Option<Cow<'d, str>>,
}

impl<'a, 'b, 'c, 'd, 'e, 'f> Change<'a, 'b, 'c, 'd, 'e, 'f> {
    pub fn new(class_name: Cow<'e, str>) -> Self {
        Self { 
            class_name,
            lesson_id: None,
            lesson: None,
            class: None,
            changed_lesson: None,
            comment: None
        }
    }
}