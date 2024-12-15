use std::collections::VecDeque;
use std::iter::Rev;


#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Point {
    Black,
    White,
    Empty
}

#[derive(Copy, Clone, PartialEq)]
pub enum Stone {
    Black,
    White
}

#[derive(Copy, Clone, PartialEq)]
pub enum Turn {
    Black,
    White
}

#[derive(Clone, PartialEq)]
pub struct Board {
    matrix: Vec<Vec<Point>>,
    size: usize
}

#[derive(Clone)]
struct HistoryItem {
    board: Board,
}

#[derive(Clone)]
struct History {
    items: Vec<HistoryItem>
}

#[derive(Clone)]
pub struct Model {
    board: Board,
    turn: Turn,
    history: History, // doesn't store the current board
    black_captures: i32, // number of stones that black captured
    white_captures: i32
}


impl Model {
    pub fn make_model(board_size: usize) -> Self {
	Self {
	    board: Board::make_empty_board(board_size),
	    turn: Turn::Black,
	    history: History::new(),
	    black_captures: 0,
	    white_captures: 0
	}
    }

    pub fn get_board(&self) -> &Board {
	&self.board
    }

    pub fn get_board_size(&self) -> usize {
	self.board.size
    }

    pub fn get_turn(&self) -> Turn {
	self.turn
    }

    pub fn get_black_captures(&self) -> i32 {
	self.black_captures
    }

    pub fn get_white_captures(&self) -> i32 {
	self.white_captures
    }

    pub fn setup_switch_turn(&mut self) -> Result<(), &str> {
	self.switch_turn();
	self.reset_history_during_setup();
	Ok(())
    }

    fn switch_turn(&mut self) {
	self.turn = match self.turn {
	    Turn::Black => Turn::White,
	    Turn::White => Turn::Black
	};
    }

    pub fn setup_add_stone(&mut self, x: usize, y: usize, stone: Stone) -> Result<(), &str> {
	let r = self.board.add_stone(x, y, stone);
	self.reset_history_during_setup();
	r
    }

    pub fn setup_remove_stone(&mut self, x: usize, y: usize) -> Result<(), &str> {
	let r = self.board.remove_stone(x, y);
	self.reset_history_during_setup();
	r
    }

    fn reset_history_during_setup(&mut self) {
	println!("Resetting history due to custom setup!");
	self.history = History::new();
    }

    pub fn make_move(&mut self, x: usize, y: usize) -> Result<(), &'static str> {
	let point = self.board.get(x, y)?;
	if point == Point::Empty {
	    let mut new_board = self.board.clone();
	    new_board.set(x, y, match self.turn {
		Turn::Black => Point::Black,
		Turn::White => Point::White
	    })?;
	    let captures = new_board.capture_stones(x, y)?;
	    if new_board.is_suicide(x, y)? {
		Err("Suicide!")
	    } else if self.is_repetition(&new_board) {
		Err("Repetition!")
	    } else {
		self.history.push(HistoryItem{board: self.board.clone()});
		self.board = new_board.clone();
		match self.turn {
		    Turn::Black => { self.black_captures += captures; },
		    Turn::White => { self.white_captures += captures; }
		}
		self.switch_turn();
		Ok(())
	    }
	} else {
	    Err("Point is not empty!")
	}
    }

    fn is_repetition(&self, board: &Board) -> bool {
	for item in self.history.in_reverse() {
	    if item.board == *board {
		return true;
	    }
	}
	false
    }

    pub fn calculate_territory_score(&self) -> (i32, i32) {
	// Japanese and Korean rules
	// Count empty intersections
	// Consider Seki
	let (black, white, _neutral) = self.board.calculate_territory_score();
	(black + self.black_captures, white + self.white_captures)
    }

    pub fn calculate_area_score(&self) -> (i32, i32) {
	// Chinese rules
	// Count stones on plus empty intersections
	let (black, white) = self.board.calculate_area_score();
	(black + self.black_captures, white + self.white_captures)
    }
}


impl Board {
    fn make_empty_board(board_size: usize) -> Self {
	Board {
	    matrix: vec![vec![Point::Empty; board_size]; board_size],
	    size: board_size
	}
    }

