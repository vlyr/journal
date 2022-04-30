use std::env;
use std::error::Error;
use std::fs;
use std::io;
use std::io::prelude::*;
use std::path::Path;
use std::process::Command;

const JOURNAL_DATA_PATH: &str = "/.local/share/journal";

const HELP_MESSAGE: &str = "journal - a tool for having a personal daily journal via git
subcommands:
  save - commit and push changes to the journal repository
  sync - pulls changes from the journal repository
  write <text editor> - opens a file for today's journal with the text editor provided

when running for the first time, journal automatically initializes a data directory at `~/.local/share/journal/`.";

pub struct Config {
    git_remote_url: String,
    date_string: String,
    path_string: String,
}

fn run_command(cmd: &str, args: &[&str]) -> Result<String, Box<dyn Error>> {
    let output = Command::new(cmd)
        .args(args)
        .spawn()?
        .wait_with_output()?
        .stdout;

    Ok(String::from_utf8(output)?)
}

fn input(message: &str) -> Result<String, Box<dyn Error>> {
    print!("{}", message);
    io::stdout().flush()?;

    let mut s = String::new();
    io::stdin().read_line(&mut s)?;
    Ok(s)
}

fn initialize() -> Result<Config, Box<dyn Error>> {
    let home = env::var("HOME")?;
    let path_string = &format!("{}{}", home, JOURNAL_DATA_PATH);
    let path = Path::new(path_string);

    if !path.exists() {
        println!(
            "Initializing a directory for storing journals in {}...",
            path.display()
        );

        fs::create_dir(path)?;
        env::set_current_dir(path)?;

        run_command("git", &["init"])?;

        let s = input("Enter a git remote URL for the repository to store your journals in: ")?;
        run_command("git", &["remote", "add", "origin", &s])?;

        println!("Journal directory has been initialized.");
    } else {
        env::set_current_dir(path)?;
    }

    let output = String::from_utf8(
        Command::new("git")
            .args(&["remote", "get-url", "origin"])
            .output()?
            .stdout,
    )?;

    let date = chrono::prelude::Utc::now().date().format("%d-%m-%Y");

    Ok(Config {
        git_remote_url: output,
        date_string: date.to_string(),
        path_string: path_string.to_string(),
    })
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args();
    args.next();
    let config = initialize()?;

    match args.next() {
        Some(arg) => match arg.as_ref() {
            "write" => {
                let text_editor = args.next().expect("vi");
                run_command(
                    &text_editor,
                    &[&format!(
                        "{}/{}.txt",
                        config.path_string, config.date_string
                    )],
                ).expect(&format!("Failed running text editor \"{}\". Pass a valid text editor as the second command argument.", text_editor));
            }

            "save" => {
                run_command("git", &["add", "."])?;
                run_command(
                    "git",
                    &[
                        "commit",
                        "-m",
                        &format!("journal saved on {}", config.date_string),
                    ],
                )?;
                run_command("git", &["branch", "-M", "main"])?;
                run_command("git", &["push", "origin", "main"])?;
                println!("Done.");
            }

            "sync" => {
                run_command("git", &["pull", "origin", "main"])?;
                println!("Done.");
            }
            _ => (),
        },

        None => println!("{}", HELP_MESSAGE),
    }

    Ok(())
}
