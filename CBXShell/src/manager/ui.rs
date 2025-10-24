///! Modern egui-based UI for CBXManager
///!
///! Compact, professional interface with proper alignment and spacing

use super::{registry_ops, state::AppState, utils};
use eframe::egui;

pub struct CBXManagerApp {
    state: AppState,
    needs_restart_prompt: bool,
}

impl Default for CBXManagerApp {
    fn default() -> Self {
        // Load current state from registry
        let state = registry_ops::read_app_state().unwrap_or_default();

        Self {
            state,
            needs_restart_prompt: false,
        }
    }
}

impl CBXManagerApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self::default()
    }

    fn apply_settings(&mut self) {
        if !self.state.is_valid() {
            return;
        }

        if let Err(e) = registry_ops::write_app_state(&self.state) {
            eprintln!("Failed to save settings: {}", e);
        } else {
            self.needs_restart_prompt = true;
        }
    }

    fn register_dll(&mut self) {
        match registry_ops::register_dll() {
            Ok(_) => {
                self.state = registry_ops::read_app_state().unwrap_or_default();
            }
            Err(e) => {
                eprintln!("Failed to register DLL: {}", e);
            }
        }
    }

    fn unregister_dll(&mut self) {
        match registry_ops::unregister_dll() {
            Ok(_) => {
                self.state = registry_ops::read_app_state().unwrap_or_default();
                self.needs_restart_prompt = true;
            }
            Err(e) => {
                eprintln!("Failed to unregister DLL: {}", e);
            }
        }
    }
}

impl eframe::App for CBXManagerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Top menu bar
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("Tools", |ui| {
                    if ui.button("Register DLL").clicked() {
                        self.register_dll();
                        ui.close_menu();
                    }
                    if ui.button("Unregister DLL").clicked() {
                        self.unregister_dll();
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("About").clicked() {
                        ui.close_menu();
                    }
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // Compact top padding
            ui.add_space(8.0);

            // Status - compact
            ui.horizontal(|ui| {
                let (icon, color) = if self.state.dll_registered {
                    ("✓", egui::Color32::from_rgb(0, 160, 0))
                } else {
                    ("⚠", egui::Color32::from_rgb(200, 150, 0))
                };
                ui.colored_label(color, icon);
                ui.label(if self.state.dll_registered {
                    "DLL Registered"
                } else {
                    "DLL Not Registered"
                });
            });

            ui.add_space(8.0);

            // Fixed width for both group boxes
            let group_width = 320.0;

            // File types group - left aligned, fixed width
            ui.horizontal(|ui| {
                egui::Frame::group(ui.style())
                    .inner_margin(egui::Margin::same(8.0))
                    .show(ui, |ui| {
                        ui.set_width(group_width);
                        ui.vertical(|ui| {
                    ui.label(egui::RichText::new("File types").strong());
                    ui.add_space(4.0);

                    // CBZ + ZIP (tight)
                    ui.checkbox(
                        self.state.get_extension_mut(".cbz").map(|e| &mut e.thumbnail_enabled).unwrap(),
                        "CBZ Image Archives",
                    );
                    ui.checkbox(
                        self.state.get_extension_mut(".zip").map(|e| &mut e.thumbnail_enabled).unwrap(),
                        "ZIP Archives",
                    );

                    ui.add_space(6.0);

                    // CBR + RAR (tight)
                    ui.checkbox(
                        self.state.get_extension_mut(".cbr").map(|e| &mut e.thumbnail_enabled).unwrap(),
                        "CBR Image Archives",
                    );
                    ui.checkbox(
                        self.state.get_extension_mut(".rar").map(|e| &mut e.thumbnail_enabled).unwrap(),
                        "RAR Archives",
                    );

                    ui.add_space(6.0);

                    // CB7 + 7Z (tight)
                    ui.checkbox(
                        self.state.get_extension_mut(".cb7").map(|e| &mut e.thumbnail_enabled).unwrap(),
                        "CB7 Image Archives",
                    );
                    ui.checkbox(
                        self.state.get_extension_mut(".7z").map(|e| &mut e.thumbnail_enabled).unwrap(),
                        "7Z Archives",
                    );
                        });
                    });
            });

            ui.add_space(8.0);

            // Advanced group - left aligned, fixed width
            ui.horizontal(|ui| {
                egui::Frame::group(ui.style())
                    .inner_margin(egui::Margin::same(8.0))
                    .show(ui, |ui| {
                        ui.set_width(group_width);
                        ui.vertical(|ui| {
                    ui.label(egui::RichText::new("Advanced").strong());
                    ui.add_space(4.0);

                    ui.checkbox(&mut self.state.sort_enabled, "Sort images alphabetically");
                    ui.add_space(2.0);
                    ui.label(
                        egui::RichText::new("Uncheck to sort images by archive order.\nRequired to display custom thumbnail.")
                            .small()
                            .color(egui::Color32::GRAY),
                    );
                        });
                    });
            });

            ui.add_space(12.0);

            // Buttons - right aligned
            ui.horizontal(|ui| {
                let button_width = 80.0;
                ui.add_space(ui.available_width() - (button_width * 3.0) - 16.0); // 3 buttons + spacing

                if ui.add_sized([button_width, 24.0], egui::Button::new("OK")).clicked() {
                    self.apply_settings();
                    if self.needs_restart_prompt && utils::prompt_restart_explorer() {
                        let _ = utils::restart_explorer();
                    }
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }

                if ui.add_sized([button_width, 24.0], egui::Button::new("Cancel")).clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }

                if ui.add_sized([button_width, 24.0], egui::Button::new("Apply")).clicked() {
                    self.apply_settings();
                }
            });
        });
    }
}
