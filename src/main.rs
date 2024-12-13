mod model;
mod view;
mod cl_view;
mod egui_view;

use crate::model::Model;
use crate::view::View;
use crate::cl_view::CLView;
use crate::egui_view::EguiView;


enum ViewType {
    CL,
    Egui
}


fn main() {
    println!("Starting Go.");

    let board_size = 9;
    let view_type = ViewType::Egui;

    let model = Model::make_model(board_size);

    match view_type {
	ViewType::CL => CLView::make(model).run(),
	ViewType::Egui => EguiView::make(model).run()
    }

    println!("Exiting Go.");
}
