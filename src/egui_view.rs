use crate::view::*;
use crate::model::{Model, Point};
use eframe::egui;



pub struct EguiView {
    board_size: usize
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

    fn run_egui(&self, model: Model) {
	let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_inner_size([640.0, 480.0]),
            ..Default::default()
	};

	eframe::run_simple_native("Go", options, move |ctx, _frame| {
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
		
		for (row_idx, row) in board.iter().enumerate() {
		    for (col_idx, point) in row.iter().enumerate() {
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

		// TODO

		//
		// UPDATE MODEL
		//

		// TODO
	    });
	}).unwrap();
    }
}
