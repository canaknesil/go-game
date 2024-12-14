#[derive(Clone)]
pub enum Point {
    Black,
    White,
    Empty
}


#[derive(Clone)]
pub struct Model {
    board_size: usize,
    board: Vec<Vec<Point>>
}


impl Model {
    pub fn make_model(board_size: usize) -> Self {
	Self {
	    board_size: board_size,
	    board: make_empty_board(board_size)
	}
    }

    pub fn get_board(&self) -> &Vec<Vec<Point>> {
	&self.board
    }

    pub fn get_board_size(&self) -> usize {
	self.board_size
    }

    pub fn restart(&mut self) -> Result<(), &str> {
	self.board = make_empty_board(self.board_size);
	Ok(())
    }
}


fn make_empty_board(board_size: usize) -> Vec<Vec<Point>> {
    vec![vec![Point::Empty; board_size]; board_size]
}
