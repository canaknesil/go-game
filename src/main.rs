mod egui_view;
mod model;
mod katago_installer;
mod smart_child;
mod smart_thread;
mod gtp;
mod child_process_engine;

use crate::egui_view::EguiView;
use dirs;
    

fn main() -> Result<(), String> {
    println!("Starting Go.");

    // Setup directories to be used.
    let katago_install_dir = dirs::home_dir().ok_or("Error at home_dir function!".to_string())?
	.join(".cango").join("katago");
    

    // Start view.
    EguiView::make(&katago_install_dir).unwrap().run();

    println!("Exiting Go.");
    Ok(())
}
