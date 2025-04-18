extern crate libc;
extern crate rustyline;

mod tokens;
mod colors;

use std::{env, fs::File, os::unix::process::CommandExt, path::Path, process::Command};
use rustyline::DefaultEditor;
use tokens::tokenize_commands;

fn main() {
    unsafe {
        libc::signal(libc::SIGINT, libc::SIG_IGN);
        libc::signal(libc::SIGQUIT, libc::SIG_IGN);
    }
    let mut last_exit_status = true;
    let mut rl = DefaultEditor::new().unwrap();
    let home = env::var("HOME").unwrap();
    if rl.load_history(&format!("{}/.rush_history", home)).is_err() {
        println!("No previous history.");
        File::create(format!("{}/.rush_history", home)).expect("Couldn't create history file");
    }

    loop {
        let prompt_string = generate_prompt(last_exit_status);
        let command_string = read_command(&mut rl, prompt_string);
        let commands = tokenize_commands(&command_string);

        for command in commands {
            last_exit_status = true;
            for mut depent_command in command {
                let mut is_background = false;
                if let Some(&"&") = depent_command.last() {
                    is_background = true;
                    depent_command.pop();
                }
                match depent_command[0] {
                    "exit" => {
                        rl.save_history(&format!("{}/.rush_history", home)).expect("Couldn't save history");
                        std::process::exit(0);
                    },
                    "cd" => {
                        last_exit_status = change_dir(depent_command[1]);
                    }
                    _ => {
                        last_exit_status = execute_command(depent_command, is_background);
                    }

                }
                if last_exit_status == false {
                    break;
                }
            }
        }
    }
}

fn generate_prompt(last_exit_status: bool) -> String {
    let path = env::current_dir().unwrap();
    let prompt = format!(
        "{}RUSHING IN {}{}{}\n",
        colors::ANSI_BOLD,
        colors::ANSI_COLOR_CYAN,
        path.display(),
        colors::RESET
    );
    if last_exit_status {
        return format!(
            "{}{}{}\u{2ba1}{} ",
            prompt,
            colors::ANSI_BOLD,
            colors::GREEN,
            colors::RESET
        );
    } else {
        return format!(
            "{}{}{}\u{2ba1}{} ",
            prompt,
            colors::ANSI_BOLD,
            colors::RED,
            colors::RESET
        )
    }
}

fn read_command(rl: &mut DefaultEditor, prompt_string: String) -> String {
    let mut command_string = rl.readline(&prompt_string).unwrap();

    while command_string.chars().last() == Some('\\') {
        command_string.pop();
        let next_string = rl.readline("").unwrap();
        command_string.push_str(&next_string);
    }

    let _ =rl.add_history_entry(&command_string);
    command_string
}

fn execute_command(command_tokens: Vec<&str>, is_background: bool) -> bool {
    let mut command_instance = Command::new(command_tokens[0]);
    if let Ok(mut child) = unsafe {
        command_instance
        .args(&command_tokens[1..])
        .pre_exec(|| {
            libc::signal(libc::SIGINT, libc::SIG_DFL);
            libc::signal(libc::SIGQUIT, libc::SIG_DFL);
            Result::Ok(())
        })
        .spawn() 
    }
        {
            if is_background == false {
                return child.wait().expect("command wasn't running").success();
            } else {
                colors::success_logger(format!("{} started!", child.id()));
                true
            }
        } else {
            colors::error_logger("Command not found!".to_string());
            false
        }
}

fn change_dir(new_path: &str) -> bool {
    let new_path = Path::new(new_path);
    match env::set_current_dir(&new_path) {
        Err(err) => {
            colors::error_logger(format!("Failed to change the dirctory!\n{}", err));
            return false;
        }
        _ => (),
    }
    
    return true;
}


