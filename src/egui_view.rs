use crate::view::*;
use crate::model::{Model, Point, Stone, Turn};
use eframe::egui;


pub struct EguiView {
    workspaces: Vec<Workspace>,
    wspc: Option<usize>,
    prev_wspc: Option<usize>,
}

struct Workspace {
    name: String,
    model: Model,
    view_state: ViewState,
}

struct ViewState {
    mode: Mode,
    stone: Stone,
    black_territory_score: i32,
    white_territory_score: i32,
    black_area_score: i32,
    white_area_score: i32,
}


#[derive(PartialEq)]
enum Mode {
    Setup,
    Game
}


impl ViewState {
    fn default() -> Self {
	Self {
	    mode: Mode::Game,
	    stone: Stone::Black,
	    black_territory_score: 0,
	    white_territory_score: 0,
	    black_area_score: 0,
	    white_area_score: 0,
	}
    }
}


impl View for EguiView {
    fn make(board_size: usize) -> Result<Self, &'static str> {
	let model = Model::make_model(board_size);
	Ok(Self {
	    workspaces: vec![Workspace {
		name: "Game".to_string(),
		model: model,
		view_state: ViewState::default(),
	    }],
	    wspc: Some(0),
	    prev_wspc: None,
	})
    }

    fn run(self) {
	self.run_egui();
    }
}


impl EguiView {
    fn new_workspace(&mut self) {
	// TODO
    }

    fn clone_workspace(&mut self) {
	// TODO
    }
    
    fn change_workspace(&mut self, new_wspc: usize) {
	// TODO
    }

    fn quit_workspace(&mut self) {
	// TODO
    }

    fn get_workspace(&self) -> Option<&Workspace> {
	self.wspc.map(|n| &self.workspaces[n])
    }

    fn get_workspace_mut(&mut self) -> Option<&mut Workspace> {
	self.wspc.map(|n| &mut self.workspaces[n])
    }

    fn get_model(&self) -> Option<&Model> {
	self.get_workspace().map(|w| &w.model)
    }

    fn get_model_mut(&mut self) -> Option<&mut Model> {
	self.get_workspace_mut().map(|w| &mut w.model)
    }

    fn get_view_state(&self) -> Option<&ViewState> {
	self.get_workspace().map(|w| &w.view_state)
    }

    fn get_view_state_mut(&mut self) -> Option<&mut ViewState> {
	self.get_workspace_mut().map(|w| &mut w.view_state)
    }

    fn run_egui(mut self) {
	let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_inner_size([640.0, 480.0]),
            ..Default::default()
	};

	// Moving self to the following closure.
	eframe::run_simple_native("Go", options, move |ctx, frame| {
	    // TODO: new workspace pop up selection UI
	    // TODO: maybe no default workspace ???

	    egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
		egui::menu::bar(ui, |ui| {
		    ui.menu_button("Go", |ui| {
			
		    });
		    ui.menu_button("Workspace", |ui| {
			if ui.button("New workspace").clicked() {
			    self.new_workspace();
			    ui.close_menu();
			}
			if ui.button("Clone workspace").clicked() {
			    self.clone_workspace();
			    ui.close_menu();
			}
			if ui.button("Quit workspace").clicked() {
			    self.quit_workspace();
			    ui.close_menu();
			}
		    });
		});
		ui.horizontal(|ui| {
		    ui.label("Workspace:");
		    for n in 0..self.workspaces.len() {
			let name = self.workspaces[n].name.clone();
			ui.radio_value(&mut self.wspc, Some(n), name);
		    }
		});
	    });
	    
	    egui::SidePanel::left("side_panel").resizable(false).exact_width(200.0).show(ctx, |ui| {
		if let Some(_) = self.wspc {
		    if let Some(vs) = self.get_view_state_mut() {
			let mode = &mut vs.mode;
			ui.label("Mode:");
			ui.radio_value(mode, Mode::Setup, "Setup");
			ui.radio_value(mode, Mode::Game, "Game");
		    }

		    if let Some(model) = self.get_model() {
			match model.get_turn() {
			    Turn::Black => {
				ui.label("Turn: Black");
			    },
			    Turn::White => {
				ui.label("Turn: White");
			    }
			}
		    }

		    if let Some(vs) = self.get_view_state() {
			match vs.mode {
			    Mode::Setup => {
				if ui.add(egui::Button::new("Switch turn")).clicked() {
				    if let Some(model) = self.get_model_mut() {
					let r = model.setup_switch_turn();
					if let Err(s) = r {
					    println!("Model setup_switch_turn unsuccessful! {s}");
					}
				    }
				}

				if let Some(vs) = self.get_view_state_mut() {
				    let stone = &mut vs.stone;
				    ui.label("Put stone:");
				    ui.radio_value(stone, Stone::Black, "Black");
				    ui.radio_value(stone, Stone::White, "White");
				}
			    },
			    Mode::Game => {
				if let Some(model) = self.get_model() {
				    ui.label(format!("Black captures: {}", model.get_black_captures()));
				    ui.label(format!("White captures: {}", model.get_white_captures()));
				}

				if let Some(w) = self.get_workspace_mut() {
				    if ui.add(egui::Button::new("Calculate score")).clicked() {
					let (ts_black, ts_white) = w.model.calculate_territory_score();
					let (as_black, as_white) = w.model.calculate_area_score();
					w.view_state.black_territory_score = ts_black;
					w.view_state.white_territory_score = ts_white;
					w.view_state.black_area_score = as_black;
					w.view_state.white_area_score = as_white;
				    }
				}

				if let Some(vs) = self.get_view_state() {
				    ui.label(format!("Black territory score: {}", vs.black_territory_score));
				    ui.label(format!("White territory score: {}", vs.white_territory_score));
				    ui.label(format!("Black area score: {}", vs.black_area_score));
				    ui.label(format!("White area score: {}", vs.white_area_score));
				}

				if let Some(model) = self.get_model_mut() {
				    if ui.add(egui::Button::new("Undo")).clicked() {
					let _ = model.undo();
				    }
				}
			    }
			}
		    }
		}
	    });

