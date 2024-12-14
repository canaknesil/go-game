use crate::view::*;
use crate::model::{Model, Point};


pub struct CLView {
    model: Model
}


impl View for CLView {
    fn make(board_size: usize) -> Self {
	Self {
	    model: Model::make_model(board_size)
	}
    }

    fn run(&mut self) {
	let res = self.model.restart();
	if let Err(s) = res {
	    println!("Error at model restart: {s}");
	}

	self.display_init_msg("Wellcome to Go!");
	self.draw_board();
    }
}


impl CLView {
    fn display_init_msg(&self, msg: &str) {
	println!("{msg}");
    }

    fn draw_board(&self) {
	let board = self.model.get_board();
	
	for row in board.into_iter() {
	    for p in row.into_iter() {
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
