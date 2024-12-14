use std::collections::VecDeque;


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

#[derive(Clone)]
pub struct Board {
    matrix: Vec<Vec<Point>>,
    size: usize
}

#[derive(Clone)]
pub struct Model {
    board: Board,
    turn: Turn,
}


impl Model {
    pub fn make_model(board_size: usize) -> Self {
	let mut model = Self {
	    board: Board::make_empty_board(board_size),
	    turn: Turn::Black
	};
	let _ = model.restart();
	model
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

    pub fn restart(&mut self) -> Result<(), &str> {
	self.board = Board::make_empty_board(self.board.size);
	Ok(())
    }

    pub fn setup_switch_turn(&mut self) -> Result<(), &str> {
	self.switch_turn();
	Ok(())
    }

    fn switch_turn(&mut self) {
	self.turn = match self.turn {
	    Turn::Black => Turn::White,
	    Turn::White => Turn::Black
	};
    }

    pub fn setup_add_stone(&mut self, x: usize, y: usize, stone: Stone) -> Result<(), &str> {
	self.board.add_stone(x, y, stone)
    }

    pub fn setup_remove_stone(&mut self, x: usize, y: usize) -> Result<(), &str> {
	self.board.remove_stone(x, y)
    }

    pub fn make_move(&mut self, x: usize, y: usize) -> Result<(), &'static str> {
	let point = self.board.get(x, y)?;
	if point == Point::Empty {
	    let mut new_board = self.board.clone();
	    new_board.set(x, y, match self.turn {
		Turn::Black => Point::Black,
		Turn::White => Point::White
	    })?;
	    new_board.capture_stones(x, y)?;
	    if new_board.is_suicide(x, y)? {
		Err("Suicide!")
	    } else if self.is_repetition(&new_board) {
		Err("Repetition!")
	    } else {
		self.board = new_board.clone();
		self.switch_turn();
		Ok(())
	    }
	} else {
	    Err("Point is not empty!")
	}
    }

    fn is_repetition(&self, board: &Board) -> bool {
	// TODO: is_repetition
	false
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

    fn capture_stones(&mut self, x: usize, y: usize) -> Result<(), &'static str> {
	// (x, y) are coordinates of the last move.
	let player_point = self.get(x, y)?;
	if let Point::Empty = player_point {
	    return Err("Point of last move is empty!");
	}

	let opponent_point = opposite(player_point).ok_or("Point of last move is empty!")?;

	let neighbors = self.get_neighbors(x, y);
	for n in neighbors {
	    if let Some((x, y)) = n {
		if opponent_point == self.get(x, y)? {
		    let (lib, group) = self.liberties(x, y)?;
		    if lib == 0 {
			for (x, y) in group {
			    self.remove_stone(x, y)?;
			}
		    }
		}
	    }
	}

	Ok(())
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

	let mut visited = vec![vec![false; self.size]; self.size];
	let mut to_be_visited = VecDeque::new(); // Only players points are stored here.
	to_be_visited.push_back((x, y));
	let mut group = Vec::new();
	let mut lib = 0;
	
	while !to_be_visited.is_empty() {
	    if let Some((x, y)) = to_be_visited.pop_front() {
		if !visited[x][y] { // this check is redundant but harmless
		    visited[x][y] = true;
		    group.push((x, y));
		    let neighbors = self.get_neighbors(x, y);
		    for n in neighbors {
			if let Some((x, y)) = n {
			    let p = self.get(x, y)?;
			    let v = visited[x][y];
			    if p == player_point {
				if !v {
				    to_be_visited.push_back((x, y));
				}
			    } else if p == opponent_point {
				// do nothing
			    } else {
				if !v {
				    visited[x][y] = true;
				    lib += 1;
				}
			    }
			}
		    }
		}
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
}

fn opposite(point: Point) -> Option<Point> {
    match point {
	Point::Black => Some(Point::White),
	Point::White => Some(Point::Black),
	Point::Empty => None
    }
}