            egui::CentralPanel::default().show(ctx, |ui| {
		if let Some(_) = self.wspc {
		    let painter = ui.painter();
		    let rect = ui.max_rect();
		    
		    //
		    // DRAW BOARD
		    //

		    let board_info = self.get_model().map(|model| {
			// Assume a drawing canvas.
			let board_size = model.get_board_size();
			let margin_cell_size_ratio = 0.8;
			let canvas_ratio = (board_size as f32 + 2.0 * margin_cell_size_ratio + 2.0) // Additional space of size cell_size on the right
			    / (board_size as f32 + 2.0 * margin_cell_size_ratio);

			let (canvas_width, canvas_height, canvas_top_left) = if rect.width() / rect.height() > canvas_ratio {
			    // bigger width
			    let h = rect.height();
			    let w = h * canvas_ratio;
			    (w, h, egui::Pos2::new(rect.left() + (rect.width() - w) / 2.0, rect.top()))
			} else {
			    // bigger height
			    let w = rect.width();
			    let h = w / canvas_ratio;
			    (w, h, egui::Pos2::new(rect.left(), rect.top() + (rect.height() - h) / 2.0))
			};

			let canvas = egui::Rect::from_min_size(canvas_top_left, egui::Vec2::new(canvas_width, canvas_height));
			painter.rect_filled(canvas, 0.0, egui::Color32::from_rgb(180, 180, 180));

			// Draw board
			let size = canvas_height;
			let board_center = canvas_top_left + egui::Vec2::new(size / 2.0, size / 2.0);
			let cell_size = canvas_height / (board_size as f32 + 2.0 * margin_cell_size_ratio);

			let square_a = egui::Rect::from_center_size(board_center, egui::Vec2::splat(size));
			painter.rect_filled(square_a, 0.0, egui::Color32::from_rgb(225, 225, 130));
			
			// Create square B with a margin inside square A
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
			let stone_radius_ratio = 0.4; // 0.5 makes the stones touch
			let stone_radius = cell_size * stone_radius_ratio;
			let black_stone_color = egui::Color32::from_rgb(0, 0, 0);
			let white_stone_color = egui::Color32::from_rgb(255, 255, 255);

			for col_idx in 0..board_size {
			    for row_idx in 0..board_size {
				let point = board.get(col_idx, row_idx).unwrap();
				let stone_color = match point {
				    Point::Black => Some(black_stone_color), // Black stones
				    Point::White => Some(white_stone_color), // White stones
				    Point::Empty => None, // No stone
				};

				if let Some(color) = stone_color {
				    let x = square_b.left() + col_idx as f32 * cell_size;
				    let y = square_b.top() + row_idx as f32 * cell_size;

				    let center = egui::Pos2::new(x, y);
				    painter.circle_filled(center, stone_radius, color);
				}
			    }
			}

			// Draw area on the right of the board
			let turn = model.get_turn();
			let turn_stone_center = egui::Pos2::new(square_a.right() + cell_size, canvas.top() + canvas_height / 2.0);
			let turn_stone_color = match turn {
			    Turn::Black => black_stone_color,
			    Turn::White => white_stone_color
			};
			painter.circle_filled(turn_stone_center, stone_radius, turn_stone_color);

			(board_origin, cell_size)
		    });
			
		    //
		    // GET MOUSE INPUT
		    //

		    // Handle clicks
		    if let Some((board_origin, cell_size)) = board_info {
			if ui.rect_contains_pointer(rect) {
			    if ui.input(|i| i.pointer.primary_clicked()) {
				if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
				    let (x, y) = board_coordinates(pos, board_origin, cell_size);
				    if let Some(vs) = self.get_view_state() {
					match vs.mode {
					    Mode::Setup => {
						let stone = vs.stone;
						if let Some(model) = self.get_model_mut() {
						    let r = model.setup_add_stone(x as usize, y as usize, stone);
						    if let Err(s) = r {
							println!("Model setup_add_stone unsuccessful! {s}");
						    }
						}
					    },
					    Mode::Game => {
						if let Some(model) = self.get_model_mut() {
						    let r = model.make_move(x as usize, y as usize);
						    if let Err(s) = r {
							println!("Model make_move unsuccessful! {s}");
						    }
						}
					    }
					}
				    }
				}
			    }
			    if ui.input(|i| i.pointer.secondary_clicked()) {
				if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
				    let (x, y) = board_coordinates(pos, board_origin, cell_size);
				    if let Some(vs) = self.get_view_state() {
					match vs.mode {
					    Mode::Setup => {
						if let Some(model) = self.get_model_mut() {
						    let r = model.setup_remove_stone(x as usize, y as usize);
						    if let Err(s) = r {
							println!("Model setup_remove_stone unsuccessful! {s}");
						    }
						}
					    },
					    Mode::Game => ()
					}
				    }
				}
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
