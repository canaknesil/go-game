pub trait View {
    fn make() -> Result<Self, String>  where Self: Sized;
    fn run(self);
}

