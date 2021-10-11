use pulldown_cmark::{html, Options, Parser};
use sitegen::ThreadPool;
use sitegen::projects;
use std::io::prelude::*;
use std::io::ErrorKind::{self, AlreadyExists, PermissionDenied};
use std::net::{TcpListener, TcpStream};
use std::{env, fs, process};

const THREADS: usize = 4;
const ADDRESS: &str = "127.0.0.1:8000";

struct Config(Vec<String>);

impl Config {
    fn new() -> Config {
        let mut c = env::args().collect::<Vec<String>>();
        c.remove(0);
        Config(c)
    }
    fn get(&self, i: usize) -> Option<&str> {
        if let Some(item) = self.0.get(i) {
            Some(item.as_str())
        } else {
            None
        }
    }
}

fn main() {
    let config = Config::new();

    if let Some(cmd) = config.get(0) {
        match cmd {
            "new" => new(true, config.get(1)),
            "init" => new(false, config.get(1)),
            _ => help(),
        }
    } else {
        help()
    }

    // let listener = TcpListener::bind(ADDRESS).unwrap();
    // let pool = ThreadPool::new(THREADS);

    // for stream in listener.incoming() {
    //     let stream = stream.unwrap();
    //     pool.execute(|| process(stream));
    // }
}

fn process(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).expect("Failed to read to buffer");

    let get = b"GET / HTTP/1.1\r\n";

    let (status, filename) = if buffer.starts_with(get) {
        ("HTTP/1.1 200 OK", "index.html")
    } else {
        ("HTTP/1.1 404 NOT FOUND", "404.html")
    };

    let contents = fs::read_to_string(filename).expect("Failed to read file");

    let response = format!(
        "{}\r\nContent-Length: {}\r\n\r\n{}",
        status,
        contents.len(),
        contents
    );

    stream
        .write(response.as_bytes())
        .expect("Failed to write stream");
    stream.flush().expect("Failed to flush stream");
}

fn new(dir_exists: bool, name: Option<&str>) {
    if let Some(n) = name {
        if !dir_exists {
            if let Err(e) = fs::create_dir_all(format!("{}/index", n)) {
                match e.kind() {
                    AlreadyExists => println!("The directory already exists!"),
                    PermissionDenied => {
                        println!("You don't have permission to make a website in this folder!")
                    }
                    _ => {
                        println!("An unexpected error occured while trying to create the website.")
                    }
                };
                process::exit(1);
            }
        }
    } else {
        println!("Please provide a name for the new website!");
        process::exit(1);
    }
}

fn help() {
    unimplemented!("WIP");
}
