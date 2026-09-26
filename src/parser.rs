use std::{
    borrow::Cow,
    hash::BuildHasherDefault,
    io::{Cursor, Write},
    mem::transmute,
    sync::Arc,
};

use bytes::Buf;
use tl::{
    HTMLTag, Node, NodeHandle, ParserOptions, VDom,
    queryselector::{QuerySelectorIterator, iterable::QueryIterable},
};
use tracing::info;

macro_rules! elements {
    ($self:ident, $src:expr, $selector:expr) => {
        $self.elements_by_query($src.query_selector($self.dom.parser(), $selector))
    };

    ($self:ident, $src:expr, $selector:expr, global) => {
        $self.elements_by_query($src.query_selector($selector))
    };
}

pub struct Parser<'a> {
    dom: VDom<'a>,
}

impl<'a> Parser<'a> {
    pub fn new(document: &'a str) -> Self {
        let dom = tl::parse(&document, ParserOptions::default()).unwrap();
        Self { dom }
    }

    pub fn changes<'h>(&self) -> Vec<Change<'_>> {
        let mut changes = Vec::new();
        for table in elements!(self, self.dom, "table", global).skip(1) {
            let (mut colgroup, mut tbody) = (None, None);
            for child in table
                .children()
                .all(self.dom.parser())
                .iter()
                .map(|child| child.as_tag())
                .flatten()
            {
                match child.name().as_bytes() {
                    b"colgroup" => colgroup = Some(child),
                    b"tbody" => tbody = Some(child),
                    _ => {}
                };
            }
            if colgroup.is_none() || tbody.is_none() {
                continue;
            };
            let columns = self.columns(colgroup.unwrap());
            changes.extend(self.table(tbody.unwrap(), columns).drain(..));
        }
        changes
    }

    fn table<'b>(&'a self, tbody: &'b HTMLTag<'a>, columns: usize) -> Vec<Change<'a>> {
        let mut stops: Vec<(usize, usize, Cow<'a, str>)> = Vec::with_capacity(16);
        let mut i = 0;
        'tr_loop: for tr in elements!(self, tbody, "tr") {
            i += 1;
            for td in elements!(self, tr, "td") {
                let Some(Some(span)) = self
                    .find_child(td, "span")
                    .map(|element| self.find_child(element, "span"))
                else {
                    continue 'tr_loop;
                };
                let table_type = match span
                    .attributes()
                    .get("style")
                    .flatten()
                    .map(|bytes| str::from_utf8(bytes.as_bytes()))
                {
                    Some(Ok("color:red")) => "Changes",
                    Some(Ok("font-family:Cambria,serif")) => "Olympiad",
                    _ => continue 'tr_loop,
                };
                if let Some(&mut (_, ref mut last_i, _)) = stops.last_mut() {
                    *last_i = i;
                };
                stops.push((i, 999, self.title(tr)));
            }
        }

        let mut elements = elements!(self, tbody, "tr");
        let mut changes: Vec<Change> = Vec::with_capacity(stops.len());

        let mut last_row: usize = 0;
        for (i, (start, end, title)) in stops.into_iter().enumerate() {
            changes.push(Change::new(columns, title));
            self.tbody(
                columns,
                &mut elements,
                &mut changes[i],
                end,
                last_row,
                start + 1,
            );
            last_row = end;
        }

        changes
    }

    fn tbody<'b>(
        &'a self,
        columns: usize,
        elements: &mut impl Iterator<Item = &'a HTMLTag<'a>>,
        change: &'b mut Change<'a>,
        stop_at: usize,
        mut i: usize,
        start_at: usize,
    ) {
        for row in elements {
            let mut temp_change = Vec::with_capacity(columns);
            i += 1;
            if i < start_at {
                continue;
            };
            if i == stop_at {
                break;
            };
            let mut is_empty = true;
            for td in elements!(self, row, "td") {
                let text = td.inner_text(self.dom.parser());
                is_empty = !(text != "&nbsp;" || !is_empty);
                temp_change.push(text);
            }
            if !is_empty {
                change.change_data.push(temp_change);
            }
        }
    }

    fn columns(&self, colgroup: &'a HTMLTag<'a>) -> usize {
        elements!(self, colgroup, "col")
            .map(|column| {
                column
                    .attributes()
                    .get("span")
                    .flatten()
                    .and_then(|unparsed| str::from_utf8(unparsed.as_bytes()).ok()?.parse().ok())
                    .unwrap_or(1)
            })
            .sum()
    }

    // fn elements(&self, src: &HTMLTag<'a>, selector: &str) -> impl Iterator<Item = &HTMLTag<'a>> {
    //     self.elements_by_query(src.query_selector(self.dom.parser(), selector))
    // }
    // 😡😡😡😡😡😡😡😡😡😡😡

    fn elements_by_query<I>(&self, query: Option<I>) -> impl Iterator<Item = &HTMLTag<'a>>
    where
        I: IntoIterator<Item = NodeHandle>,
    {
        query
            .into_iter()
            .flatten()
            .filter_map(|node| node.get(self.dom.parser()))
            .filter_map(|node| node.as_tag())
    }

    fn find_child<'b>(&'a self, element: &'b HTMLTag<'a>, name: &str) -> Option<&'b HTMLTag<'a>> {
        element
            .children()
            .all(self.dom.parser())
            .iter()
            .filter_map(|node| node.as_tag())
            .find(|element| element.name() == name)
    }

    fn title(&self, tr: &'a HTMLTag<'a>) -> Cow<'a, str> {
        let mut result = String::with_capacity(tr.inner_text(self.dom.parser()).len());
        for td in elements!(self, tr, "td") {
            let text = td.inner_text(self.dom.parser());
            let mut chars = text.char_indices();
            while let Some((i, char)) = chars.next() {
                match char {
                    '\n' | '\t' => {}
                    '&' if text.get(i..i + 6) == Some("&nbsp;") => {
                        result.push(' ');
                        for _ in 0..5 {
                            chars.next();
                        }
                    }
                    _ => {
                        result.push(char);
                    }
                };
            }
        }

        result.truncate(result.trim_end().len());
        Cow::Owned(result)
    }
}

