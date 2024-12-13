use crate::view::*;
use crate::model::{Model, Point};
use eframe::egui;



pub struct EguiView {
    model: Model
}


impl View for EguiView {
    fn make(model: Model) -> Self {
	Self {
	    model
	}
    }

    fn run(&mut self) {
	self.init_model();
	self.run_egui();
    }
}


impl EguiView {
    fn init_model(&mut self) {
	let res = self.model.restart();
	if let Err(s) = res {
	    println!("Error at model restart: {s}");
	}
    }

    fn run_egui(&self) {
	let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
            ..Default::default()
	};

	// TODO: Implement the UI.
	let mut name = "Arthur".to_owned();
	let mut age = 42;

	eframe::run_simple_native("Go", options, move |ctx, _frame| {
            egui::CentralPanel::default().show(ctx, |ui| {
		ui.heading("My egui Application");
		ui.horizontal(|ui| {
                    let name_label = ui.label("Your name: ");
                    ui.text_edit_singleline(&mut name)
			.labelled_by(name_label.id);
		});
		ui.add(egui::Slider::new(&mut age, 0..=120).text("age"));
		if ui.button("Increment").clicked() {
                    age += 1;
		}
		ui.label(format!("Hello '{name}', age {age}"));
            });
	}).unwrap();
    }
}
