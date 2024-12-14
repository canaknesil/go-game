mod model;
mod view;
mod cl_view;
mod egui_view;

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

    match view_type {
	ViewType::CL => CLView::make(board_size).run(),
	ViewType::Egui => EguiView::make(board_size).run()
    }

    println!("Exiting Go.");
}
