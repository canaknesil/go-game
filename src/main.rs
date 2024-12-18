mod view;
mod cl_view;
mod egui_view;
mod model;
mod gtp;
mod child_process_engine;

use crate::view::View;
use crate::cl_view::CLView;
use crate::egui_view::EguiView;


enum ViewType {
    CL,
    Egui
}


fn main() {
    println!("Starting Go.");

    let board_size = 13;
    let view_type = "egui";

    let view_type = match view_type {
	"cl" => ViewType::CL,
	"egui" => ViewType::Egui,
	other => {
	    println!("View type {other} is not supported. Selecting cl.");
	    ViewType::CL
	}
    };

    match view_type {
	ViewType::CL => CLView::make(board_size).unwrap().run(),
	ViewType::Egui => EguiView::make(board_size).unwrap().run()
    }

    println!("Exiting Go.");
}
