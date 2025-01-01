use crate::gtp::*;
use crate::smart_child::SmartChild;
use std::process::ChildStdout;
use std::io::{Read, Write};
use std::sync::mpsc::{Sender, Receiver, channel};
use std::thread;
use std::str;


pub struct ChildProcessEngine {
    id: u32,
    child: SmartChild,
    writer_thread: Option<thread::JoinHandle<()>>, // writer_thread and tx_channel are declared as Option because they are taken during quit.
    child_stdout: ChildStdout,
    tx_channel: Option<Sender<String>>,
}


impl ChildProcessEngine {
    pub fn new(command: &str) -> Result<Self, String> {
	let mut child = SmartChild::from_command_str(command)?;

	let mut stdin = child.take_stdin()?;
	let stdout = child.take_stdout()?;

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
	
	Ok(Self {
	    id: 0,
	    child: child,
	    writer_thread: Some(writer),
	    child_stdout: stdout,
	    tx_channel: Some(tx),
	})
    }
}


// TODO: Experiment and understand if an empty drop implementation is necessary to ensure destruction of the smart child.
impl Drop for ChildProcessEngine {
    fn drop(&mut self) {
	// Drop trait is implemented for SmartChild, to kill it when
	// it goes out of scope. If user doesn't call quit, or
	// execution of quit is unsuccessful, the child process is
	// killed by the destructor.

	// do nothing
    }
}


impl GTPEngineRaw for ChildProcessEngine {
    fn gen_command_id(&mut self) -> u32 {
	self.id += 1;
	self.id
    }
    
    fn write_to_engine(&self, s: &str) -> Result<(), String> {
	// s does not have go end with double new line, they are added here.
	let mut s = String::from(s.trim());
	s.push_str("\n\n");
	match &self.tx_channel {
	    Some(tx) => tx.send(s).map_err(|_| "Failed to send to channel!".to_string()),
	    None => Err("Sender is None!".to_string()),
	}
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
}


impl GTPEngineMinimal for ChildProcessEngine {
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

    // Note: quit method takes ownership of self, sends quit command,
    // terminates the writer thread, waits for the child to
    // quit. Destructor of Self doesn't call quit, the user is
    // expected to call quit.
    
    fn quit(mut self) -> Result<(), String> {
	self.send_command("quit")?;

	self.tx_channel.take(); // Replace tx with None so it is dropped.
	self.writer_thread
	    .take().ok_or("Writer thread couldn't be taken!".to_string())?
	    .join().map_err(|_| "Writer thread failed to join!".to_string())?;
	
	self.child.wait().map_err(|_| "Child process failed to exit!".to_string())?;
	Ok(())
    }

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

    fn play(&mut self, gtp_move: GTPMove) -> Result<(), String> {
	self.send_command(&format!("play {}", gtp_move.to_string()?))?;
	Ok(())
    }

    fn genmove(&mut self, color: Color) -> Result<GenMove, String> {
	let s = self.send_command(&format!("genmove {}", color.to_string()))?;
	if s == "resign" {
	    Ok(GenMove::Resign)
	} else {
	    Ok(GenMove::Play(Vertex::from_string(&s)?))
	}
    }
}


impl GTPEngineTournament for ChildProcessEngine {
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
}


impl GTPEngineRegression for ChildProcessEngine {
    fn loadsgf(&mut self, filename: &str, move_number: u32) -> Result<(), String> {
	self.send_command(&format!("loadsgf {filename} {move_number}"))?;
	Ok(())
    }

    fn reg_genmove(&mut self, color: Color) -> Result<GenMove, String> {
	let s = self.send_command(&format!("reg_genmove {}", color.to_string()))?;
	if s == "resign" {
	    Ok(GenMove::Resign)
	} else {
	    Ok(GenMove::Play(Vertex::from_string(&s)?))
	}
    }
}


impl GTPEngineExtendedCorePlay for ChildProcessEngine {
    fn undo(&mut self) -> Result<(), String> {
	self.send_command("undo")?;
	Ok(())
    }
}


impl GTPEngineExtendedTournament for ChildProcessEngine {
    fn time_settings(&mut self, main_time: u32, byo_yomi_time: u32, byo_yomi_stones: u32) -> Result<(), String> {
	self.send_command(&format!("time_settings {main_time} {byo_yomi_time} {byo_yomi_stones}"))?;
	Ok(())
    }

    fn time_left(&mut self, color: Color, time: u32, stones: u32) -> Result<(), String> {
	self.send_command(&format!("time_left {} {} {}", color.to_string(), time, stones))?;
	Ok(())
    }

    fn final_score(&mut self) -> Result<Score, String> {
	let s = self.send_command("final_score")?;
	if !is_single_token(&s) {
	    return Err(format!("Final score '{s}' is not a single token!"));
	}
	Score::from_string(&s)
    }

    fn final_status_list(&mut self, status: Status) -> Result<Vec<Vertex>, String> {
	let s = self.send_command(&format!("final_status_list {}", status.to_string()))?;
	let mut v = Vec::new();
	for tok in s.split_whitespace() {
	    v.push(Vertex::from_string(tok)?);
	}
	Ok(v)
    }
}


impl GTPEngineDebug for ChildProcessEngine {    
    fn showboard(&mut self) -> Result<String, String> {
	self.send_command("showboard")
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gtp() {
	let mut engine = ChildProcessEngine::new("katago.exe gtp").unwrap();
	
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

	engine.play(GTPMove::new(Color::Black, Vertex::Coordinate(0, 1))).unwrap();
	println!("Black played A2.");
	engine.play(GTPMove::new(Color::White, Vertex::Pass)).unwrap();
	println!("White passed.");
	println!("Engine generate move for black: {:?}", engine.genmove(Color::Black).unwrap());
	engine.undo().unwrap();
	println!("Undo.");

	engine.time_settings(1000, 1000, 1000).unwrap();
	println!("Time settings 1000 1000 1000.");
	engine.time_left(Color::Black, 1000, 1000).unwrap();
	println!("Time left 1000 1000.");
	println!("Final score is {:?}.", engine.final_score().unwrap());
	println!("Final status list for alive: {:?}", engine.final_status_list(Status::Alive).unwrap());
	println!("Final status list for seki : {:?}", engine.final_status_list(Status::Seki).unwrap());
	println!("Final status list for dead : {:?}", engine.final_status_list(Status::Dead).unwrap());

	// loadsgf is note tested
	//println!("Engine generate move for black for regression: {:?}", engine.reg_genmove(Color::Black).unwrap());
	println!("Showing board:\n{}", engine.showboard().unwrap());

	engine.quit().unwrap();
	println!("Quited");
    }
}
