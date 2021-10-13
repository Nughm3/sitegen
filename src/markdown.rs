use pulldown_cmark::{html, Options, Parser};
use std::{env, fs, io::{self, Read}, path::Path, process};

pub fn parse(file: &Path) -> io::Result<()> {
    // Enable all modern Markdown features
    let mut options = Options::empty();
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_SMART_PUNCTUATION);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_TASKLISTS);

    // Open a template and apply it
    let mut template = String::new();
    if let Ok(mut f) = fs::File::open("../templates/base.html") {
        f.read_to_string(&mut template)?;
    } else {
        eprintln!("Was unable to locate template `base.html` file, creating it...");
        env::set_current_dir("..")?;
        super::create_templates()?;
        env::set_current_dir("compiled")?;
        parse(file)?;
    }

    let contents = fs::read_to_string(&file)?;
    let parser = Parser::new_ext(&contents, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    // Write the template with the Markdown into the output file
    let mut output = String::new();
    let mut count = 1;
    for line in template.lines() {
        if line.contains("{{ body }}") {
            break;
        }
        count += 1;
    }
    if count == template.lines().count() {
        println!("Invalid template!");
        process::exit(1);
    }
    for i in 0..count - 1 {
        output.push_str(format!("{}\n", &template.lines().nth(i).unwrap().to_owned()).as_str());
    }
    output.push_str(&html_output);
    for i in count..template.lines().count() {
        if let Some(s) = template.lines().nth(i) {
            output.push_str(format!("{}\n", &s.to_owned()).as_str());
        } else {
            break;
        }
    }
    fs::write(
        format!(
            "{}/{}.html",
            Path::new(file).parent().unwrap().to_str().unwrap(),
            Path::new(file).file_stem().unwrap().to_str().unwrap()
        ),
        output,
    )?;

    Ok(())
}
