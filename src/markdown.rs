use pulldown_cmark::{html, Options, Parser};
use std::{
    fs,
    io::{self, Read},
    path::Path,
};

pub fn parse(file: &Path) -> io::Result<()> {
    // Enable all modern Markdown features
    let mut options = Options::empty();
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_SMART_PUNCTUATION);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_TASKLISTS);

    // Open a template and apply it
    if let Ok(mut f) = fs::File::open("templates/base.html") {
        let mut template = String::new();
        f.read_to_string(&mut template)?;
        println!("{}", template);
    } else {
        eprintln!("Was unable to locate template `base.html` file, creating it...");
        super::create_templates();
        parse(file)?;
    }

    let contents = fs::read_to_string(&file)?;
    let parser = Parser::new_ext(&contents, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    // Write the template with the Markdown into the output file

    Ok(())
}
