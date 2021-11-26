use color_eyre::eyre::Result;
use sitegen::*;
use std::io::ErrorKind::{AlreadyExists, PermissionDenied};
use std::{
    env,
    fs::{self, File},
    io,
    path::Path,
    process,
};

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

#[derive(Debug)]
enum OpType {
    Add,
    Remove,
    Edit,
}

fn main() -> Result<()> {
    use OpType::*;
    setup();
    let args = Args::new();

    if let Some(cmd) = args.get(0) {
        let arg_1 = args.get(1);
        match cmd {
            "new" => new(arg_1),
            "run" => {
                if let Err(e) = server::run() {
                    eprintln!("Failed to run server:\n{:?}", e);
                    process::exit(1);
                }
            }
            "add" => op(Add, arg_1)?,
            "rm" => op(Remove, arg_1)?,
            "edit" => op(Edit, arg_1)?,
            "config" => config()?,
            "init" => {
                if let Ok(()) = init("".to_owned()) {
                    println!("Successfully created new project");
                } else {
                    eprintln!("A problem occured in the creation of the new project");
                }
            }
            _ => help(true),
        }
    } else {
        help(false);
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
            if let Ok(()) = init(n.to_owned()) {
                println!("Successfully created new project");
            } else {
                eprintln!("A problem occured in the creation of the new project.");
                process::exit(1);
            }
        }
    } else {
        eprintln!("Please provide a name for the new website!");
        eprint!("Website name >> ");
        let mut name = String::new();
        io::stdin()
            .read_line(&mut name)
            .expect("Failed to read line");
        new(Some(&name.trim()));
    }
}

fn init(name: String) -> Result<()> {
    type P = Path;
    File::create("server.json")?; // Server configuration file
    File::create("log.txt")?; // Server log file

    fs::create_dir("templates")?; // Used to store the templates which each page will inherit from
    File::create(P::new("templates/base.html"))?; // Base template
    File::create(P::new("templates/base.css"))?;
    File::create(P::new("templates/not_found.html"))?; // 404 Not Found template
    File::create(P::new("templates/not_found.css"))?;

    fs::create_dir("pages")?; // Used to store the user's pages
    File::create(P::new("pages/index.md"))?;

    if name != String::from("") {
        config::configure(name)?;
    } else {
        let name = env::current_dir()?;
        let name = name
            .components()
            .last()
            .unwrap()
            .as_os_str()
            .to_str()
            .to_owned()
            .unwrap();
        config::configure(name.to_owned())?;
    }

    Ok(())
}

// op for 'operation'
fn op(action: OpType, name: Option<&str>) -> Result<()> {
    if let Some(n) = name {
        match action {
            OpType::Add => {
                env::set_current_dir("pages")?;
                fs::create_dir(&n)?;
                File::create(format!("{n}/{n}.md", n = n))?;
                println!("Created page {}", n);
            }
            OpType::Remove => {
                env::set_current_dir("pages")?;
                if Path::new(n).exists() {
                    eprint!("This will remove the page {}, continue? [Y/n] ", n);
                    let mut action = String::new();
                    io::stdin().read_line(&mut action)?;
                    if action.to_lowercase() == String::from("n") {
                        println!("Cancelled!");
                    } else {
                        fs::remove_dir_all(n)?;
                    }
                } else {
                    eprintln!("Could not find the page you were looking for!");
                }
            }
            OpType::Edit => {
                env::set_current_dir("pages")?;
                if Path::new(n).exists() {
                    if let Ok(e) = env::var("EDITOR") {
                        process::Command::new(e).arg(format!(" {}", n)).spawn()?;
                    } else {
                        eprintln!("Your EDITOR environment variable isn't set!");
                    }
                } else {
                    eprintln!("Your current working directory does not have a website in it!");
                }
            }
        }
    } else {
        eprintln!(
            "Please provide the name of the page you'd like to {}!",
            format!("{:?}", &action).to_lowercase()
        );
        eprint!("Page name >> ");
        let mut name = String::new();
        io::stdin().read_line(&mut name)?;
        op(action, Some(&name.trim()))?;
    }
    Ok(())
}

fn config() -> Result<()> {
    if Path::new("server.json").exists() {
        if let Ok(e) = env::var("EDITOR") {
            process::Command::new(e).arg(" server.json").spawn()?;
        } else {
            eprintln!("Your EDITOR environment variable isn't set! You can still manually edit `server.json`.");
        }
    } else {
        eprintln!("Your current working directory does not have a website in it!");
    }
    Ok(())
}

fn help(unrecognized: bool) {
    if unrecognized {
        eprintln!("Unrecognized command. Use the following commands:")
    } else {
        eprintln!(
            "sitegen v{} - Markdown based static site generator",
            VERSION
        );
    }
    eprintln!("  new <name>  -> Create a new site with a name");
    eprintln!("  init        -> Initialize a site in the current working directory");
    eprintln!("  run         -> Compiles all Markdown files to HTML and starts the server");
    eprintln!("  add <name>  -> Add a page");
    eprintln!("  rm <name>   -> Remove a page");
    eprintln!("  edit <name> -> Edit a page's file (Requires EDITOR)");
    eprintln!("  config      -> Edit your project's configuration file (Requires EDITOR)");
}

fn setup() {
    if let Err(_) = env::var("RUST_BACKTRACE") {
        env::set_var("RUST_BACKTRACE", "1");
    }
}
