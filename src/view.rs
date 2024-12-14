pub trait View {
    fn make(board_size: usize) -> Result<Self, &'static str>  where Self: Sized;
    fn run(self);
}

