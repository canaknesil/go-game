use crate::model::{Model, Point, Stone, Turn};
use crate::katago_installer::*;
use eframe::egui;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;


pub struct EguiView {
    workspaces: Vec<Workspace>,
    wspc: Option<usize>,
    mode: ViewMode,
    new_workspace_setup: NewWorkspaceSetup,
    katago_installer: KataGoInstaller,
    // KataGo installer is internally protected from mutual
    // execution. The view provides a second layer of protection for
    // its own purposes.
    katago_installer_status: Arc<Mutex<KataGoInstallerStatus>>,
}

struct Workspace {
    name: String,
    model: Model,
    mode: WorkspaceMode,
    game_mode: GameMode,
    stone: Stone,
    black_territory_score: i32,
    white_territory_score: i32,
    black_area_score: i32,
    white_area_score: i32,
    // TODO: Mark last move
}

struct KataGoInstallerStatus {
    is_installed: Option<bool>,
    is_operational: Option<bool>,
    is_tuned: Option<bool>,
}

#[derive(Copy, Clone, PartialEq)]
enum ViewMode {
    Workspace,
    CreateWorkspace,
    InstallEngine,
}

struct NewWorkspaceSetup {
    board_size: usize,
    count: usize,
    game_mode: GameMode,
}

#[derive(Copy, Clone, PartialEq)]
enum WorkspaceMode {
    Setup,
    Game,
    Analysis,
}

#[derive(Copy, Clone, PartialEq)]
enum GameMode {
    HumanVsHuman,
    HumanVsComputer(Turn), // turn is the human's color
}


impl Clone for Workspace {
    fn clone(&self) -> Self {
	let mut new_name = self.name.clone();
	new_name.push_str("-Clone");

	Self {
	    name: new_name,
	    model: self.model.clone(),
	    mode: self.mode,
	    game_mode: self.game_mode,
	    stone: self.stone,
	    black_territory_score: self.black_territory_score,
	    white_territory_score: self.white_territory_score,
	    black_area_score: self.black_area_score,
	    white_area_score: self.white_area_score,
	}
    }
}


impl NewWorkspaceSetup {
    fn default() -> Self {
	Self {
	    board_size: 13,
	    count: 0,
	    game_mode: GameMode::HumanVsHuman,
	}
    }
}


impl EguiView {
    pub fn make(katago_install_dir: &Path) -> Result<Self, String> {
	let katago_installer = KataGoInstaller::new(katago_install_dir);
	let katago_installer_status = KataGoInstallerStatus {
	    is_installed: Some(katago_installer.is_installed()),
	    is_operational: None, // Checking this probably takes several seconds, not doing here.
	    is_tuned: Some(katago_installer.is_tuned()),
	};

	let view = Self {
	    workspaces: Vec::new(),
	    wspc: None,
	    mode: ViewMode::CreateWorkspace,
	    new_workspace_setup: NewWorkspaceSetup::default(),
	    katago_installer: katago_installer,
	    katago_installer_status: Arc::new(Mutex::new(katago_installer_status)),
	};
	Ok(view)
    }

    pub fn run(self) {
	self.run_egui();
    }

    fn new_workspace(&mut self) {
	// Make the workspace
	let model = Model::make_model(self.new_workspace_setup.board_size);
	self.new_workspace_setup.count += 1;

	let mut w = Workspace {
	    name: format!("W{}", self.new_workspace_setup.count),
	    model: model,
	    mode: WorkspaceMode::Game,
	    game_mode: self.new_workspace_setup.game_mode,
	    stone: Stone::Black,
	    black_territory_score: 0,
	    white_territory_score: 0,
	    black_area_score: 0,
	    white_area_score: 0,
	};

	// If computer is black, make the first move.
	if let GameMode::HumanVsComputer(Turn::White) = w.game_mode {
	    let r = w.model.make_move_computer();
	    if let Err(s) = r {
		println!("Model make_move_computer unsuccessful! {s}");
	    }
	}

	// Add the workspace to the view.
	if let Some(n) = self.wspc {
	    self.workspaces.insert(n+1, w);
	    self.wspc = Some(n + 1);
	} else {
	    self.workspaces.push(w);
	    self.wspc = Some(0);
	}
    }

