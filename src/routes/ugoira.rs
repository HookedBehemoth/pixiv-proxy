use actix_web::{get, web, HttpResponse};

pub fn routes() -> impl actix_web::dev::HttpServiceFactory {
    handle_ugoira
}

#[cfg(feature = "ugoira")]
#[get("/ugoira/{id}")]
pub async fn handle_ugoira(
    client: web::Data<awc::Client>,
    id: web::Path<u64>,
) -> actix_web::Result<HttpResponse> {
    use crate::api::ugoira::{fetch_ugoira_meta, UgoiraFrame};
    use actix_web::{error, web::Buf};
    use std::{
        io::{Cursor, Read, Seek, Write},
        sync::{Arc, Mutex},
    };
    use tokio::sync::oneshot;

    let meta = fetch_ugoira_meta(&client, id.into_inner()).await?;

    let mut ugoira = client
        .get(&meta.original_src)
        .send()
        .await
        .map_err(error::ErrorInternalServerError)?;

    let reader = ugoira.body().limit(0x1000000).await.unwrap().reader();
    let reader = std::io::BufReader::with_capacity(0x4000, reader);
    let reader: Box<dyn Read + Send> = Box::new(reader);

    struct Opaque<'a> {
        reader: Box<dyn Read + Send>,
        file: Option<zip::read::ZipFile<'a>>,
        writer: Cursor<Vec<u8>>,
    }
    unsafe impl Send for Opaque<'_> {}
    unsafe impl Sync for Opaque<'_> {}

    let opaque = Opaque {
        reader,
        file: None,
        writer: Cursor::new(Vec::with_capacity(0x100000)),
    };

    let opaque = Arc::new(Mutex::new(opaque));

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

    let (sender, receiver) = oneshot::channel::<i32>();

    let opaque_clone = opaque.clone();

    std::thread::spawn(move || {
        let ret = {
            let mut opaque = opaque_clone.lock().unwrap();
            let opaque: &mut Opaque = &mut opaque;
            unsafe {
                convert(
                    opaque as *mut Opaque<'_> as *mut libc::c_void,
                    read,
                    next,
                    write,
                    seek,
                    meta.frames.as_ptr(),
                    meta.frames.len(),
                )
            }
        };

        sender.send(ret).unwrap();
    });

    match receiver.await {
        Ok(0) => Ok(HttpResponse::Ok()
            .content_type("video/mp4")
            .insert_header(("cache-control", "max-age=31536000"))
            .body(opaque.lock().unwrap().writer.clone().into_inner())),
        _ => Ok(HttpResponse::InternalServerError().finish()),
    }
}

#[cfg(not(feature = "ugoira"))]
#[get("/ugoira/{id}")]
pub async fn handle_ugoira(_: web::Path<u64>) -> HttpResponse {
    HttpResponse::NotImplemented().finish()
}
