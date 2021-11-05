use super::*;
use color_eyre::eyre::Result;
use fs_extra::{copy_items, dir::CopyOptions};
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::{
    collections::HashMap,
    env,
    ffi::OsStr,
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
    process,
    sync::Arc,
};
use walkdir::WalkDir;

pub fn run() -> Result<()> {
    // Run a check for a server configuration file here
    // If it's absent, the server should not start
    if !Path::new("server.json").exists() {
        eprintln!("There isn't a project in this directory.");
        process::exit(1);
    }
    let mut f = fs::File::open("server.json")?;
    let config = config::load(&mut f)?;

    let (name, address, port, threads) = (config.name, config.address, config.port, config.threads);
    let index_page = if config.index_page != PathBuf::new() {
        Some(config.index_page)
    } else {
        None
    };

    println!("sitegen v{} by Nughm3", VERSION);
    println!("Starting server for project {}\n", name);

    // Set up the directory where Markdown will be parsed into HTML
    // This will recursively copy the entire pages directory to `compiled`.
    if let Err(e) = fs::remove_dir_all("compiled") {
        if e.kind() == ErrorKind::PermissionDenied {
            eprintln!("Could not initialize `compiled` folder");
            process::exit(1)
        }
    }
    let copy_options = CopyOptions {
        copy_inside: true,
        ..Default::default()
    };
    copy_items(&vec!["pages"], "compiled", &copy_options)?;

    // Find each markdown file and parse it to HTML
    env::set_current_dir("compiled")?;
    let mut failures = 0;
    println!("Parsing Markdown files...");
    let files = WalkDir::new(".")
        .into_iter()
        .filter_map(|f| f.ok())
        .filter(|f| Path::new(f.path()).extension() == Some(OsStr::new("md")));
    for file in files {
        if let Ok(()) = markdown::parse(file.path()) {
            println!(
                " * Successfully parsed {} into HTML",
                &file.path().as_os_str().to_str().unwrap()[2..]
            );
            fs::remove_file(file.path())?;
        } else {
            eprintln!(
                " * Failed to parse {} into HTML",
                &file.path().as_os_str().to_str().unwrap()[2..]
            );
            failures += 1;
            fs::remove_file(file.path())?;
        }
    }

    if failures > 0 {
        eprint!(
            "{} failed.\n\n{} files failed to parse into HTML. Start server anyways? [y/N] ",
            failures, failures
        );
        let mut response = String::new();
        std::io::stdin().read_line(&mut response)?;
        if response.to_lowercase() != String::from("y") {
            process::exit(1);
        }
    }

    println!("\nGenerating routes...");
    let map = route(index_page)?;
    let map = Arc::new(map);

    // Start the server
    println!("\nStarting server... (CTRL-C to stop)");
    let listener = TcpListener::bind(format!("{}:{}", &address, &port))?;
    let pool = ThreadPool::new(threads.into());
    println!("Bound to {}:{} with {} threads\n", address, port, threads);

    // Initialize the logging
    match init_logging() {
        Ok(_) => (),
        Err(e) => eprintln!("Failed to init logging: {:?}", e),
    };

    for stream in listener.incoming() {
        let stream = stream?;
        let thread_map = Arc::clone(&map);
        pool.execute(move || handle(stream, thread_map).expect("An issue occurred on a thread"));
    }

    Ok(())
}

fn handle(mut stream: TcpStream, map: Arc<HashMap<String, PathBuf>>) -> Result<()> {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer)?;
    let mut status = "";
    let mut filename: Option<PathBuf> = None;

    for (route, path) in map.iter() {
        let get = format!("GET {} HTTP/1.1\r\n", &route);
        if buffer.starts_with(get.as_bytes()) {
            status = "HTTP/1.1 200 OK";
            filename = Some(path.to_path_buf());
        }
    }

    if filename == None {
        status = "HTTP/1.1 404 NOT FOUND";
        filename = Some(Path::new("../templates/not_found.html").to_path_buf());
    }

    let contents = fs::read_to_string(filename.unwrap())?;
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

fn route(index_page: Option<PathBuf>) -> Result<HashMap<String, PathBuf>> {
    let mut map = HashMap::new();
    if index_page != None {
        map.insert("/".to_owned(), index_page.unwrap());
    } else if Path::new("index.html").exists() {
        map.insert("/".to_owned(), Path::new("index.html").to_path_buf());
        println!("Using index.html for /");
    } else {
        eprintln!("Couldn't find an index.html file (You can define any file to be the homepage in server.json)");
    }
    WalkDir::new(".")
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|f| f.path().extension() == Some(OsStr::new("html")))
        .for_each(|f| {
            let route = f.path().components().skip(1).collect::<PathBuf>();
            let route = format!("/{}", route.to_str().unwrap());
            let route = &route[0..route.len() - 5];
            println!(" * Routed {} to {}", route, f.path().to_str().unwrap());
            map.insert(route.to_owned(), f.path().to_path_buf());
        });
    Ok(map)
}
