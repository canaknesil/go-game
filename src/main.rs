mod model;
mod view;
mod cl_view;
mod controller;

use crate::model::Model;
use crate::cl_view::CLView;
use crate::controller::Controller;

fn main() {
    println!("Starting Go.");

    let board_size = 9;

    let model = Model::make_model(board_size);
    let view = CLView::make_cl_view(board_size);
    let controller = Controller::make_controller(model, view);

    controller.run();

    println!("Exiting Go.");
}
