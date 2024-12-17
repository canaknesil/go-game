// Go Text Protocol version 2
// https://www.lysator.liu.se/~gunnar/gtp/gtp2-spec-draft2/gtp2-spec.html

use std::process::{Command, Stdio, Child, ChildStdout};
use std::io::{Read, Write};
use std::sync::mpsc::{Sender, Receiver, channel};
use std::thread;
use std::str;


struct Engine {
    id: u32,
    child: Child,
    writer_thread: thread::JoinHandle<()>,
    child_stdout: ChildStdout,
    tx_channel: Sender<String>,
}

#[derive(Debug)]
enum Vertex {
    Pass,
    Coordinate(u32, u32), // Zero-indexed, origin at bottom-left corner.
}

#[derive(Debug)]
struct GTPMove {
    color: Color,
    vertex: Vertex
}

#[derive(Debug)]
enum Color {
    Black,
    White,
}


impl Engine {
    fn new(command: &str) -> Result<Self, String> {
	let mut args = command.split_whitespace();
	let program = args.next().ok_or("Empty command")?;
	let args: Vec<_> = args.collect();

	let mut child = Command::new(program)
	    .args(args.clone())
	    .stdin(Stdio::piped())
	    .stdout(Stdio::piped())
	    .stderr(Stdio::null())
	    .spawn()
	    .map_err(|_| format!("Failed to start child process: {} {}", program, args.join(" ")))?;

	let mut stdin = child.stdin.take().ok_or("Failed to get stdin!".to_string())?;
	let stdout = child.stdout.take().ok_or("Failed to get stdout!".to_string())?;

	// Writing to child with a thread in case OS I/O pipes cause a deadlock.
	let (tx, rx): (Sender<String>, Receiver<String>) = channel();

	let writer = thread::spawn(move || {
	    loop {
		let s = rx.recv();
		match s {
		    Ok(s) => { stdin.write_all(s.as_bytes()).expect("Failed to write to stdin!"); },
		    Err(_) => { break; }
		}
	    }
	});
	
	Ok(Engine {
	    id: 0,
	    child: child,
	    writer_thread: writer,
	    child_stdout: stdout,
	    tx_channel: tx,
	})
    }

    fn write_to_engine(&self, s: &str) -> Result<(), String> {
	// s does not have go end with double new line, they are added here.
	let mut s = String::from(s.trim());
	s.push_str("\n\n");
	self.tx_channel.send(s).map_err(|_| "Failed to send to channel!".to_string())
    }

    fn read_from_engine(&mut self) -> Result<String, String> {
	let mut s = Vec::new();
	loop {
	    let mut buff = [0; 1];
	    let n = self.child_stdout.read(&mut buff).map_err(|_| "Engine sent EOF. This is unexpected!".to_string())?;
	    if n == 0 {
		return Err("Engine sent EOF. This is unexpected!".to_string());
	    } else if n == 1 {
		// do nothing
	    } else {
		return Err(format!("read method should return 1, received {n}"));
	    }
	    s.push(buff[0]);

	    if s.last_chunk::<2>().is_some_and(|x| x == "\n\n".as_bytes())
		|| s.last_chunk::<4>().is_some_and(|x| x == "\r\n\r\n".as_bytes()) {
		break;
	    }
	}

	let s = str::from_utf8(&s).map_err(|_| format!("String cannot be screated from utf8."))?
	    .trim();
	Ok(s.to_string())
    }

    fn send_command(&mut self, command: &str) -> Result<String, String> {
	self.id += 1;

	let command = format!("{} {}", self.id, command);

	self.write_to_engine(&command)?;
	let response_str = self.read_from_engine()?;
	println!("GTP comand  : {}", command);
	println!("GTP response: {}", response_str);

	// Check response status
	let mut tokens = response_str.split_whitespace();
	let first = tokens.next().ok_or("Empty GTP response")?;
	if first == format!("={}", self.id) {
	    Ok(response_str[first.len()..].trim_start().to_string())
	} else if first == format!("?{}", self.id) {
	    Err(response_str[first.len()..].trim_start().to_string())
	} else {
	    Err(response_str)
	}
    }
}


impl Engine {
    //
    // ADMINISTRATION COMMANDS
    //
    
    fn protocol_version(&mut self) -> Result<u32, String> {
	let s = self.send_command("protocol_version")?;
	s.parse::<u32>().map_err(|_| format!("{} cannot be parsed into u32!", s))
    }

    fn name(&mut self) -> Result<String, String> {
	self.send_command("name")
    }

    fn version(&mut self) -> Result<String, String> {
	self.send_command("version")
    }

    fn known_command(&mut self, command_name: &str) -> Result<bool, String> {
	if !is_single_token(command_name) {
	    return Err(format!("command_name '{command_name}' is not single token!"));
	}
	let s = self.send_command(&format!("known_command {}", command_name))?;
	if s == "true" {
	    Ok(true)
	} else if s == "false" {
	    Ok(false)
	} else {
	    Err(format!("command_name '{command_name}' is not 'true' or 'false'!"))
	}
    }

    fn list_commands(&mut self) -> Result<Vec<String>, String> {
	let s = self.send_command("list_commands")?;
	Ok(s.split_whitespace().map(|x| String::from(x)).collect())
    }