    pub fn get(&self, x: usize, y: usize) -> Result<Point, &'static str> {
	self.matrix
	    .get(x).ok_or("Board index x out of range!")?
	    .get(y).ok_or("Board index y out of range!").copied()
    }

    fn set(&mut self, x: usize, y: usize, p: Point) -> Result<(), &'static str> {
	let r = self.matrix
	    .get_mut(x).ok_or("Board index x out of range!")?
	    .get_mut(y).ok_or("Board index y out of range!")?;
	*r = p;
	Ok(())
    }

    fn remove_stone(&mut self, x: usize, y: usize) -> Result<(), &'static str> {
	self.set(x, y, Point::Empty)
    }

    fn add_stone(&mut self, x: usize, y: usize, stone: Stone) -> Result<(), &'static str> {
	self.set(x, y, match stone {
	    Stone::Black => Point::Black,
	    Stone::White => Point::White
	})
    }

    fn capture_stones(&mut self, x: usize, y: usize) -> Result<i32, &'static str> {
	// (x, y) are coordinates of the last move.
	let player_point = self.get(x, y)?;
	if let Point::Empty = player_point {
	    return Err("Point of last move is empty!");
	}

	let opponent_point = opposite(player_point).ok_or("Point of last move is empty!")?;

	let neighbors = self.get_neighbors(x, y);
	let mut captures: i32 = 0;
	
	for n in neighbors {
	    if let Some((x, y)) = n {
		if opponent_point == self.get(x, y)? {
		    let (lib, group) = self.liberties(x, y)?;
		    if lib == 0 {
			captures += group.len() as i32;
			for (x, y) in group {
			    self.remove_stone(x, y)?;
			}
		    }
		}
	    }
	}

	Ok(captures)
    }

    fn is_suicide(&self, x: usize, y: usize) -> Result<bool, &'static str> {	
	let player_point = self.get(x, y)?;
	if let Point::Empty = player_point {
	    return Err("Can't check suicide. Point is empty!");
	}

	let (lib, _group) = self.liberties(x, y)?;
	Ok(lib == 0)
    }

    fn liberties(&self, x: usize, y: usize) -> Result<(i32, Vec<(usize, usize)>), &'static str> {
	let player_point = self.get(x, y)?; // Here player is the one liberties are calculated for.
	if let Point::Empty = player_point {
	    return Err("Can't calculate liberties. Point is empty!");
	}

	let opponent_point = opposite(player_point).ok_or("Can't calculate liberties. Point is empty!")?;

	let (group, perimeter) = self.spread(x, y)?;

	let mut lib = 0;
	for (x, y) in perimeter {
	    let p = self.get(x, y)?;
	    if p == opponent_point {
		// do nothing
	    } else if p == Point::Empty {
		lib += 1;
	    } else {
		println!("Algorithm 'spread' returned a wrong typed perimeter point. This should not have happened.");
	    }	    
	}

	Ok((lib, group))
    }

    fn get_neighbors(&self, x: usize, y: usize) -> [Option<(usize, usize)>; 4] {
	let left = if x > 0 { Some((x-1, y)) } else { None };
	let top  = if y > 0 { Some((x, y-1)) } else { None };
	let right  = if x < self.size - 1 { Some((x+1, y)) } else { None };
	let bottom = if y < self.size - 1 { Some((x, y+1)) } else { None };

	[left, right, top, bottom]
    }

    fn calculate_territory_score(&self) -> (i32, i32, i32) {
	let (black, white, neutral) = self.count_territories();

	// TODO: Consider Seki
	println!("In the current implementation of territory scoring, Seki is not checked.");

	(black, white, neutral)
    }

    fn calculate_area_score(&self) -> (i32, i32) {
	let (black_stones, white_stones) = self.count_stones();
	let (black_terr, white_terr, _neutral_terr) = self.count_territories();

	(black_stones + black_terr, white_stones + white_terr)
    }

    fn count_stones(&self) -> (i32, i32) {
	let mut black = 0;
	let mut white = 0;
	for x in 0..self.size {
	    for y in 0..self.size {
		match self.get(x, y).unwrap() {
		    Point::Black => { black += 1; },
		    Point::White => { white += 1; },
		    Point::Empty => ()
		}
	    }
	}
	(black, white)
    }

    fn count_territories(&self) -> (i32, i32, i32) {
	let mut visited = vec![vec![false; self.size]; self.size];
	let mut black: u32 = 0;
	let mut white: u32 = 0;
	let mut neutral: u32 = 0;

	for x in 0..self.size {
	    for y in 0..self.size {
		let p = self.get(x, y).unwrap();
		if p == Point::Empty && !visited[x][y] {
		    // Process a territory
		    let (area, perimeter) = self.spread(x, y).unwrap();

		    let territory = area.len() as u32;
		    for (x, y) in area {
			visited[x][y] = true;
		    }
		    
		    let mut black_perimeter = 0;
		    let mut white_perimeter = 0;
		    for (x, y) in perimeter {
			let p = self.get(x, y).unwrap();
			match p {
			    Point::Black => { black_perimeter += 1; },
			    Point::White => { white_perimeter += 1; },
			    Point::Empty => ()
			}
		    }
		    
		    if black_perimeter == 0 && white_perimeter == 0 {
			neutral += territory;
		    } else if black_perimeter > 0 && white_perimeter == 0 {
			black += territory;
		    } else if white_perimeter > 0 && black_perimeter == 0 {
			white += territory;
		    } else {
			neutral += territory;
		    }
		}
	    }
	}
	
	(black as i32, white as i32, neutral as i32)
    }

    fn spread(&self, x: usize, y: usize) -> Result<(Vec<(usize, usize)>, Vec<(usize, usize)>), &'static str> {
	let point = self.get(x, y)?;
	
	let mut visited = vec![vec![false; self.size]; self.size];
	let mut to_be_visited = VecDeque::new(); // Only points of type that are being spread are stored here.
	to_be_visited.push_back((x, y));
	let mut area = Vec::new();
	let mut perimeter = Vec::new();
	
	while !to_be_visited.is_empty() {
	    if let Some((x, y)) = to_be_visited.pop_front() {
	 	if !visited[x][y] { // this check is redundant but harmless
		    visited[x][y] = true;
		    area.push((x, y));
		    let neighbors = self.get_neighbors(x, y);
		    for n in neighbors {
			if let Some((x, y)) = n {
			    let p = self.get(x, y)?;
			    let v = visited[x][y];
			    if p == point {
				if !v {
				    to_be_visited.push_back((x, y));
				}
			    } else {
				if !v {
				    visited[x][y] = true;
				    perimeter.push((x, y));
				}
			    }
			}
		    }
		}
	    }
	}

	Ok((area, perimeter))
    }
}


impl History {
    fn new() -> Self {
	Self {
	    items: Vec::new()
	}
    }

    fn push(&mut self, item: HistoryItem) {
	self.items.push(item);
    }

    fn in_reverse(&self) -> Rev<std::slice::Iter<'_, HistoryItem>> {
	self.items.iter().rev()
    }
}


fn opposite(point: Point) -> Option<Point> {
    match point {
	Point::Black => Some(Point::White),
	Point::White => Some(Point::Black),
	Point::Empty => None
    }
}
