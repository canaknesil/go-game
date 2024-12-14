pub trait View {
    fn make(board_size: usize) -> Self;
    fn run(&mut self);
}

