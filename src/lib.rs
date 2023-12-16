use edit::{edit_file, Builder};
use owo_colors::OwoColorize;
use slug::slugify;
use std::{
    fs,
    io::{self, Write},
    path::PathBuf,
};

pub fn write(garden_path: PathBuf, title: Option<String>) -> Result<(), std::io::Error> {
    let (mut file, filepath) = Builder::new()
        .suffix(".md")
        .rand_bytes(5)
        .tempfile_in(&garden_path)?
        .keep()?;

    let template = format!("# {}", title.as_deref().unwrap_or(""));
    file.write_all(template.as_bytes())?;
    edit_file(&filepath)?;

    let contents = fs::read_to_string(&filepath)?;

    let document_title = title.or_else(|| {
        contents
            .lines()
            .find(|v| v.starts_with("# "))
            .map(|line| line.trim_start_matches("# ").to_string())
    });

    let filename = match document_title {
        Some(raw_title) => confirm_filename(&raw_title),
        None => ask_for_filename(),
    }
    .map(|title| slugify(title))?;

    for attempt in 0.. {
        let mut dest = garden_path.join(if attempt == 0 {
            filename.clone()
        } else {
            format!("{filename}-{:02}", attempt)
        });

        dest.set_extension("md");

        if dest.exists() {
            continue;
        }

        fs::rename(filepath, &dest)?;

        break;
    }

    Ok(())
}

fn ask_for_filename() -> io::Result<String> {
    rprompt::prompt_reply(
        "Enter filename
> "
        .blue()
        .bold(),
    )
}

fn confirm_filename(raw_title: &str) -> io::Result<String> {
    loop {
        // prompt defaults to uppercase char in question
        // this is a convention, not a requirement enfocred by
        // the code
        let result = rprompt::prompt_reply(&format!(
            "current title {}
Do you want a different title? (y/{}): ",
            &raw_title.bold().green(),
            "N".bold()
        ))?;

        match result.as_str() {
            "y" | "Y" => break ask_for_filename(),
            "n" | "N" | "" => {
                // the capital N in the prompt means "default",
                // so we handle "" as input here.
                break Ok(raw_title.to_string());
            }
            _ => {
                // ask again, continue the loop...
            }
        };
    }
}
