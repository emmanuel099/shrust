use std::io;
use std::io::prelude::*;
use std::string::ToString;

use std::error::Error;
use std::fmt;

use std::collections::BTreeMap;

#[derive(Debug)]
pub enum ExecError {
    Other(String),
    MissingArgs,
    UnknownCommand(String),
    Quit
}
use ExecError::*;

impl fmt::Display for ExecError {
    fn fmt(&self, format: &mut fmt::Formatter) -> fmt::Result {
        return match self {
            &Quit => write!(format, "Quit"),
            &UnknownCommand(ref cmd) => write!(format, "Unknown Command {}", cmd),
            &MissingArgs => write!(format, "Not enough arguments"),
            &Other(ref string) => write!(format, "{}", string)
        };
    }
}

impl Error for ExecError {
    fn description(&self) -> &str {
        return match self {
            &Quit => "The command requested to quit",
            &UnknownCommand(..) => "The provided command is unknown",
            &MissingArgs => "Not enough arguments have been provided",
            &Other(..) => "Other error occured"
        };
    }
}

pub type ExecResult = Result<(), ExecError>;

pub type CmdFn<T> = Fn(&mut T, &[&str]);

pub struct Command<T> {
    name: String,
    description: String,
    nargs: usize,
    func: Box<CmdFn<T>>
}

impl <T> Command<T> {
    pub fn new(name: String, description: String, nargs: usize, func: Box<CmdFn<T>>) -> Command<T> {
        return Command {
            name: name,
            description: description,
            nargs: nargs,
            func: func
        };
    }

    pub fn help(&self) {
        println!("{} :\t{}", self.name, self.description);
    }

    pub fn run(&self, value: &mut T, args: &[&str]) -> ExecResult {
        if args.len() < self.nargs {
            return Err(MissingArgs);
        }
        (self.func)(value, args);
        return Ok(());
    }
}

pub struct Shell<T> {
    commands: BTreeMap<String, Command<T>>,
    value: T,
    prompt: String
}

impl <T> Shell<T> {
    pub fn new(value: T) -> Shell<T> {
        return Shell {
            commands: BTreeMap::new(),
            value: value,
            prompt: String::from(">")
        };
    }

    pub fn set_prompt(&mut self, prompt: String) {
        self.prompt = prompt;
    }

    pub fn register_command(&mut self, cmd: Command<T>) {
        self.commands.insert(cmd.name.clone(), cmd);
    }

    pub fn new_command<S: ToString, F: Fn(&mut T, &[&str]) + 'static>(&mut self, name: S, description: S, nargs: usize, func: F) {
        self.register_command(Command::new(name.to_string(), description.to_string(), nargs, Box::new(func)));
    }

    pub fn help(&self) -> ExecResult {
        for cmd in self.commands.values() {
            cmd.help();
        }
        return Ok(());
    }

    pub fn run(&mut self, line: &str) -> ExecResult {
        let mut splt = line.trim().split_whitespace();
        return match splt.next() {
            None => Ok(()),
            Some("help") => self.help(),
            Some("quit") => Err(Quit),
            Some(cmd) => match self.commands.get(cmd) {
                None => Err(UnknownCommand(cmd.to_string())),
                Some(c) => c.run(&mut self.value, &splt.collect::<Vec<&str>>())
            }
        };
    }

    fn print_prompt(&self) {
        let mut stdout = io::stdout();
        write!(stdout, "{}", self.prompt).unwrap();
        stdout.flush().unwrap();
    }

    pub fn run_loop(&mut self) {
        let stdin = io::stdin();
        self.print_prompt();
        for line in stdin.lock().lines().map(|l| l.unwrap()) {
            if let Err(e) =  self.run(&line) {
                match e {
                    Quit => return,
                    e @ _ => println!("{}", e)
                };
            }
            self.print_prompt();
        }
    }
}
