#[macro_use] extern crate log;

use std::process::{Child, Command, Stdio};

use std::io::Read;
use std::io::Write;

use std::fmt;
use std::thread;
use std::time::Duration;

use std::cell::RefCell;

mod error;
use error::{Result, EngineError};

pub struct Engine {
    engine: RefCell<Child>,
    depth: u32
}

const DEFAULT_DEPTH: u32 = 10;

impl Engine {
    pub fn new(path: &str) -> Result<Engine> {
        let cmd = Command::new(path)
                          .stdin(Stdio::piped())
                          .stdout(Stdio::piped())
                          .spawn()
                          .expect("Unable to run engine");

        let res = Engine {
            engine: RefCell::new(cmd),
            depth: DEFAULT_DEPTH
        };

        res.read_line()?;
        res.command("uci")?;

        Ok(res)
    }

    pub fn depth(mut self, newdepth: u32) -> Engine {
        self.depth = newdepth;
        self
    }

    pub fn set_position(&self, moves: &Vec<String>) -> Result<()> {
        self.write_fmt(format_args!("position startpos moves {}\n",
                                    moves.join(" ")))?;
        Ok(())
    }

    pub fn bestmove(&self) -> Result<String> {
        self.write_fmt(format_args!("go depth {}\n", self.depth))?;
        loop {
            let s = self.read_line()?;
            debug!("{}", s);
            if s.starts_with("bestmove") {
                return Ok(s.split(" ").collect::<Vec<&str>>()[1].trim().to_string());
            }
        }
    }

    pub fn setoption(&self, name: &str, value: &str) -> Result<()> {
        self.write_fmt(format_args!("setoption name {} value {}\n",
                                    name, value))?;
        let error_msg =  self.read_left_output()?;
        
        if error_msg.trim().is_empty() {
            Ok(())
        } else {
            Err(EngineError::UnknownOption(name.to_string()))
        }
    }

    pub fn command(&self, cmd: &str) -> Result<String> {
        self.write_fmt(format_args!("{}\n", cmd.trim()))?;
        thread::sleep(Duration::from_millis(100));
        self.read_left_output()
    }

    fn read_left_output(&self) -> Result<String> {
        let mut s: Vec<String> = vec![];

        self.write_fmt(format_args!("isready\n"))?;
        loop {
            let next_line = self.read_line()?;
            match next_line.trim() {
                "readyok" => return Ok(s.join("\n")),
                other     => s.push(other.to_string())
            }
        }
    }

    fn write_fmt(&self, args: fmt::Arguments) -> Result<()> {
        info!("Command: {:?}", fmt::format(args));
        self.engine.borrow_mut().stdin.as_mut().unwrap().write_fmt(args)?;
        Ok(())
    }

    fn read_line(&self) -> Result<String> {
        let mut s = String::new();
        let mut buf: Vec<u8> = vec![0];

        loop {
            self.engine.borrow_mut().stdout.as_mut().unwrap().read(&mut buf)?;
            s.push(buf[0] as char);
            if buf[0] == '\n' as u8 {
                break
            }
        }
        Ok(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let engine = Engine::new("stockfish").unwrap();
        engine.setoption("Skill Level", "15").unwrap();
        let t = engine.bestmove().unwrap();

        println!("{}", t);
    }
}
