#[derive(Copy, Clone)]
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

#[derive(Copy, Clone)]
pub enum Turn {
    Black,
    White
}


#[derive(Clone)]
pub struct Model {
    board_size: usize,
    board: Vec<Vec<Point>>,
    turn: Turn,
}


impl Model {
    pub fn make_model(board_size: usize) -> Self {
	Self {
	    board_size: board_size,
	    board: make_empty_board(board_size),
	    turn: Turn::Black,
	}
    }

    pub fn get_board(&self) -> &Vec<Vec<Point>> {
	&self.board
    }

    pub fn get_board_size(&self) -> usize {
	self.board_size
    }

    pub fn get_turn(&self) -> Turn {
	self.turn
    }

    pub fn restart(&mut self) -> Result<(), &str> {
	self.board = make_empty_board(self.board_size);
	Ok(())
    }

    pub fn setup_switch_turn(&mut self) -> Result<(), &str> {
	match self.turn {
	    Turn::Black => {
		self.turn = Turn::White;
	    },
	    Turn::White => {
		self.turn = Turn::Black;
	    }
	}
	Ok(())
    }

    pub fn setup_add_stone(&mut self, x: usize, y: usize, stone: Stone) -> Result<(), &str> {
	let range = 0..self.board_size;
	if range.contains(&x) && range.contains(&y) {
	    self.board[x][y] = match stone {
		Stone::Black => Point::Black,
		Stone::White => Point::White
	    };
	    Ok(())
	} else {
	    Err("Indices out of range!")
	}
    }

    pub fn setup_remove_stone(&mut self, x: usize, y: usize) -> Result<(), &str> {
	let range = 0..self.board_size;
	if range.contains(&x) && range.contains(&y) {
	    self.board[x][y] = Point::Empty;
	    Ok(())
	} else {
	    Err("Indices out of range!")
	}
    }
}


fn make_empty_board(board_size: usize) -> Vec<Vec<Point>> {
    vec![vec![Point::Empty; board_size]; board_size]
}
