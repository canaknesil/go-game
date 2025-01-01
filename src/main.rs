mod cl_view;
mod egui_view;
mod model;
mod katago_installer;
mod smart_child;
mod gtp;
mod child_process_engine;

use crate::cl_view::CLView;
use crate::egui_view::EguiView;
use dirs;
    

enum ViewType {
    CL,
    Egui
}


fn main() -> Result<(), String> {
    // TODO: Consider std::error::Error for whole project, rather than String.

    println!("Starting Go.");

    // Setup directories to be used.
    let katago_install_dir = dirs::home_dir().ok_or("Error at home_dir function!".to_string())?
	.join(".cango").join("katago");
    

    // Start view.
    // TODO: Consider removing cl view from project. Is it currently lack behind of development. Replace it with a debugging feature.
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
	ViewType::CL => CLView::make().unwrap().run(),
	ViewType::Egui => EguiView::make(&katago_install_dir).unwrap().run()
    }

    println!("Exiting Go.");
    Ok(())
}