    fn clone_workspace(&mut self) {
	if let Some(n) = self.wspc {
	    let w = &self.workspaces[n];
	    self.workspaces.insert(n+1, w.clone());
	    self.wspc = Some(n + 1);
	} else {
	    println!("Cannot clone workspace! No workspace to clone or no workspace is selected.");
	}
    }
    
    fn quit_workspace(&mut self) {
	if let Some(n) = self.wspc {
	    if self.workspaces.len() == 1 {
		self.wspc = None;
	    } else if n == 0 {
		// do nothing
	    } else {
		self.wspc = Some(n - 1);
	    }
	    
	    let _ = self.workspaces.remove(n);
	} else {
	    println!("Cannot quit workspace! No workspace to clone or no workspace is selected.");
	}
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

    fn run_egui(mut self) {
	let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_inner_size([640.0, 480.0]),
            ..Default::default()
	};

	// Moving self to the following closure.
	eframe::run_simple_native("Go", options, move |ctx, _frame| {
	    self.draw_top_panel(ctx);
	    match self.mode {
		ViewMode::Workspace => {
		    if let Some(_) = self.wspc {
			self.draw_workspace_side_panel(ctx);
			self.draw_workspace_central_panel(ctx);
		    }
		},
		ViewMode::CreateWorkspace => {
		    self.draw_create_workspace_central_panel(ctx);
		},
		ViewMode::InstallEngine => {
		    self.draw_install_engine_central_panel(ctx);
		},
	    }
	}).unwrap();
    }

