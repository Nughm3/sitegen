use super::*;
use std::str::from_utf8;
use fs_extra::{copy_items, dir::CopyOptions};
use glob::glob;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::{
    collections::HashMap,
    env,
    ffi::OsStr,
    fs,
    io::{self, ErrorKind},
    path::{Path, PathBuf},
    process,
    rc::Rc,
    sync::Arc,
};
use walkdir::WalkDir;

pub fn run() -> io::Result<()> {
    // Run a check for a server configuration file here
    // If it's absent, the server should not start
    if !Path::new("server.json").exists() {
        eprintln!("There isn't a project in this directory.");
        process::exit(1);
    }
    let mut f = fs::File::open("server.json")?;
    let config = config::load(&mut f)?;

    let (name, address, port, threads, map) = (
        config.name,
        config.address,
        config.port,
        config.threads,
        config.template_maps,
    );

    println!("sitegen v{} by Nughm3", VERSION);
    println!("Starting server for project {}\n", name);

    // Set up the directory where Markdown will be parsed into HTML
    // This will recursively copy the entire pages directory to `compiled`.
    if let Err(e) = fs::remove_dir_all("compiled") {
        if e.kind() == ErrorKind::PermissionDenied {
            process::exit(1)
        }
    }
    let copy_options = CopyOptions {
        copy_inside: true,
        ..Default::default()
    };
    copy_items(&vec!["pages"], "compiled", &copy_options).expect("Failed to copy files");

    // Find each markdown file and parse it to HTML
    env::set_current_dir("compiled")?;
    let mut failures = 0;
    println!("Parsing Markdown files...");
    let _ = WalkDir::new(".")
        .into_iter()
        .filter_map(|f| f.ok())
        .filter(|f| Path::new(f.path()).extension() == Some(OsStr::new("md")))
        .for_each(|f| {
            if let Ok(()) = markdown::parse(f.path()) {
                println!(
                    " - Successfully parsed {} into HTML",
                    &f.path().as_os_str().to_str().unwrap()[2..]
                );
            } else {
                eprintln!(
                    " - Failed to parse {} into HTML",
                    &f.path().as_os_str().to_str().unwrap()[2..]
                );
                failures += 1;
            }
        });
    let _ = WalkDir::new(".")
        .into_iter()
        .filter_map(|f| f.ok())
        .filter(|f| Path::new(f.path()).extension() == Some(OsStr::new("md")))
        .for_each(|f| fs::remove_file(f.path()).expect("Failed to remove a file"));
    if failures > 0 {
        eprint!(
            "{} files failed to parse into HTML. Start server anyways? [y/N] ",
            failures
        );
        let mut response = String::new();
        std::io::stdin().read_line(&mut response)?;
        if response.to_lowercase() != String::from("y") {
            process::exit(1);
        }
    }

    let map = route(map).expect("Failed to generate route map");
    let map = Arc::new(map);

    // Start the server
    println!("\nStarting server... (CTRL-C to stop)");
    let listener = TcpListener::bind(format!("{}:{}", &address, &port))?;
    let pool = ThreadPool::new(threads.into());
    println!("Bound to {}:{} with {} threads\n", address, port, threads);

    for stream in listener.incoming() {
        let stream = stream?;
        let thread_map = Arc::clone(&map);
        pool.execute(move || handle(stream, thread_map).expect("An issue occurred on a thread"));
    }

    Ok(())
}

fn handle(mut stream: TcpStream, map: Arc<HashMap<String, PathBuf>>) -> io::Result<()> {
    use http::*;

    let mut buffer = [0; 1024];
    stream.read(&mut buffer)?;

    let req: HttpRequest = {
        let buffer = from_utf8(&buffer).expect("Failed to parse to string");
        let method: String = buffer.split_whitespace().nth(0).unwrap().into();
        let method: RequestMethod = method.into();
        let route: String = buffer.split_whitespace().nth(1).unwrap().to_owned();
        let version: String = buffer
            .split_whitespace()
            .nth(2)
            .unwrap()
            .split("\r\n")
            .nth(0)
            .unwrap()
            .to_owned();
        let headers = Some(buffer.split("\r\n").nth(1).unwrap().to_owned());
        let body = Some(buffer.split("\r\n").nth(2).unwrap().to_owned());
        HttpRequest {
            method,
            route,
            version,
            headers,
            body,
        }
    };

    for (route, path) in map.iter() {
        if route.contains(&req.route) {
            let contents = fs::read_to_string(path)?;
            let head = format!("Content-Length: {}", contents.len());
            let response = HttpResponse {
                headers: Some(head.as_str()),
                body: Some(&contents),
                ..Default::default()
            };
            stream.write(response.format().as_bytes())?;
            stream.flush()?;
        }
    }

    Ok(())
}

fn route(
    custom: HashMap<String, PathBuf>,
) -> Result<HashMap<String, PathBuf>, Box<dyn std::error::Error>> {
    let mut map = HashMap::new();
    for entry in glob("**/*.html")? {
        let entry = Rc::new(entry?);
        let route = Rc::clone(&entry)
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap()
            .to_owned();
        let file = Rc::clone(&entry).to_path_buf();
        map.insert(route, file);
    }
    map.insert("/".to_string(), Path::new("index.html").to_path_buf());
    Ok(map.into_iter().chain(custom).collect())
}
