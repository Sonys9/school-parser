use std::{mem::forget, ops::Deref, slice::from_raw_parts};

use bytes::Bytes;

use crate::errors::Error;

pub struct HttpClient {
    lessons_change_page: &'static str,
    lessons_page: &'static str,
}

impl HttpClient {
    pub fn new(lessons_change_page: &'static str, lessons_page: &'static str) -> Self {
        Self {
            lessons_change_page,
            lessons_page,
        }
    }

    pub async unsafe fn get_page_document<'b, 'a: 'b>(
        &self,
        page: &'static str,
    ) -> Result<Str<'b, 'a>, Error> {
        let document_bytes = reqwest::get(page).await?.bytes().await?;

        unsafe { Str::new(document_bytes) }
    }
}

#[derive(Debug)]
pub struct Str<'b, 'a: 'b> {
    slice: &'a [u8],
    str: &'b str,
    ptr: *mut u8,
}

impl<'b, 'a: 'b> Str<'a, 'b> {
    pub unsafe fn new(bytes: Bytes) -> Result<Self, Error> {
        let (ptr, len) = (bytes.as_ptr(), bytes.len());
        forget(bytes);

        unsafe {
            let slice = from_raw_parts(ptr, len);
            let str = str::from_utf8(slice)?;
            Ok(Self {
                slice,
                str,
                ptr: ptr as *mut u8,
            })
        }
    }
}

impl<'b, 'a: 'b> Deref for Str<'a, 'b> {
    type Target = &'b str;

    fn deref(&self) -> &Self::Target {
        &self.str
    }
}

impl<'b, 'a> Drop for Str<'a, 'b> {
    fn drop(&mut self) {
        let len = self.slice.len();
        let vec = unsafe { Vec::from_raw_parts(self.ptr, len, len) };
        drop(vec);
    }
}
