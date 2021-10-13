use sitegen::server;
use std::io::ErrorKind::{AlreadyExists, PermissionDenied};
use std::{env, fs, io, path::Path, process};

const THREADS: usize = 4;
const ADDRESS: &str = "127.0.0.1:8000";

struct Args(Vec<String>);

impl Args {
    fn new() -> Args {
        let mut c = env::args().collect::<Vec<String>>();
        c.remove(0);
        Args(c)
    }
    fn get(&self, i: usize) -> Option<&str> {
        if let Some(item) = self.0.get(i) {
            Some(item.as_str())
        } else {
            None
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::new();

    if let Some(cmd) = args.get(0) {
        match cmd {
            "new" => new(args.get(1)),
            "init" => {
                if let Ok(()) = init() {
                    println!("Successfully created new project");
                } else {
                    eprintln!("A problem occured in the creation of the new project");
                }
            }
            "run" => server::run(ADDRESS, THREADS)?,
            _ => help(),
        }
    } else {
        help()
    }

    Ok(())
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

fn init() -> io::Result<()> {
    type P = Path;
    fs::File::create("server.json")?; // Server configuration file
    fs::File::create("log.txt")?; // Server log file

    fs::create_dir("templates")?; // Used to store the templates which each page will inherit from
    fs::File::create(P::new("templates/base.html"))?; // Base template
    fs::File::create(P::new("templates/base.css"))?;
    fs::File::create(P::new("templates/not_found.html"))?; // 404 Not Found template
    fs::File::create(P::new("templates/not_found.css"))?;

    fs::create_dir_all(P::new("pages/index"))?; // A default webpage used as the entry point
    fs::File::create(P::new("pages/index/index.md"))?;

    Ok(())
}

fn help() {
    println!("sitegen - Markdown based static site generator\n");
}
