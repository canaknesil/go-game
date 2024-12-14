use crate::view::*;
use crate::model::{Model, Point, Stone, Turn};
use eframe::egui;



pub struct EguiView {
    board_size: usize
}

#[derive(PartialEq)]
enum Mode {
    Setup,
    Game
}


impl View for EguiView {
    fn make(board_size: usize) -> Self {
	Self {
	    board_size
	}
    }

    fn run(&mut self) {
	let mut model = Model::make_model(self.board_size);
	self.init_model(&mut model);
	self.run_egui(model);
    }
}


impl EguiView {
    fn init_model(&mut self, model: &mut Model) {
	let res = model.restart();
	if let Err(s) = res {
	    println!("Error at model restart: {s}");
	}
    }

    fn run_egui(&self, mut model: Model) {
	let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_inner_size([640.0, 480.0]),
            ..Default::default()
	};

	// View state
	let mut mode = Mode::Setup;
	let mut stone = Stone::Black;

	eframe::run_simple_native("Go", options, move |ctx, _frame| {
	    egui::SidePanel::left("side_panel").show(ctx, |ui| {
		ui.label("Mode:");
		ui.radio_value(&mut mode, Mode::Setup, "Setup");
		ui.radio_value(&mut mode, Mode::Game, "Game");

		let turn = model.get_turn();
		match turn {
		    Turn::Black => {
			ui.label("Turn: Black");
		    },
		    Turn::White => {
			ui.label("Turn: White");
		    }
		}

		match mode {
		    Mode::Setup => {
			if ui.add(egui::Button::new("Switch turn")).clicked() {
			    let r = model.setup_switch_turn();
			    if let Err(s) = r {
				println!("Model setup_switch_turn unsuccessful! {s}");
			    }
			}

			ui.label("Put stone:");
			ui.radio_value(&mut stone, Stone::Black, "Black");
			ui.radio_value(&mut stone, Stone::White, "White");
		    },
		    Mode::Game => {
		    }
		}
	    });
	    
            egui::CentralPanel::default().show(ctx, |ui| {
		//
		// DRAW
		//
		
		let painter = ui.painter();
		let rect = ui.max_rect();

		// Find the largest square A within the rect
		let size = rect.width().min(rect.height());
		let square_a = egui::Rect::from_center_size(rect.center(), egui::Vec2::splat(size));
		painter.rect_filled(square_a, 0.0, egui::Color32::from_rgb(225, 225, 130));
		
		// Create square B with a margin inside square A
		let board_size = model.get_board_size();
		let margin_cell_size_ratio = 0.8;
		let cell_size = square_a.width() / (board_size as f32 - 1.0 + 2.0 * margin_cell_size_ratio);
		let margin = cell_size * margin_cell_size_ratio;
		let square_b = square_a.shrink(margin);
		let board_origin = square_b.left_top();

		// Draw a Go board inside square B
		let cell_size = square_b.width() / (board_size as f32 - 1.0);
		
		for i in 0..board_size {
                    // Horizontal lines
                    let y = square_b.top() + i as f32 * cell_size;
                    painter.line_segment([
			egui::Pos2::new(square_b.left(), y),
			egui::Pos2::new(square_b.right(), y),
                    ], egui::Stroke::new(1.0, egui::Color32::BLACK));
		    
                    // Vertical lines
                    let x = square_b.left() + i as f32 * cell_size;
                    painter.line_segment([
			egui::Pos2::new(x, square_b.top()),
			egui::Pos2::new(x, square_b.bottom()),
                    ], egui::Stroke::new(1.0, egui::Color32::BLACK));
		}

		// Draw stones
		let board = model.get_board();
		
		for (col_idx, col) in board.iter().enumerate() {
		    for (row_idx, point) in col.iter().enumerate() {
			let stone_color = match point {
			    Point::Black => Some(egui::Color32::from_rgb(0, 0, 0)), // Black stones
			    Point::White => Some(egui::Color32::from_rgb(255, 255, 255)), // White stones
			    Point::Empty => None, // No stone
			};

			if let Some(color) = stone_color {
			    let x = square_b.left() + col_idx as f32 * cell_size;
			    let y = square_b.top() + row_idx as f32 * cell_size;

			    let center = egui::Pos2::new(x, y);
			    let radius = cell_size * 0.4;

			    painter.circle_filled(center, radius, color);
			}
		    }
		}

		//
		// GET INPUT
		//

		// Handle clicks
		if ui.rect_contains_pointer(rect) {
		    if ui.input(|i| i.pointer.primary_clicked()) {
			if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
			    let (x, y) = board_coordinates(pos, board_origin, cell_size);
			    match mode {
				Mode::Setup => {
				    let r = model.setup_add_stone(x as usize, y as usize, stone);
				    if let Err(s) = r {
					println!("Model setup_add_stone unsuccessful! {s}");
				    }
				},
				Mode::Game => ()
			    }
			}
		    }
		    if ui.input(|i| i.pointer.secondary_clicked()) {
			if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
			    let (x, y) = board_coordinates(pos, board_origin, cell_size);
			    match mode {
				Mode::Setup => {
				    let r = model.setup_remove_stone(x as usize, y as usize);
				    if let Err(s) = r {
					println!("Model setup_remove_stone unsuccessful! {s}");
				    }
				},
				Mode::Game => ()
			    }
			}
		    }
		}
	    });
	}).unwrap();
    }
}


fn board_coordinates(pos: egui::Pos2, origin: egui::Pos2, cell_size: f32) -> (i32, i32) {
    let pos = (pos - origin) / cell_size;
    let x = pos.x.round() as i32;
    let y = pos.y.round() as i32;
    (x, y)
}
