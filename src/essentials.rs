use core::ffi::{c_char, c_void};

#[unsafe(no_mangle)]
pub unsafe extern "C" fn memset(origin_ptr: *mut c_void, fill: i32, len: usize) -> *mut c_void {
    let fill = fill as u8;
    let mut ptr = origin_ptr as *mut u8;
    for _ in 0..len {
        unsafe { *ptr = fill };
        ptr = unsafe { ptr.add(1) };
    };
    origin_ptr
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn strlen(str: *const c_char) -> usize {
    let mut size = 0;
    while unsafe { *str.add(size) } != 0 {
        size += 1;
    };
    size
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bcmp(first: *const c_void, second: *const c_void, len: usize) -> i32 {
    let first = first as *const u8;
    let second = second as *const u8;
    
    for i in 0..len {
        let (first_byte, second_byte) = unsafe { (*first.add(i), *second.add(i)) };
        if first_byte != second_byte {
            return first_byte as i32 - second_byte as i32;
        };
    };
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn memmove(dst: *mut c_void, src: *const c_void, len: usize) -> *mut c_void {
    let dst_mut = dst as *mut u8;
    let src = src as *mut u8;
    let dst = dst as *mut c_void;

    if dst_mut == src || len == 0 {
        return dst;
    };

    if dst_mut > src && dst_mut < unsafe {src.add(len) } {
        let mut i = len;
        while i > 0 {
            i -= 1;
            unsafe { *dst_mut.add(i) = *src.add(i) };
        };
        return dst;
    };

    for i in 0..len {
        unsafe { *dst_mut.add(i) = *src.add(i) };
    };
    dst
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn memcpy(dst: *mut c_void, src: *const c_void, len: usize) -> *mut c_void {
    let src = src as *mut u8;
    let dst = dst as *mut u8;
    
    for i in 0..len {
        unsafe { *dst.add(i) = *src.add(i) };
    };

    dst as *mut c_void
}