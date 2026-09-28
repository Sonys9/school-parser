use std::{io::{Cursor, Read}, mem::transmute, sync::OnceLock};

use parking_lot::{Mutex, RawMutex, lock_api::MutexGuard};

const MAX_STR_LEN: usize = 64;
pub const ARRAY_LEN: usize = ((u8::MAX as usize + 1) * MAX_STR_LEN + (u8::MAX as usize + 1)) / 4; // for honesty

static ID_TABLE: OnceLock<Mutex<Table>> = OnceLock::new();

pub struct Table {
    pub data: [u8; ARRAY_LEN],
    pub len: usize,
}

pub fn table() -> &'static Mutex<Table> {
    ID_TABLE.get_or_init(|| Mutex::new(Table { data: [0u8; ARRAY_LEN], len: 0 }))
}

pub fn id(name: &str) -> u8 {
    let max = name.floor_char_boundary(MAX_STR_LEN);
    let name = name[..max].as_bytes();

    let mut guard = table().lock();
    if guard.len == 0 {
        write(&mut guard, 0, name);
        return 0;
    };
    let slice = &guard.data[..guard.len];
    let id = match find_or_last(slice, name) {
        Ok(id) => return id,
        Err(last_id) => last_id + 1
    };

    write(&mut guard, id, name);
    id
}

pub fn name<'a>(id: u8) -> Option<&'a str> {
    let guard = table().lock();
    let mut slice = &guard.data[..guard.len];
    loop {
        let value_id = read_u8(&mut slice);
        let len = read_u8(&mut slice) as usize;
        let (name, rslice) = slice.split_at(len);
        slice = rslice;
        if value_id == id {
            let static_name: &'static [u8] = unsafe { transmute(name) }; // always safe because WE NEVER REMOVE ANYTHING and it never moves because 0 reallocations
            return Some(unsafe { str::from_utf8_unchecked(static_name) }) // always safe because we write utf8 bytes
        }
        if slice.is_empty() {
            return None;
        };
    };
}

fn write(guard: &mut MutexGuard<'_, RawMutex, Table>, id: u8, name: &[u8]) {
    let len = guard.len;
    guard.data[len as usize] = id;
    guard.data[len + 1] = name.len() as u8;
    guard.data[len + 2..len + 2 + name.len()].copy_from_slice(name);
    guard.len += 2 + name.len();
}

fn find_or_last(mut slice: &[u8], queue: &[u8]) -> Result<u8, u8> {
    loop {
        let id = read_u8(&mut slice);
        let len = read_u8(&mut slice) as usize;
        let (name, rslice) = slice.split_at(len);
        slice = rslice;
        if name == queue {
            return Ok(id);
        };
        if slice.is_empty() {
            return Err(id);
        };
    };
}

pub fn read_u8(slice: &mut &[u8]) -> u8 {
    let u8 = slice[0];
    *slice = &slice[1..];
    u8
}