#![deny(warnings)]
extern crate futures;
extern crate tokio_fs;
extern crate tokio_io;

use hyper::rt::Future;
use std::io;
use std::thread::{sleep_ms, spawn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use hyper::service::service_fn;
use self::futures::future;

static NOTFOUND: &[u8] = b"Not Found";
const INDEX_FILE: &str = "client/ui/index.html";
const DIST_FILE: &str = "client/ui/dist/bundle.js";
const DIST_MAP_FILE: &str = "client/ui/dist/bundle.js.map";

type ResponseFuture = Box<Future<Item = Response<Body>, Error = io::Error> + Send>;

fn simple_file_send(f: &str) -> ResponseFuture {
    // Serve a file by asynchronously reading it entirely into memory.
    // Uses tokio_fs to open file asynchronously, then tokio_io to read into
    // memory asynchronously.
    let filename = f.to_string(); // we need to copy for lifetime issues
    Box::new(
        tokio_fs::file::File::open(filename)
            .and_then(|file| {
                let buf: Vec<u8> = Vec::new();
                tokio_io::io::read_to_end(file, buf)
                    .and_then(|item| Ok(Response::new(item.1.into())))
                    .or_else(|_| {
                        Ok(Response::builder()
                            .status(StatusCode::INTERNAL_SERVER_ERROR)
                            .body(Body::empty())
                            .unwrap())
                    })
            })
            .or_else(|_| {
                Ok(Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(NOTFOUND.into())
                    .unwrap())
            }),
    )
}

fn handler(req: Request<Body>) -> ResponseFuture {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") | (&Method::GET, "/index.html") => {
            simple_file_send(INDEX_FILE)
        }
        (&Method::GET, "/bundle.js") => {
            simple_file_send(DIST_FILE)
        }
        (&Method::GET, "/bundle.js.map") => {
            simple_file_send(DIST_MAP_FILE)
        }
        _ => Box::new(future::ok(
            Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::empty())
                .unwrap(),
        )),
    }
}
pub fn serve() -> String {
    // This is our socket address...
    let addr = ([127, 0, 0, 1], 0).into();

    let server = Server::bind(&addr).serve(|| {
        service_fn(handler)
    });
    let url = format!("http://{}", server.local_addr());
    let server = server.map_err(|e| eprintln!("server error: {}", e));

    spawn(move || {
        ::hyper::rt::run(server);
    });
    while !server_is_up(&url) {
        sleep_ms(100);
    }
    url
}

fn server_is_up(url: &str) -> bool {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    let client = ::hyper::Client::new();
    let is_ok: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    let is_ok_clone = is_ok.clone();

    let work = client
        .get(url.parse().unwrap())
        .and_then(move |res| {
            println!("Response: {}", res.status());
            is_ok_clone.store(true, Ordering::Relaxed);
            Ok(())
        })
        .map_err(|e| eprintln!("server error: {}", e));
    ::hyper::rt::run(work);
    is_ok.load(Ordering::Relaxed)
}