/*
Расписание:
Считаем количество колонок в <colgroup>
Если есть span то прибавляем значение span
Читаем ровно столько, сколько получилось
Структура примерного Vec:
    [
        "Класс", "Номер урока", ...,
        "10а", "5", "Вфи", "307", None, "нет 7 ур."
    ]
И сохраняем как (column_len, [...]) или в структуре

Заголовок читаем так: читаем САМЫЙ ПЕРВЫЙ В ТАБЛИЦЕ tr как inner_text(), удаляем весь мусор, заменяя &nbsp; пробелами, а ну или как список ссылок можно еще например:
["ИЗМЕНЕНИЯ РАСПИСАНИЯ:", " ", "среда", "", "дата"]
2026-09-23T03:37:07.635963Z  INFO schoolbot::parser: "ИЗМЕНЕНИЯ РАСПИСАНИЯ:"
2026-09-23T03:37:07.635968Z  INFO schoolbot::parser: "&nbsp;"
2026-09-23T03:37:07.635973Z  INFO schoolbot::parser: "среда&nbsp;"
2026-09-23T03:37:07.635978Z  INFO schoolbot::parser: "23 сентября 2026 г."

2026-09-23T03:37:07.635983Z  INFO schoolbot::parser: "Класс"
2026-09-23T03:37:07.635988Z  INFO schoolbot::parser: "Номер урока"
2026-09-23T03:37:07.635993Z  INFO schoolbot::parser: "Предмет"
2026-09-23T03:37:07.635998Z  INFO schoolbot::parser: "Каб."
2026-09-23T03:37:07.636003Z  INFO schoolbot::parser: "Заменяемый предмет"
2026-09-23T03:37:07.636008Z  INFO schoolbot::parser: "Комментарии"
2026-09-23T03:37:07.636014Z  INFO schoolbot::parser: "10а"
2026-09-23T03:37:07.636018Z  INFO schoolbot::parser: "5"
2026-09-23T03:37:07.636023Z  INFO schoolbot::parser: "Вфи"
2026-09-23T03:37:07.636028Z  INFO schoolbot::parser: "307"
2026-09-23T03:37:07.636032Z  INFO schoolbot::parser: "&nbsp;"
2026-09-23T03:37:07.636037Z  INFO schoolbot::parser: "нет 7 ур."

*/

#[derive(Debug)]
pub struct Change<'a> {
    //pub change_data: FxHashMap<Cow<'b, str>, Vec<Cow<'c, str>>>,
    pub column_len: usize,
    pub change_data: Vec<Vec<Cow<'a, str>>>,
    pub title: Cow<'a, str>,
}

impl<'a> Change<'a> {
    pub fn new(column_len: usize, title: Cow<'a, str>) -> Self {
        Self {
            column_len,
            change_data: Vec::with_capacity(24),
            title,
        }
    }
}
