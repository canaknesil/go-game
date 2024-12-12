#[derive(Clone)]
pub enum Point {
    Black,
    White,
    Empty
}

pub struct Model {
    board: Vec<Vec<Point>>
}

impl Model {
    pub fn make_model(board_size: usize) -> Self {
	Self {
	    board: vec![vec![Point::Empty; board_size]; board_size]
	}
    }

    pub fn get_board(&self) -> &Vec<Vec<Point>> {
	&self.board
    }
}
