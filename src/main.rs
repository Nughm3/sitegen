use pulldown_cmark::{html, Options, Parser};
use sitegen::ThreadPool;
use std::io::prelude::*;
use std::io::ErrorKind::{AlreadyExists, PermissionDenied};
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
            "new" => new(config.get(1)),
            "init" => {
                if let Ok(()) = init() {
                    println!("Successfully created new project");
                } else {
                    eprintln!("A problem occured in the creation of the new project");
                }
            }
            "run" => {
                if let Err(_) = fs::File::open("./server.json") {
                    eprintln!("There isn't a project in this directory.");
                    process::exit(1);
                }
                let listener = TcpListener::bind(ADDRESS).unwrap();
                let pool = ThreadPool::new(THREADS);

                for stream in listener.incoming() {
                    let stream = stream.unwrap();
                    pool.execute(|| handle(stream));
                }
            }
            _ => help(),
        }
    } else {
        help()
    }
}

fn handle(mut stream: TcpStream) {
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

fn new(name: Option<&str>) {
    if let Some(n) = name {
        if let Err(e) = fs::create_dir(n) {
            match e.kind() {
                AlreadyExists => eprintln!("The directory already exists!"),
                PermissionDenied => {
                    eprintln!("You don't have permission to make a website in this folder!")
                }
                _ => {
                    eprintln!("An unexpected error occured while trying to create the website.")
                }
            };
            process::exit(1);
        } else {
            env::set_current_dir(n).expect("Failed to change to the newly created project");
            if let Ok(()) = init() {
                println!("Successfully created new project");
            } else {
                eprintln!("A problem occured in the creation of the new project.\n(Does a project already exist at that location?)");
                process::exit(1);
            }
        }
    } else {
        eprintln!("Please provide a name for the new website!");
        process::exit(1);
    }
}

fn init() -> Result<(), Box<dyn std::error::Error>> {
    fs::File::create("./server.json")?; // Server configuration file

    fs::create_dir("./templates")?; // Used to store the templates which each page will inherit from
    fs::File::create("templates/base.html")?; // Base template
    fs::File::create("templates/base.css")?;
    fs::File::create("templates/not_found.html")?; // 404 Not Found template
    fs::File::create("templates/not_found.css")?;

    fs::create_dir("./index")?; // A default webpage used as the entry point
    fs::File::create("index/index.md")?;

    Ok(())
}

fn help() {
    println!("sitegen - Markdown based static site generator\n");
}
