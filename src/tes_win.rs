use eframe::*;
use egui::Modifiers;
use std::{path::PathBuf, io::{BufReader, Read, BufWriter, Write}};
use rfd::FileDialog;
use std::fs;
use filetime::FileTime;
use std::process::*;

use rv_sim::{asm_f, parts::*};

pub fn fun() {
	let mut native_option = NativeOptions::default();
	native_option.default_theme = eframe::Theme::Light;
	native_option.initial_window_size = Some(egui::Vec2 {x: 600.0, y: 450.0});
	native_option.min_window_size = Some(egui::Vec2 {x: 350.0, y: 450.0});
	let _ = run_native("Simple RISC-V Developer", native_option, Box::new(|cc| Box::new(MyEguiApp::new(cc))));
}

impl Default for MyEguiApp {
	fn default() -> MyEguiApp {
		MyEguiApp { 
			source_path: Option::None,
			source: String::new(), 
			console: String::new(), 
			iconsole: String::new()
		}
	}
}

struct MyEguiApp {
	source_path: Option<PathBuf>,
	pub source: String,
	pub console: String,
	pub iconsole: String,
}

impl MyEguiApp {
	fn new(_cc: &CreationContext<'_>) -> Self {
		Self::default()
	}
}

impl App for MyEguiApp {
	fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {

		let save = egui::KeyboardShortcut::new(Modifiers::CTRL, egui::Key::S);

		egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
			// The top panel is often a good place for a menu bar:

			egui::menu::bar(ui, |ui| {
				#[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
				{
					ui.menu_button("File", |ui| {
						if ui.button("Quit").clicked() {
							frame.close();
						}
						if ui.button("Open")
						.clicked() {
							if let Some(path) = FileDialog::new()
							.add_filter("Assembly file", &["asm"])
							.set_directory("../")
							.pick_file() {
								self.source_path = Some(path.clone());

								let file = fs::File::open(path).unwrap();
								let mut reader = BufReader::new(file);
								let _ = reader.read_to_string(&mut self.source);
							}
							ui.close_menu();
						}
						if ui.button("Save").clicked() {
							if let Some(p) = self.source_path.clone() {
								if !p.is_file() {
									if let Some(path) = FileDialog::new()
									.add_filter("Assembly file", &["asm"])
									.set_directory("../")
									.save_file() {
										self.source_path = Some(path.clone());
									}
								}
								if let Some(path) = self.source_path.clone() {
									let file = fs::File::create(path).unwrap();
									let mut writer = BufWriter::new(file);
									writer.write_all(self.source.as_bytes()).unwrap();
								}
							} else {
								if let Some(path) = FileDialog::new()
								.add_filter("Assembly file", &["asm"])
								.set_directory("../")
								.save_file() {
									self.source_path = Some(path.clone());
								}
								if let Some(path) = self.source_path.clone() {
									let file = fs::File::create(path).unwrap();
									let mut writer = BufWriter::new(file);
									writer.write_all(self.source.as_bytes()).unwrap();
								}
							}
							ui.close_menu();
						}
						if ui.button("Save As").clicked() {
							if let Some(path) = FileDialog::new()
							.add_filter("Assembly file", &["asm"])
							.set_directory("../")
							.save_file() {
								self.source_path = Some(path.clone());
								if let Some(path) = self.source_path.clone() {
									let file = fs::File::create(path).unwrap();
									let mut writer = BufWriter::new(file);
									writer.write_all(self.source.as_bytes()).unwrap();
								}
							}
							ui.close_menu();
						}
					});
					ui.add_space(16.0);
				}

				egui::widgets::global_dark_light_mode_buttons(ui);
			});
		});
		egui::SidePanel::left("left_panel").show(ctx, |ui| {
			ui.heading(egui::RichText::new("Action").font(egui::FontId::proportional(8.0)));

			let btn_txt = egui::RichText::new("ASM").font(egui::FontId::proportional(15.0));
			let btn = egui::Button::new(btn_txt);
			let resp = ui.add_sized(egui::Vec2{x: 100.0, y: 20.0}, btn);
			if resp.clicked() {
				if let Some(path) = self.source_path.clone() {
					let str = path.to_str().unwrap();
					asm_f::asm(str, "tmp/data.dat");
					let s = format!("Success Assemble.\nFrom asm file({}) to bin file(tmp/data.dat)\n", str);
					self.console.push_str(&s);
				} else {
					self.console.push_str("Failed Assemble.\n");
				}
			}

			let btn_txt = egui::RichText::new("Sim").font(egui::FontId::proportional(15.0));
			let btn = egui::Button::new(btn_txt);
			let resp = ui.add_sized(egui::Vec2{x: 100.0, y: 20.0}, btn);
			if resp.clicked() {
				if let Ok(m) = fs::metadata("tmp/data.dat") {
					if let Some(path) = self.source_path.clone() {
						let sm = fs::metadata(path.clone()).unwrap();

						// source file time > bin file time
						if FileTime::from_last_modification_time(&sm) > FileTime::from_last_modification_time(&m) {
							let str = path.to_str().unwrap();
							asm_f::asm(str, "tmp/data.dat");
							let s = format!("Success Assemble.\nFrom asm file({}) to bin file(tmp/data.dat)\n\n", str);
							self.console.push_str(&s);
						}

						let mut vm = rv_sim::parts::VMachine::new(1024);
						vm.cpu.binread("tmp/data.dat");
						vm.start_dbg(256);

						
						let d: u32 = vm.cpu.memory.read(0);
						self.console.push_str("[Memory dump mode]\n");
						self.console.push_str(&format!("Address[{:08x}]: {:08x} \n",0, d));
						//println!("[Memory dump mode]");
						//println!("Address[{:08x}]: {:08x} ",0, d);

						for i in 1..(100 / 4 + 1) {
							let d: u32 = vm.cpu.memory.read(i*4);
							self.console.push_str(&format!("Address[{:08x}]: {:08x} \n",i * 4, d));
							//println!("Address[{:08x}]: {:08x} ",0, d);
						}
					} else {
						self.console.push_str("Failed Simulated RISC-V Sequence.\n");
					}
				} else {
					let str = self.source_path.clone().unwrap();
					let str = str.to_str().unwrap();
					asm_f::asm(str, "tmp/data.dat");
					let s = format!("Success Assemble.\nFrom asm file({}) to bin file(tmp/data.dat)\n\n", str);
					self.console.push_str(&s);

					let mut vm = rv_sim::parts::VMachine::new(1024);
					vm.cpu.binread("tmp/data.dat");
					vm.start_dbg(256);

					let d: u32 = vm.cpu.memory.read(0);
					self.console.push_str("[Memory dump mode]\n");
					self.console.push_str(&format!("Address[{:08x}]: {:08x} \n",0, d));
					//println!("[Memory dump mode]");
					//println!("Address[{:08x}]: {:08x} ",0, d);

					for i in 1..(100 / 4 + 1) {
						let d: u32 = vm.cpu.memory.read(i*4);
						self.console.push_str(&format!("Address[{:08x}]: {:08x} \n",i * 4, d));
						//println!("Address[{:08x}]: {:08x} ",0, d);
					}
				}
			}

			let btn_txt = egui::RichText::new("V-Sim").font(egui::FontId::proportional(15.0));
			let btn = egui::Button::new(btn_txt);
			let resp = ui.add_sized(egui::Vec2{x: 100.0, y: 20.0}, btn);
			if resp.clicked() {
				let conv_arg : [&str; 5]= ["-o", "tmp/a.out", "utils/tb_core.v", "utils/imem.v", "core/*"];

				let out = Command::new("iverilog")
					.args(conv_arg)
					.stdout(Stdio::piped())
					.stderr(Stdio::piped())
					.output()
					.unwrap();

				let sout = String::from_utf8(out.stdout).unwrap();
				self.console.push_str(&sout);

				if !out.status.success() {
					self.console.push_str("iverilog could't finish correctly.\n");
				} else {
					self.console.push_str("iverilog sequence successfly.\n\n");

					let out = Command::new("vvp")
						.args(["tmp/a.out"])
						.stdout(Stdio::piped())
						.stderr(Stdio::piped())
						.output()
						.unwrap();

					if !out.status.success() {
						self.console.push_str("vvp could't finish correctly.\n");
					} else {
						let sout = String::from_utf8(out.stdout).unwrap();
						self.console.push_str(&sout);
					}
				}
			}

			let btn_txt = egui::RichText::new("Show GTK-Wave").font(egui::FontId::proportional(15.0));
			let btn = egui::Button::new(btn_txt);
			let resp = ui.add_sized(egui::Vec2{x: 100.0, y: 20.0}, btn);
			if resp.clicked() {
				let mut child = Command::new("gtkwave")
				.args(["tmp/dump.vcd"])
				.stdout(Stdio::null())
				.spawn()
				.expect("faled to execute process.");
				let status = child.wait().unwrap();
				//let sout = String::from_utf8().unwrap();
				//self.console.push_str(&sout);
				self.console.push_str("\nshutdown gtkwave.\n");
			}

			let btn_txt = egui::RichText::new("Generate").font(egui::FontId::proportional(15.0));
			let btn = egui::Button::new(btn_txt);
			let resp = ui.add_sized(egui::Vec2{x: 100.0, y: 20.0}, btn);
		});

		egui::CentralPanel::default().show(ctx, |ui| {
			
			ui.set_max_height(frame.info().window_info.size.y - 180.0);
			ui.set_min_height(frame.info().window_info.size.y - 180.0);

			ui.heading("Risc-V Simple Developer");
			
			ui.spacing();

			let code_txt = egui::TextEdit::multiline(&mut self.source)
			.font(egui::FontId::proportional(10.0))
			.code_editor();
			egui::ScrollArea::vertical().max_height(frame.info().window_info.size.y - 200.0).show(ui, |ui| {
				ui.add_sized(egui::Vec2{x: frame.info().window_info.size.x - 150.0, y: frame.info().window_info.size.y - 200.0}, code_txt);
			});

			//ui.separator();
			ui.end_row();
			
		});

		egui::TopBottomPanel::bottom("bottom_panel").default_height(110.0).show(ctx, |ui| {
			ui.set_max_height(120.0);
			ui.set_min_height(120.0);

			ui.heading(egui::RichText::new("Console Panel").font(egui::FontId::proportional(10.0)));
			let console_txt = egui::TextEdit::multiline(&mut self.console)
			.font(egui::FontId::proportional(10.0));

			egui::ScrollArea::vertical().max_height(90.0).show(ui, |ui| {
				ui.add_sized(egui::Vec2{x: frame.info().window_info.size.x - 150.0, y: 90.0}, console_txt);
			});

			let input_console_txt = egui::TextEdit::singleline(&mut self.iconsole)
			.font(egui::FontId::proportional(10.0))
			.hint_text("console");

			let btn_txt = egui::RichText::new("run").font(egui::FontId::proportional(10.0));
			let console_run = egui::Button::new(btn_txt);

			ui.horizontal(|ui| {
				ui.add_sized(egui::Vec2{x: frame.info().window_info.size.x - 200.0, y: 20.0}, input_console_txt);
				let resp = ui.add_sized(egui::Vec2{x: 40.0, y: 20.0}, console_run);
			});
		});
	}
}