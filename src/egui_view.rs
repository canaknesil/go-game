use crate::view::*;
use crate::model::{Model, Point, Stone, Turn};
use eframe::egui;



pub struct EguiView {
    model: Model
}

#[derive(PartialEq)]
enum Mode {
    Setup,
    Game
}


impl View for EguiView {
    fn make(board_size: usize) -> Result<Self, &'static str> {
	let model = Model::make_model(board_size);
	Ok(Self {
	    model
	})
    }

    fn run(self) {
	self.run_egui();
    }
}


impl EguiView {
    fn run_egui(mut self) {
	let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_inner_size([640.0, 480.0]),
            ..Default::default()
	};

	// View state
	let mut mode = Mode::Game;
	let mut stone = Stone::Black;
	let mut black_territory_score = 0;
	let mut white_territory_score = 0;
	let mut black_area_score = 0;
	let mut white_area_score = 0;

	eframe::run_simple_native("Go", options, move |ctx, _frame| {
	    // TODO: menu
	    
	    egui::SidePanel::left("side_panel").show(ctx, |ui| {
		ui.label("Mode:");
		ui.radio_value(&mut mode, Mode::Setup, "Setup");
		ui.radio_value(&mut mode, Mode::Game, "Game");

		let turn = self.model.get_turn();
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
			    let r = self.model.setup_switch_turn();
			    if let Err(s) = r {
				println!("Model setup_switch_turn unsuccessful! {s}");
			    }
			}

			ui.label("Put stone:");
			ui.radio_value(&mut stone, Stone::Black, "Black");
			ui.radio_value(&mut stone, Stone::White, "White");
		    },
		    Mode::Game => {
			ui.label(format!("Black captures: {}", self.model.get_black_captures()));
			ui.label(format!("White captures: {}", self.model.get_white_captures()));

			if ui.add(egui::Button::new("Calculate score")).clicked() {
			    (black_territory_score, white_territory_score) = self.model.calculate_territory_score();
			    (black_area_score, white_area_score) = self.model.calculate_area_score();
			}
			ui.label(format!("Black territory score: {}", black_territory_score));
			ui.label(format!("White territory score: {}", white_territory_score));
			ui.label(format!("Black area score: {}", black_area_score));
			ui.label(format!("White area score: {}", white_area_score));
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
		let board_size = self.model.get_board_size();
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
		let board = self.model.get_board();

		for col_idx in 0..board_size {
		    for row_idx in 0..board_size {
			let point = board.get(col_idx, row_idx).unwrap();
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
				    let r = self.model.setup_add_stone(x as usize, y as usize, stone);
				    if let Err(s) = r {
					println!("Model setup_add_stone unsuccessful! {s}");
				    }
				},
				Mode::Game => {
				    let r = self.model.make_move(x as usize, y as usize);
				    if let Err(s) = r {
					println!("Model make_move unsuccessful! {s}");
				    }
				}
			    }
			}
		    }
		    if ui.input(|i| i.pointer.secondary_clicked()) {
			if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
			    let (x, y) = board_coordinates(pos, board_origin, cell_size);
			    match mode {
				Mode::Setup => {
				    let r = self.model.setup_remove_stone(x as usize, y as usize);
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
