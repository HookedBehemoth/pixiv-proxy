use crate::api::error::ApiError;

#[cfg(feature = "ugoira")]
pub fn ugoira(client: &ureq::Agent, id: u64) -> Result<rouille::Response, ApiError> {
    use crate::api::ugoira::{fetch_ugoira_meta, UgoiraFrame};
    use std::{
        io::BufReader,
        io::{Cursor, Read, Seek, Write},
    };

    let meta = fetch_ugoira_meta(&client, id)?;

    let ugoira = client.get(&meta.original_src).call()?;
    let body = ugoira.into_body();

    let reader: Box<dyn Read + Send> = Box::new(body.into_reader());
    let reader = BufReader::with_capacity(0x4000, reader);

    struct Opaque<'a> {
        reader: BufReader<Box<dyn Read + Send>>,
        file: Option<zip::read::ZipFile<'a, BufReader<Box<dyn Read + Send>>>>,
        writer: Cursor<Vec<u8>>,
    }
    let mut opaque = Opaque {
        reader,
        file: None,
        writer: Cursor::new(Vec::with_capacity(0x100000)),
    };

    unsafe extern "C" fn read(opaque: *mut libc::c_void, ptr: *mut u8, sz: i32) -> i32 {
        let opaque = opaque as *mut Opaque<'_>;
        let slice = std::slice::from_raw_parts_mut(ptr, sz as usize);
        let file = (*opaque).file.as_mut().unwrap();
        file.read(slice).unwrap() as i32
    }
    unsafe extern "C" fn next(opaque: *mut libc::c_void) {
        let opaque = opaque as *mut Opaque<'_>;
        let reader = &mut (*opaque).reader;
        (*opaque).file = Some(
            zip::read::read_zipfile_from_stream(reader)
                .unwrap()
                .unwrap(),
        );
    }
    unsafe extern "C" fn write(opaque: *mut libc::c_void, ptr: *mut u8, sz: i32) -> i32 {
        let opaque = opaque as *mut Opaque<'_>;
        let slice = std::slice::from_raw_parts(ptr, sz as usize);
        (*opaque).writer.write_all(slice).unwrap();
        sz
    }
    unsafe extern "C" fn seek(opaque: *mut libc::c_void, offset: i64, whence: i32) -> i64 {
        let opaque = opaque as *mut Opaque<'_>;
        let position = match whence {
            0 => std::io::SeekFrom::Start(offset as u64),
            1 => std::io::SeekFrom::Current(offset),
            2 => std::io::SeekFrom::End(offset),
            _ => panic!("invalid whence"),
        };
        (*opaque).writer.seek(position).unwrap() as i64
    }

    extern "C" {
        fn convert(
            opaque: *mut libc::c_void,
            read: unsafe extern "C" fn(*mut libc::c_void, *mut u8, i32) -> i32,
            next: unsafe extern "C" fn(*mut libc::c_void),
            write: unsafe extern "C" fn(*mut libc::c_void, *mut u8, i32) -> i32,
            seek: unsafe extern "C" fn(*mut libc::c_void, i64, i32) -> i64,
            frames: *const UgoiraFrame,
            frame_count: usize,
        ) -> i32;
    }

    let ret = unsafe {
        convert(
            &mut opaque as *mut Opaque<'_> as *mut libc::c_void,
            read,
            next,
            write,
            seek,
            meta.frames.as_ptr(),
            meta.frames.len(),
        )
    };

    if ret != 0 {
        Err(ApiError::Internal("Failed to re-encode image".into()))
    } else {
        Ok(
            rouille::Response::from_data("video/mp4", opaque.writer.into_inner())
                .with_public_cache(365 * 24 * 60 * 60),
        )
    }
}

#[cfg(not(feature = "ugoira"))]
pub fn ugoira(_: &ureq::Agent, _: u64) -> Result<rouille::Response, ApiError> {
    Err(ApiError::External(418, "Feature not enabled".into()))
}
