use crate::model::Point as MPoint;
use crate::model::Model;
use crate::view::Point as VPoint;
use crate::view::{View, ControllerCallback};


pub struct Controller<V: View> {
    model: Model,
    view: V
}


impl<V: View> Controller<V> {
    pub fn make_controller(model: Model, view: V) -> Self {
	Self {model, view}
    }

    pub fn run(&self) {
	self.view.display_init_msg("Wellcome to Go!");
	self.draw_board();
	self.view.listen(self);
    }

    fn draw_board(&self) {
	let board = self.convert_board_to_view();
	self.view.draw_board(&board)
    }

    fn convert_board_to_view(&self) -> Vec<Vec<VPoint>> {
	self.model.get_board().into_iter().map(|inner_vec| {
	    inner_vec.into_iter().map(|x| {
		match x {
		    MPoint::Black => VPoint::Black,
		    MPoint::White => VPoint::White,
		    MPoint::Empty => VPoint::Empty,
		}
	    }).collect()
	}).collect()
    }

    fn send_command(&self, cmd: Vec<&str>) -> Result<(), &str> {
	println!("Controller received command: {cmd:?}");

	match cmd[..] {
	    [] => Err("Empty command!"),
	    ["start"] => self.start(),
	    _ => Err("Unrecognize command!")
	}
    }

    fn start(&self) -> Result<(), &str> {
	println!("Starting new game.");
	Ok(())
    }
}


impl<V: View> ControllerCallback for Controller<V> {
    fn send_command(&self, args: Vec<&str>) -> Result<(), &str> {
	self.send_command(args)
    }
}
