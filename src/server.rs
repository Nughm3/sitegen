use super::*;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::{
    ffi::OsStr,
    fs,
    io::{self, ErrorKind},
    path::Path,
    process,
};
use walkdir::WalkDir;

pub fn run(addr: &str, threads: usize) -> io::Result<()> {
    // Run a check for a server configuration file here
    // If it's absent, the server should not start
    if let Err(_) = fs::File::open("./server.json") {
        eprintln!("There isn't a project in this directory.");
        process::exit(1);
    }

    // Set up the directory where Markdown will be parsed into HTML
    if let Err(e) = fs::remove_dir_all("./compiled") {
        match e.kind() {
            ErrorKind::PermissionDenied => process::exit(1),
            _ => fs::create_dir("./compiled")?,
        }
    } else {
        fs::create_dir("./compiled")?;
    }

    // Find each markdown file and parse it to HTML
    let mut failures = 0;
    let _ = WalkDir::new(".")
        .into_iter()
        .filter_map(|f| f.ok())
        .filter(|f| Path::new(f.path()).extension() == Some(OsStr::new("md")))
        .for_each(|f| {
            if let Ok(()) = markdown::parse(f.path()) {
                println!("Successfully parsed {:?} into HTML", f.path());
            } else {
                eprintln!("Failed to parse {:?} into HTML", f.path());
                failures += 1;
            }
        });
    if failures > 0 {
        eprintln!(
            "{} files failed to parse into HTML. Start server anyways? [y/N]",
            failures
        );
        let mut response = String::new();
        std::io::stdin().read_line(&mut response)?;
        if response.to_lowercase() != String::from("y") {
            process::exit(1);
        }
    }

    // Start the server
    println!("Starting server...");
    let listener = TcpListener::bind(&addr)?;
    let pool = ThreadPool::new(threads);
    println!("Bound to {} with {} threads", addr, threads);

    for stream in listener.incoming() {
        let stream = stream?;
        pool.execute(|| handle(stream).expect("An issue occurred on a thread"));
    }

    Ok(())
}

fn handle(mut stream: TcpStream) -> io::Result<()> {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer)?;

    let get = b"GET / HTTP/1.1\r\n";

    let (status, filename) = if buffer.starts_with(get) {
        ("HTTP/1.1 200 OK", "index.html")
    } else {
        ("HTTP/1.1 404 NOT FOUND", "404.html")
    };

    let contents = fs::read_to_string(filename)?;

    let response = format!(
        "{}\r\nContent-Length: {}\r\n\r\n{}",
        status,
        contents.len(),
        contents
    );

    stream.write(response.as_bytes())?;
    stream.flush()?;

    Ok(())
}
