#[derive(Clone)]
pub enum Point {
    Black,
    White,
    Empty
}

pub trait View {
    fn display_init_msg(&self, msg: &str);
    fn draw_board(&self, board: &Vec<Vec<Point>>);
    fn listen(&self, controller: &impl ControllerCallback);
}


pub trait ControllerCallback {
    fn send_command(&self, args: Vec<&str>) -> Result<(), &str>;
}
