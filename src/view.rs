use crate::model::Model;


pub trait View {
    fn make(model: Model) -> Self;
    fn run(&mut self);
}