    fn quit(mut self) -> Result<(), String> {
	self.send_command("quit")?;
	drop(self.tx_channel);
	self.writer_thread.join().map_err(|_| "Writer thread failed to join!".to_string())?;
	self.child.wait().map_err(|_| "Child process failed to exit!".to_string())?;
	Ok(())
    }

    //
    // SETUP COMMANDS
    //

    fn boardsize(&mut self, size: u32) -> Result<(), String> {
	self.send_command(&format!("boardsize {size}"))?;
	Ok(())
    }

    fn clear_board(&mut self) -> Result<(), String> {
	self.send_command("clear_board")?;
	Ok(())
    }

    fn komi(&mut self, new_komi: f32) -> Result<(), String> {
	self.send_command(&format!("komi {}", new_komi.to_string()))?;
	Ok(())
    }

    fn fixed_handicap(&mut self, number_of_stones: u32) -> Result<Vec<Vertex>, String> {
	let s = self.send_command(&format!("fixed_handicap {number_of_stones}"))?;
	let mut v = Vec::new();
	for tok in s.split_whitespace() {
	    v.push(Vertex::from_string(tok)?);
	}
	Ok(v)
    }

    fn place_free_handicap(&mut self, number_of_stones: u32) -> Result<Vec<Vertex>, String> {
	let s = self.send_command(&format!("place_free_handicap {number_of_stones}"))?;
	let mut v = Vec::new();
	for tok in s.split_whitespace() {
	    v.push(Vertex::from_string(tok)?);
	}
	Ok(v)
    }

    fn set_free_handicap(&mut self, vertices: Vec<Vertex>) -> Result<(), String> {
	let mut command = String::from("set_free_handicap");
	for v in vertices {
	    command.push_str(" ");
	    command.push_str(&v.to_string()?);
	}
	self.send_command(&command)?;
	Ok(())
    }

    //
    // CORE PLAY COMMANDS
    //

    fn play(&mut self, gtp_move: GTPMove) -> Result<(), String> {
	// TODO
	Ok(())
    }

    //
    // TOURNAMENT COMMANDS
    //

    //
    // REGRESSION COMMANDS
    //

    //
    // DEBUG COMMANDS
    //
}


impl Vertex {
    fn to_string(&self) -> Result<String, String> {
	match *self {
	    Self::Pass => Ok("pass".to_string()),
	    Self::Coordinate(x, y) => {
		if x >= 25 || y >= 25 {
		    Err(format!("Vertex coordinate ({}, {}) requires a board larger than 25x25.", x, y))
		} else {
		    let mut x_char = b'A' as u32 + x;
		    if x_char >= b'I' as u32 {
			x_char = x_char + 1;
		    }
		    let x_char = char::from_u32(x_char).ok_or(format!("{} cannot be converted to ASCII!", x_char))?;
		    let y_str = (y+1).to_string();
		    Ok(format!("{}{}", x_char, y_str))
		}
	    }
	}
    }

    fn from_string(s: &str) -> Result<Vertex, String> {
	if s.len() <= 1 {
	    Err(format!("Cannot create a Vertex from string of size {}!", s.len()))
	} else if s == "pass" {
	    Ok(Vertex::Pass)
	} else {
	    let mut x = s.chars().next().unwrap()
		.to_ascii_uppercase() as i32;
	    if x == b'I' as i32 {
	     	return Err(format!("Illegal vertex {}!", s));
	    }
	    if x > b'I' as i32 {
		x -= 1;
	    }
	    x = x - b'A' as i32;
	    if !(x >= 0 && x <= 25) {
		return Err(format!("Illegal vertex {}!", s));
	    }
	    	    
	    let y = &s[1..];
	    let y = y.parse::<u32>().map_err(|_| format!("{} cannot be parsed into u32!", y))? - 1;
	    
	    Ok(Vertex::Coordinate(x as u32, y))
	}
    }
}


fn is_single_token(s: &str) -> bool {
    if s.len() == 0 {
	return false;
    }
    if s.split_whitespace().collect::<Vec<_>>().len() != 1 {
	return false;
    }
    true
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gtp() {
	let mut engine = Engine::new("katago.exe gtp").unwrap();
	
	println!("Protocol version: {:?}", engine.protocol_version().unwrap());
	println!("Name: {:?}", engine.name().unwrap());
	println!("Version: {:?}", engine.version().unwrap());
	println!("Is known_command a known command: {:?}", engine.known_command("known_command").unwrap());
	println!("Is asdf a known command: {:?}", engine.known_command("asdf").unwrap());
	println!("List commands: {:?}", engine.list_commands().unwrap());

	engine.boardsize(19).unwrap();
	println!("Set board size to 19x19.");
	engine.clear_board().unwrap();
	println!("Board cleared.");
	engine.komi(2.5).unwrap();
	println!("Komi set to 2.5.");
	println!("Fixed handicap 5: {:?}", engine.fixed_handicap(5).unwrap());
	engine.clear_board().unwrap();
	println!("Board cleared.");
	println!("Place free handicap 5: {:?}", engine.place_free_handicap(5).unwrap());
	engine.clear_board().unwrap();
	println!("Board cleared.");
	engine.set_free_handicap(vec![Vertex::Coordinate(0, 0), Vertex::Coordinate(18, 18)]).unwrap();
	println!("Set free handicap to A1 and T19.");

	engine.quit().unwrap();
	println!("Quited");
    }
}
