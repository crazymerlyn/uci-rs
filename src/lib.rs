#[macro_use]
extern crate log;

use std::process::{Child, Command, Stdio};

use std::io::Write;
use std::io::{self, Read};

use std::fmt;
use std::thread;
use std::time::Duration;

use std::cell::RefCell;

pub struct Engine {
    engine: RefCell<Child>,

    movetime: u32,
}

const DEFAULT_TIME: u32 = 100;

impl Engine {
    /// Create a new [`Engine`] instance.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the engine executable.
    ///
    /// # Panics
    ///
    /// * Panics if the engine couldn't be spawned (path is invalid, execution permission denied, etc.)
    ///
    /// # Errors
    ///
    /// Returns an `EngineError` if there's an errors while communicating with the engine.
    ///
    /// [`Engine`]: struct.Engine.html
    pub fn new(path: &str) -> Result<Engine> {
        let cmd = Command::new(path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("Unable to run engine");

        let res = Engine {
            engine: RefCell::new(cmd),
            movetime: DEFAULT_TIME,
        };

        res.read_line()?;
        res.command("uci")?;

        Ok(res)
    }

    /// Changes the amount of time the engine spends looking for a move
    ///
    /// # Arguments
    ///
    /// * `new_movetime` - New timelimit in milliseconds
    #[must_use]
    pub fn movetime(mut self, new_movetime: u32) -> Engine {
        self.movetime = new_movetime;
        self
    }

    /// Asks the engine to play the given moves from the initial position on it's internal board.
    ///
    /// # Arguments
    ///
    /// * `moves` - A list of moves for the engine to play. Uses Coordinate notation
    ///
    /// # Errors
    ///
    /// Returns `EngineError` if there's an error while communicating with the engine.
    ///
    /// # Examples
    ///
    /// ```
    /// let engine = uci::Engine::new("stockfish").unwrap();
    /// let moves = vec!["e2e4".to_string(), "e7e5".to_string()];
    /// engine.make_moves(&moves).unwrap();
    /// ```
    pub fn make_moves(&self, moves: &[String]) -> Result<()> {
        self.write_fmt(format_args!(
            "position startpos moves {}\n",
            moves.join(" ")
        ))
    }

    /// Asks the engine to use the position represented by the given FEN string
    ///
    /// # Errors
    ///
    ///
    ///
    /// # Examples
    ///
    /// ```
    /// let engine = uci::Engine::new("stockfish").unwrap();
    /// engine.set_position("2k4R/8/3K4/8/8/8/8/8 b - - 0 1").unwrap();
    /// assert_eq!(engine.bestmove().unwrap(), "c8b7");
    /// ```
    pub fn set_position(&self, fen: &str) -> Result<()> {
        let moves: Vec<String> = vec![];
        self.make_moves_from_position(fen, &moves)
    }

    /// Asks the engine to use the position represented by the given FEN string
    /// and then play the given moves from that position
    ///
    /// # Errors
    ///
    /// Returns an `EngineError` if there's an error while communicating with the engine.
    pub fn make_moves_from_position(&self, fen: &str, moves: &[String]) -> Result<()> {
        self.write_fmt(format_args!(
            "position fen {} moves {}\n",
            fen,
            moves.join(" ")
        ))
    }

    /// Returns the best move in the current position according to the engine
    /// # Errors
    /// Returns an error if the engine is not ready to return a move
    pub fn bestmove(&self) -> Result<String> {
        self.write_fmt(format_args!("go movetime {}\n", self.movetime))?;
        loop {
            let s = self.read_line()?;
            debug!("{}", s);
            if s.starts_with("bestmove") {
                return Ok(s.split(' ').collect::<Vec<&str>>()[1].trim().to_string());
            }
        }
    }

    /// Sets an engine specific option to the given value
    ///
    /// # Arguments
    ///
    /// * `name`  - Name of the option
    /// * `value` - New value for the option
    ///
    /// # Errors
    ///
    /// Returns an `EngineError` if the engine doesn't support the option
    ///
    /// # Examples
    ///
    /// ```
    /// let engine = uci::Engine::new("stockfish").unwrap();
    /// engine.set_option("Skill Level", "5").unwrap();
    /// ```
    pub fn set_option(&self, name: &str, value: &str) -> Result<()> {
        self.write_fmt(format_args!("setoption name {name} value {value}\n"))?;
        let error_msg = self.read_left_output()?;

        if error_msg.trim().is_empty() {
            Ok(())
        } else {
            Err(EngineError::UnknownOption(name.to_string()))
        }
    }

    /// Sends a command to the engine and returns the output
    ///
    /// # Errors
    ///
    /// Returns an `EngineError` if there was an error while sending the command to the engine
    ///
    /// # Examples
    ///
    /// ```
    /// let engine = uci::Engine::new("stockfish").unwrap();
    /// let analysis = engine.command("go depth 10").unwrap();
    /// println!("{}", analysis);
    /// ```
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
                other => s.push(other.to_string()),
            }
        }
    }

    fn write_fmt(&self, args: fmt::Arguments) -> Result<()> {
        info!("Command: {:?}", fmt::format(args));
        self.engine
            .borrow_mut()
            .stdin
            .as_mut()
            .unwrap()
            .write_fmt(args)?;
        Ok(())
    }

    fn read_line(&self) -> Result<String> {
        let mut s = String::new();
        let mut buf: Vec<u8> = vec![0];

        loop {
            let _ = self
                .engine
                .borrow_mut()
                .stdout
                .as_mut()
                .unwrap()
                .read(&mut buf)?;
            s.push(buf[0] as char);
            if buf[0] == b'\n' {
                break;
            }
        }
        Ok(s)
    }
}

/// The error type for any errors encountered with the engine.
#[derive(Debug)]
pub enum EngineError {
    /// Wrapper around any io errors encountered while trying to communicate with the engine.
    Io(io::Error),

    /// Engine doesn't recognize the specified option.
    UnknownOption(String),
}

use self::EngineError::{Io, UnknownOption};
impl fmt::Display for EngineError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Io(ref err) => write!(f, "IO error: {err}"),
            UnknownOption(ref option) => write!(f, "No such option: '{option}'"),
        }
    }
}

impl From<io::Error> for EngineError {
    fn from(err: io::Error) -> EngineError {
        Io(err)
    }
}

/// A Result type which uses [`EngineError`] for representing errors.
///
/// [`EngineError`]: enum.EngineError.html
pub type Result<T> = std::result::Result<T, EngineError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let engine = Engine::new("stockfish").unwrap().movetime(200);
        engine.set_option("Skill Level", "15").unwrap();
        let t = engine.bestmove().unwrap();

        println!("{t}");
    }
}
