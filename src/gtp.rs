// Go Text Protocol version 2 - Draft 2
// https://www.lysator.liu.se/~gunnar/gtp/gtp2-spec-draft2/gtp2-spec.html

use std::str;

//
// GTP COMMANDS THAT CONTROLLER CALLS
//

pub trait GTPEngineMinimal {
    fn protocol_version(&mut self) -> Result<u32, String>;
    fn name(&mut self) -> Result<String, String>;
    fn version(&mut self) -> Result<String, String>;
    fn known_command(&mut self, command_name: &str) -> Result<bool, String>;
    fn list_commands(&mut self) -> Result<Vec<String>, String>;
    fn quit(self) -> Result<(), String>;
    fn boardsize(&mut self, size: u32) -> Result<(), String>;
    fn clear_board(&mut self) -> Result<(), String>;
    fn komi(&mut self, new_komi: f32) -> Result<(), String>;
    fn play(&mut self, gtp_move: GTPMove) -> Result<(), String>;
    fn genmove(&mut self, color: Color) -> Result<GenMove, String>;
}

pub trait GTPEngineTournament {
    fn fixed_handicap(&mut self, number_of_stones: u32) -> Result<Vec<Vertex>, String>;
    fn place_free_handicap(&mut self, number_of_stones: u32) -> Result<Vec<Vertex>, String>;
    fn set_free_handicap(&mut self, vertices: Vec<Vertex>) -> Result<(), String>;
}

pub trait GTPEngineRegression {
    fn loadsgf(&mut self, filename: &str, move_number: u32) -> Result<(), String>;
    fn reg_genmove(&mut self, color: Color) -> Result<GenMove, String>;
}

pub trait GTPEngineExtendedCorePlay {
    fn undo(&mut self) -> Result<(), String>;
}

pub trait GTPEngineExtendedTournament {
    fn time_settings(&mut self, main_time: u32, byo_yomi_time: u32, byo_yomi_stones: u32) -> Result<(), String>;
    fn time_left(&mut self, color: Color, time: u32, stones: u32) -> Result<(), String>;
    fn final_score(&mut self) -> Result<Score, String>;
    fn final_status_list(&mut self, status: Status) -> Result<Vec<Vertex>, String>;
}

pub trait GTPEngineDebug {
    fn showboard(&mut self) -> Result<String, String>;
}


pub trait GTPEngineRaw {
    // Engine implements these methods to make raw command execution work.
    fn gen_command_id(&mut self) -> u32 { 0 }
    fn write_to_engine(&self, s: &str) -> Result<(), String>;
    fn read_from_engine(&mut self) -> Result<String, String>;

    // Both controller and engine can call these methods for raw command execution.
    fn send_command(&mut self, command: &str) -> Result<String, String> {
	let id = self.gen_command_id();
	let command = format!("{} {}", id, command);

	self.write_to_engine(&command)?;
	let response_str = self.read_from_engine()?;
	println!("GTP comand  : {}", command);
	println!("GTP response: {}", response_str);

	// Check response status
	let mut tokens = response_str.split_whitespace();
	let first = tokens.next().ok_or("Empty GTP response")?;
	if first == format!("={}", id) {
	    Ok(response_str[first.len()..].trim_start().to_string())
	} else if first == format!("?{}", id) {
	    Err(response_str[first.len()..].trim_start().to_string())
	} else {
	    Err(response_str)
	}
    }
}


//
// TYPES USED IN ABOVE TRAITS
//

#[derive(Debug)]
pub enum Vertex {
    Pass,
    Coordinate(u32, u32), // Zero-indexed, origin at bottom-left corner.
}

#[derive(Debug)]
pub struct GTPMove {
    color: Color,
    vertex: Vertex
}

#[derive(Debug)]
pub enum GenMove {
    Resign,
    Play(Vertex),
}

#[derive(Debug)]
pub enum Color {
    Black,
    White,
}

#[derive(Debug)]
pub enum Status {
    Alive,
    Seki,
    Dead,
}

#[derive(Debug)]
pub enum Score {
    Black(f32),
    White(f32),
    Draw,
}


impl Vertex {
    pub fn to_string(&self) -> Result<String, String> {
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

    pub fn from_string(s: &str) -> Result<Vertex, String> {
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


impl GTPMove {
    pub fn new(color: Color, vertex: Vertex) -> Self {
	Self { color: color, vertex: vertex }
    }

    pub fn to_string(&self) -> Result<String, String> {
	Ok(format!("{} {}", self.color.to_string(), self.vertex.to_string()?))
    }
}


impl Color {
    pub fn to_string(&self) -> String {
	match *self {
	    Self::Black => "B".to_string(),
	    Self::White => "W".to_string()
	}
    }
}


impl Status {
    pub fn to_string(&self) -> String {
	match *self {
	    Self::Alive => "alive".to_string(),
	    Self::Seki => "seki".to_string(),
	    Self::Dead => "dead".to_string()
	}
    }
}


impl Score {
    pub fn from_string(s: &str) -> Result<Self, String> {
	if !is_single_token(s) {
	    return Err(format!("Score '{s}' is not single token!"));
	}
	if s == "0" {
	    Ok(Self::Draw)
	} else {
	    let v: Vec<_> = s.split("+").collect();
	    if v.len() != 2 {
		Err(format!("Illegal score format: {s}"))
	    } else {
		let color = v[0];
		let score = v[1];
		let score = score.parse::<f32>().map_err(|_| format!("Cannot parse score {score}!"))?;
		if color == "B" {
		    Ok(Self::Black(score))
		} else if color == "W" {
		    Ok(Self::White(score))
		} else {
		    Err(format!("Illegal score format: {s}"))
		}
	    }
	}
    }
}


//
// ADDITIONAL UTILITY CODE
//

pub fn is_single_token(s: &str) -> bool {
    if s.len() == 0 {
	return false;
    }
    if s.split_whitespace().collect::<Vec<_>>().len() != 1 {
	return false;
    }
    true
}
