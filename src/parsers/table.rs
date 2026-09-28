use std::{io::{Cursor, Read}, sync::OnceLock};

use parking_lot::{Mutex, RawMutex, lock_api::MutexGuard};

static ID_TABLE: OnceLock<Mutex<Vec<u8>>> = OnceLock::new();

fn table() -> &'static Mutex<Vec<u8>> {
    ID_TABLE.get_or_init(|| Mutex::new(Vec::with_capacity(128)))
}

pub fn id(name: &str) -> u8 {
    let max = name.floor_char_boundary(255);
    let name = name[..max].as_bytes();

    let guard = table().lock();
    let slice = guard.as_slice();
    if slice.is_empty() {
        write(guard, 0, name);
        return 0;
    };
    let id = match find_or_last(slice, name) {
        Ok(id) => return id,
        Err(last_id) => last_id + 1
    };

    write(guard, id, name);
    id
}

fn write(mut guard: MutexGuard<'_, RawMutex, Vec<u8>>, id: u8, name: &[u8]) {
    guard.push(id);
    guard.push(name.len() as u8);
    guard.extend_from_slice(name);
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

fn read_u8(slice: &mut &[u8]) -> u8 {
    let u8 = slice[0];
    *slice = &slice[1..];
    u8
}