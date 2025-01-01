use crate::model::{Model, Point};


pub struct CLView {
    model: Model
}


impl CLView {
    pub fn make() -> Result<Self, String> {
	let board_size = 13;
	let model = Model::make_model(board_size, None, None);
	Ok(Self {
	    model
	})	
    }

    pub fn run(self) {
	self.display_init_msg("Wellcome to Go!");
	self.draw_board();
    }

    fn display_init_msg(&self, msg: &str) {
	println!("{msg}");
    }

    fn draw_board(&self) {
	let board = self.model.get_board();
	let board_size = self.model.get_board_size();

	for x in 0..board_size {
	    for y in 0..board_size {
		let p = board.get(x, y).unwrap();
		let sign = match p {
		    Point::Black => "◯ ",
		    Point::White => "⬤ ",
		    Point::Empty => "+ "
		};
		print!("{sign}");
	    }
	    println!();
	}
    }
}
