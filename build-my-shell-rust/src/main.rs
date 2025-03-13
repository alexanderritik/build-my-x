use std::collections::HashMap;
use std::io::{self, Write};
use std::{env, process};
use std::path::PathBuf;
use std::process::Command;

fn exit(_arg0: &str, args: &[&str]) -> u8 {
    let exit_code = args.get(0).unwrap_or(&"0");
    let exit_code = exit_code.parse::<i32>().unwrap_or(0);
    process::exit(exit_code);
}

fn echo(_arg0: &str, args: &[&str]) -> u8 {
    let mut output = args.join(" ");
    output = output.replace("'", "");
    output = output.replace("\n", "");
    println!("{}", output);
    return 0;
}

fn pwd(_arg0: &str, _args: &[&str]) -> u8 {
    let current_dir = env::current_dir().unwrap();
    println!("{}", current_dir.display());
    return 0;
}

fn cd(_arg0: &str, args: &[&str]) -> u8 {
    let new_dir = args.get(0).unwrap_or(&"");
    let new_dir = PathBuf::from(new_dir);

    if String::from("~") == new_dir.to_str().unwrap() {
        let home = env::var("HOME").unwrap();
        env::set_current_dir(home).unwrap();
        return 0;
    }

    if new_dir.is_dir() {
        env::set_current_dir(new_dir).unwrap();
        return 0;
    } else {
        eprintln!("{}: {}: No such file or directory", _arg0, new_dir.display());
        return 1;
    }
}

fn cat(_arg0: &str, args: &[&str]) -> u8 {
    let mut exit_code = 0;

    for file in args {
        let file = PathBuf::from(file);
        if file.is_file() {
            let contents = std::fs::read_to_string(file).unwrap();
            println!("{}", contents);
        } else {
            eprintln!("{}: {}: No such file or directory", _arg0, file.display());
            exit_code = 1;
        }
    }

    return exit_code;
}

fn type_builtin(arg0: &str, args: &[&str]) -> u8 {
    let builtins = vec!["exit", "echo", "type", "pwd"];

    return if let Some(command) = args.get(0) {
        if builtins.contains(command) {
            println!("{} is a shell {}", command, "builtin");
        } else if let Some(path) = get_command_path(command) {
            println!("{} is {}", command , path.into_os_string().into_string().unwrap());
        } else {
            println!("{} not found", command);
        }

        0
    } else {
        eprintln!("Usage: {} <command>", arg0);
        1
    };
}

fn get_command_path(command: &&str) -> Option<PathBuf> {
    env::var_os("PATH").and_then(|paths| {
        env::split_paths(&paths).filter_map(|dir| {
            let full_path = dir.join(&command);
            if full_path.is_file() {
                Some(full_path)
            } else {
                None
            }
        }).next()
    })
}

fn execute_command(arg0: &str, args: &Vec<&str>) {
    Command::new(arg0)
        .args(args)
        .spawn()
        .expect("failed to execute command")
        .wait()
        .expect("Failed to wait");
}

fn main() {
    let mut commands: HashMap<String, fn(&str, &[&str]) -> u8> = HashMap::new();

    commands.insert(String::from("hello"), |_arg0, _args| {
        println!("Hello, World!");
        return 0;
    });

    commands.insert(String::from("exit"), exit);
    commands.insert(String::from("echo"), echo);
    commands.insert(String::from("type"), type_builtin);
    commands.insert(String::from("pwd"), pwd);
    commands.insert(String::from("cd"), cd);
    commands.insert(String::from("cat"), cat);

    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        // Wait for user input
        let stdin = io::stdin();
        let mut input = String::new();
        stdin.read_line(&mut input).unwrap();

        // let mut split = input.split_whitespace();
        let mut split  = input.split(' ');

        let Some(arg0) = split.next() else {
            continue;
        };

        let rest: Vec<_> = split.collect();

        if let Some(cmd) = commands.get(arg0) {
            cmd(arg0, &rest);
        } else if let Some(_path) = get_command_path(&arg0) {
            execute_command(arg0, &rest);
        } else {
            println!("{}: {} not found", arg0, "command");
        }
    }
}