    fn draw_top_panel(&mut self, ctx: &egui::Context) {
	egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
	    egui::menu::bar(ui, |ui| {
		// ui.menu_button("Go", |ui| {
		
		// });
		ui.menu_button("Workspace", |ui| {
		    if ui.button("New workspace").clicked() {
			self.mode = ViewMode::CreateWorkspace;
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
		ui.menu_button("Engine", |ui| {
		    if ui.button("Setup engine").clicked() {
			self.mode = ViewMode::InstallEngine;
			ui.close_menu();
		    }
		});
	    });
	    ui.horizontal(|ui| {
		ui.label("Workspace:");
		for n in 0..self.workspaces.len() {
		    let name = self.workspaces[n].name.clone();
		    ui.selectable_value(&mut self.wspc, Some(n), name);
		}
	    });
	});
    }

    fn draw_workspace_side_panel(&mut self, ctx: &egui::Context) {
	// TODO: Don't provide some analysis features and human-vs-computer features if engine is not setup.
	egui::SidePanel::left("side_panel").resizable(false).exact_width(200.0).show(ctx, |ui| {
	    // Workspace mode selection
	    if let Some(w) = self.get_workspace_mut() {
		let mode = &mut w.mode;
		ui.label("Mode:");
		ui.radio_value(mode, WorkspaceMode::Setup, "Setup");
		ui.radio_value(mode, WorkspaceMode::Game, "Game");
		ui.radio_value(mode, WorkspaceMode::Analysis, "Analysis");
	    }
	    ui.separator();

	    // Widgets common to all workspace modes
	    if let Some(model) = self.get_model() {
		match model.get_turn() {
		    Turn::Black => {
			ui.label("Turn: Black");
		    },
		    Turn::White => {
			ui.label("Turn: White");
		    }
		}

		ui.label(format!("Black captures: {}", model.get_black_captures()));
		ui.label(format!("White captures: {}", model.get_white_captures()));
	    }

	    if let Some(w) = self.get_workspace() {
		let s = match w.game_mode {
		    GameMode::HumanVsHuman => "Human vs. human",
		    GameMode::HumanVsComputer(Turn::Black) => "Human (black) vs. computer (white)",
		    GameMode::HumanVsComputer(Turn::White) => "Human (white) vs. computer (black)",
		};
		ui.label(s);
	    }
	    ui.separator();

	    // Widgets specific to setup mode
	    if let Some(w) = self.get_workspace() {
		if let WorkspaceMode::Setup = w.mode {
		    if ui.button("Switch turn").clicked() {
			if let Some(model) = self.get_model_mut() {
			    let r = model.setup_switch_turn();
			    if let Err(s) = r {
				println!("Model setup_switch_turn unsuccessful! {s}");
			    }
			}
		    }
		    
		    if let Some(w) = self.get_workspace_mut() {
			let stone = &mut w.stone;
			ui.label("Put stone:");
			ui.radio_value(stone, Stone::Black, "Black");
			ui.radio_value(stone, Stone::White, "White");
		    }
		    
		    // TODO: Setup captured stones
		    ui.separator();
		}
	    }

	    // Widgets common to game and analysis mode
	    if let Some(w) = self.get_workspace() {
		let game_mode = w.game_mode;
		if let WorkspaceMode::Game | WorkspaceMode::Analysis = w.mode {
		    // TODO: Pass and resign
		    		    
		    if let Some(model) = self.get_model_mut() {
			if ui.button("Undo").clicked() {
			    match game_mode {
				GameMode::HumanVsHuman => {
				    if !model.undo() { println!("Cannot undo! No history."); }
				},
				GameMode::HumanVsComputer(Turn::Black) => {
				    if !model.undo() { println!("Cannot undo! No history."); }
				    if !model.undo() { println!("Cannot undo! No history."); }
				}
				GameMode::HumanVsComputer(Turn::White) => {
				    if model.get_move_count() >= 2 {
					model.undo();
					model.undo();
				    } else {
					println!("Cannot undo! No history before computer's start.");
				    }
				},
			    }
			}
		    }
		    ui.separator();
		}
	    }

	    // Widgets specific to analysis mode
	    if let Some(w) = self.get_workspace() {
		if let WorkspaceMode::Analysis = w.mode {
		    if let Some(w) = self.get_workspace_mut() {
			if ui.button("Calculate score").clicked() {
			    let (ts_black, ts_white) = w.model.calculate_territory_score();
			    let (as_black, as_white) = w.model.calculate_area_score();
			    w.black_territory_score = ts_black;
			    w.white_territory_score = ts_white;
			    w.black_area_score = as_black;
			    w.white_area_score = as_white;
			}
		    }
		    
		    if let Some(w) = self.get_workspace() {
			ui.label(format!("Black territory score: {}", w.black_territory_score));
			ui.label(format!("White territory score: {}", w.white_territory_score));
			ui.label(format!("Black area score: {}", w.black_area_score));
			ui.label(format!("White area score: {}", w.white_area_score));
		    }
		    ui.separator();
		}
	    }
	});
    }

    fn draw_workspace_central_panel(&mut self, ctx: &egui::Context) {
	egui::CentralPanel::default().show(ctx, |ui| {
	    let painter = ui.painter();
	    let rect = ui.max_rect();
	    
	    //
	    // DRAW BOARD
	    //

	    if let Some(model) = self.get_model() {
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
		let square_b = square_a.shrink(margin); // the square that contains smaller squares whose centers are intersections
		let board_origin = square_b.left_top(); // top left of the square of size cell_size whose center is the intersection

		// Draw a Go board inside square B
		let top = square_b.top() + cell_size / 2.0;
		let bottom = square_b.bottom() - cell_size / 2.0;
		let left = square_b.left() + cell_size / 2.0;
		let right = square_b.right() - cell_size / 2.0;
		for i in 0..board_size {
		    // Horizontal lines
		    let y = top + i as f32 * cell_size;
		    painter.line_segment([
			egui::Pos2::new(left, y),
			egui::Pos2::new(right, y),
		    ], egui::Stroke::new(1.0, egui::Color32::BLACK));
		    
		    // Vertical lines
		    let x = left + i as f32 * cell_size;
		    painter.line_segment([
			egui::Pos2::new(x, top),
			egui::Pos2::new(x, bottom),
		    ], egui::Stroke::new(1.0, egui::Color32::BLACK));
		}

		// Draw stones
		let board = model.get_board();
		let stone_radius_ratio = 0.45; // 0.5 makes the stones touch
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
			    let x = square_b.left() + cell_size / 2.0 + col_idx as f32 * cell_size;
			    let y = square_b.top() + cell_size / 2.0 + row_idx as f32 * cell_size;

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
		
		//
		// GET MOUSE INPUT
		//
		
		// Handle clicks
		if ui.rect_contains_pointer(rect) {
		    if ui.input(|i| i.pointer.primary_clicked()) {
			if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
			    match board_coordinates(pos, board_origin, cell_size) {
				Ok((x, y)) => {
				    self.handle_left_click_board(x, y);
				},
				Err(s) => {
				    println!("{s}");
				}
			    }
			}
		    }
		    if ui.input(|i| i.pointer.secondary_clicked()) {
			if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
			    match board_coordinates(pos, board_origin, cell_size) {
				Ok((x, y)) => {
				    self.handle_right_click_board(x, y);
				},
				Err(s) => {
				    println!("{s}");
				}
			    }
			}
		    }
		}
	    }
	});
    }

    fn handle_left_click_board(&mut self, x: usize, y: usize) {
	if let Some(w) = self.get_workspace() {
	    let game_mode = w.game_mode;
	    match w.mode {
		WorkspaceMode::Setup => {
		    let stone = w.stone;
		    if let Some(model) = self.get_model_mut() {
			let r = model.setup_add_stone(x, y, stone);
			if let Err(s) = r {
			    println!("Model setup_add_stone unsuccessful! {s}");
			}
		    }
		},
		WorkspaceMode::Game => {
		    if let Some(model) = self.get_model_mut() {
			let r = model.make_move(x, y); // human move
			match r {
			    Ok(()) => {
				if let GameMode::HumanVsComputer(turn) = game_mode {
				    let model_turn = model.get_turn();
				    if turn != model_turn {
					let r = model.make_move_computer(); // computer move
					if let Err(s) = r {
					    println!("Model make_move_computer unsuccessful! {s}");
					}
				    } else {
					println!("Game mode is HumanVsComputer({turn:?}). It's computer's turn but model returned {model_turn:?}. Computer will not make any move.");
				    }
				}
			    },
			    Err(s) => {
				println!("Model make_move unsuccessful! {s}");
			    }
			}
		    }
		},
		WorkspaceMode::Analysis => (), // TODO: mouse clicks during analysis
	    }
	}
    }

    fn handle_right_click_board(&mut self, x: usize, y: usize) {
	if let Some(w) = self.get_workspace() {
	    match w.mode {
		WorkspaceMode::Setup => {
		    if let Some(model) = self.get_model_mut() {
			let r = model.setup_remove_stone(x, y);
			if let Err(s) = r {
			    println!("Model setup_remove_stone unsuccessful! {s}");
			}
		    }
		},
		WorkspaceMode::Game => (), // do nothing
		WorkspaceMode::Analysis => (), // TODO: right click in analysis mode
	    }
	}
    }

    fn draw_create_workspace_central_panel(&mut self, ctx: &egui::Context) {
	// TODO: Don't provide some analysis features and human-vs-computer features if engine is not setup.
	egui::CentralPanel::default().show(ctx, |ui| {
	    ui.add(egui::Slider::new(&mut self.new_workspace_setup.board_size, 2..=25).text("Board size"));

	    let game_mode = &mut self.new_workspace_setup.game_mode;
	    ui.label("Game mode:");
	    ui.radio_value(game_mode, GameMode::HumanVsHuman, "Human vs. human");
	    ui.radio_value(game_mode, GameMode::HumanVsComputer(Turn::Black), "Human (black) vs. computer (white)");
	    ui.radio_value(game_mode, GameMode::HumanVsComputer(Turn::White), "Human (white) vs. computer (black)");

	    ui.horizontal(|ui| {
		if ui.button("Cancel").clicked() {
		    self.mode = ViewMode::Workspace;
		}
		if ui.button("Create workspace").clicked() {
		    self.new_workspace();
		    self.mode = ViewMode::Workspace;
		}
	    });
	});
    }

    fn draw_install_engine_central_panel(&mut self, ctx: &egui::Context) {
	egui::CentralPanel::default().show(ctx, |ui| {
	    // TODO: Custom engine setup via command, save the command to config file

	    let mutex = &self.katago_installer_status;

	    // Status information
	    let mut installation_status_str = String::from("KataGo installation status: ");
	    let mut tuning_status_str = String::from("KataGo tuning status: ");
	    if let Ok(status) = mutex.try_lock() {
		match status.is_installed {
		    Some(true) => { installation_status_str.push_str("installed"); },
		    Some(false) => { installation_status_str.push_str("not installed"); },
		    None => { installation_status_str.push_str("not known"); },
		}
		match status.is_tuned {
		    Some(true) => { tuning_status_str.push_str("tuned"); },
		    Some(false) => { tuning_status_str.push_str("not tuned"); },
		    None => { tuning_status_str.push_str("not known"); },
		}
	    } else {
		installation_status_str.push_str("not known");
		tuning_status_str.push_str("not known");
	    }
	    ui.label(installation_status_str);
	    ui.label(tuning_status_str);

	    // Buttons
	    ui.horizontal(|ui| {
		if let Ok(_status) = mutex.try_lock() {
		    // Install button
		    if ui.button("(Re)Install").clicked() {
			self.do_katago_installer_op(|installer, status| {
			    match installer.install() {
				Ok(()) => {
				    status.is_installed = Some(true);
				},
				Err(s) => {
				    status.is_installed = Some(false);
				    println!("KataGo installation unsuccessful! {s}");
				    return;
				}
			    }
			    match installer.test() {
				Ok(()) => {
				    status.is_operational = Some(true);
				},
				Err(s) => {
				    status.is_operational = Some(false);
				    println!("KataGo testing unsuccessful! {s}");
				    return;
				}
			    }			
			});
		    }
		    
		    // Tune button
		    if ui.button("(Re)Tune").clicked() {
			self.do_katago_installer_op(|installer, status| {
			    match installer.tune() {
				Ok(()) => {
				    status.is_tuned = Some(true);
				},
				Err(s) => {
				    status.is_tuned = Some(false);
				    println!("KataGo tuning unsuccessful! {s}");
				    return;
				}
			    }
			});
		    }
		} else {
		    // Deactivated buttons and spinner.
		    if ui.add_enabled(false, egui::Button::new("Re(Install)")).clicked() {}
		    if ui.add_enabled(false, egui::Button::new("Re(Tune)")).clicked() {}
		    ui.add(egui::Spinner::new());
		}
	    });
	});
    }

    fn do_katago_installer_op<F>(&self, func: F)
    where F: FnOnce(&KataGoInstaller, &mut KataGoInstallerStatus) + std::marker::Send + 'static {
	// Move a copy of mutex and installer to the new
	// thread. Main thread dropps the mutex as soon as the new
	// thread is created. The new thread locks the mutex.
	let installer = self.katago_installer.clone();
	let mutex = self.katago_installer_status.clone();
	let _handler = thread::spawn(move || {
	    let mut status = mutex.lock().unwrap();
	    func(&installer, &mut status);
	});
    }
}


fn board_coordinates(pos: egui::Pos2, origin: egui::Pos2, cell_size: f32) -> Result<(usize, usize), String> {
    // Origin in the top left corner of the square of size cell_size
    // whose center is the corner intersection.
    let pos = (pos - origin) / cell_size;
    let x = pos.x.floor() as i32;
    let y = pos.y.floor() as i32;
    if x >= 0 && y >= 0 {
	Ok((x as usize, y as usize))
    } else {
	Err(format!("Board coordinates ({x}, {y}) are calculated from egui position are negative!"))
    }
}
