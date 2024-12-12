use crate::view::*;

pub struct CLView {
    board_size: usize,
}

impl CLView {
    pub fn make_cl_view(board_size: usize) -> Self {
	Self {
	    board_size
	}
    }
}

impl View for CLView {
    fn display_init_msg(&self, msg: &str) {
	println!("{msg}");
    }

    fn draw_board(&self, board: &Vec<Vec<Point>>) {
	for row in board.into_iter() {
	    for p in row.into_iter() {
		let sign = match p {
		    Point::Black => '◯',
		    Point::White => '⬤',
		    Point::Empty => '+'
		};
		print!("{sign}");
	    }
	    println!();
	}
    }

    fn listen(&self, controller: &impl ControllerCallback) {
	let res = controller.send_command(vec!["start"]);
	if let Err(s) = res {
	    println!("Error at controller callback: {s}");
	}
    }
}
