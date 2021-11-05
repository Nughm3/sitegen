use color_eyre::eyre::Result;
use glob::glob;
use pulldown_cmark::{html, Options, Parser};
use std::{
    fs,
    io::{self, Read},
    path::Path,
    process,
};
use titlecase::titlecase;

pub fn parse(file: &Path) -> Result<()> {
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
        eprintln!("Was unable to locate template `base.html` file, exiting...");
        process::exit(1);
    }

    let contents = fs::read_to_string(&file)?;
    let parser = Parser::new_ext(&contents, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    let output = template
        .replace("{{ body }}", &html_output)
        .replace(
            "{{ title }}",
            &titlecase(file.file_stem().unwrap().to_str().unwrap()),
        )
        .replace("{{ tree }}", &tree()?);

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

fn tree() -> io::Result<String> {
    let mut html = String::new();
    for entry in glob("./**/*.md")
        .expect("Failed to read directories")
        .filter_map(|e| e.ok())
    {
        let name = entry.display().to_string();
        let link = name[..name.len() - 3].to_string();
        let name = name[..name.len() - 3].to_string();
        html.push_str(format!("<a href=\"{}\">{}</a>\n", link, name).as_str());
    }
    Ok(html)
}
