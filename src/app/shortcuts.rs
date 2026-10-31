//! The keyboard shortcut sheet: `?` in either view, or the ⋯ menus.

use eframe::egui::{self, Key, Modifiers, RichText};

use super::NlpApp;
use super::widgets::{self, theme};

/// `?` (Shift+/ on most layouts) opens or closes the sheet.
pub fn toggle_pressed(ctx: &egui::Context) -> bool {
    ctx.input(|i| {
        !i.modifiers.command
            && (i.key_pressed(Key::Questionmark)
                || (i.modifiers.shift && i.key_pressed(Key::Slash)))
    })
}

impl NlpApp {
    pub fn shortcuts_ui(&mut self, ctx: &egui::Context) {
        if !ctx.egui_wants_keyboard_input() && toggle_pressed(ctx) {
            self.shortcuts_open = !self.shortcuts_open;
        }
        if !self.shortcuts_open {
            return;
        }
        let k = |m: Modifiers, key: Key| widgets::shortcut(ctx, m, key);
        let cmd = Modifiers::COMMAND;
        let shift = Modifiers::SHIFT;
        let none = Modifiers::NONE;
        let groups: [(&str, Vec<(String, &str)>); 4] = [
            (
                "Roll",
                vec![
                    ("← → ↑ ↓".into(), "Move the selection (⇧ extends it)"),
                    (k(cmd, Key::A), "Select all"),
                    (k(none, Key::Escape), "Clear the selection"),
                    (k(none, Key::Enter), "Open in the editor"),
                    (k(none, Key::Space), "Quick look"),
                    ("P X U".into(), "Pick / reject / unflag"),
                    ("N".into(), "Hold to see the negatives"),
                    (k(cmd, Key::E), "Export"),
                    (
                        format!("{} {}", k(cmd, Key::OpenBracket), k(cmd, Key::CloseBracket)),
                        "Rotate left / right",
                    ),
                ],
            ),
            (
                "Editor",
                vec![
                    ("← →".into(), "Previous / next frame"),
                    (
                        format!("{} {}", k(none, Key::Space), k(none, Key::Escape)),
                        "Back to the roll",
                    ),
                    (k(none, Key::B), "Before / after"),
                    (k(shift, Key::B), "Split view"),
                    (k(none, Key::J), "Show clipping"),
                    (k(none, Key::W), "Pick a neutral grey"),
                    (k(none, Key::C), "Crop & straighten"),
                    (k(none, Key::H), "Flip horizontally"),
                    (k(none, Key::F), "Show / hide the filmstrip"),
                    ("P X U".into(), "Pick / reject / unflag"),
                    ("+ −".into(), "Zoom in / out"),
                ],
            ),
            (
                "Lab keys (number row or numpad)",
                vec![
                    ("7 8 9".into(), "Add cyan, magenta, yellow"),
                    ("1 2 3".into(), "Remove cyan, magenta, yellow"),
                    ("4".into(), "Print darker (density +)"),
                    ("6".into(), "Print lighter (density −)"),
                    ("0".into(), "Reset the correction"),
                ],
            ),
            (
                "Everywhere",
                vec![
                    (k(cmd, Key::Z), "Undo"),
                    (k(cmd | shift, Key::Z), "Redo"),
                    (k(cmd, Key::C), "Copy settings"),
                    (k(cmd, Key::V), "Paste settings"),
                    (k(cmd | shift, Key::V), "Paste to every frame (editor)"),
                    ("?".into(), "This sheet"),
                ],
            ),
        ];
        let frame = egui::Frame::new()
            .fill(theme::PANEL)
            .stroke(egui::Stroke::new(1.0, theme::LINE))
            .corner_radius(12.0)
            .inner_margin(egui::Margin::same(22))
            .shadow(ctx.global_style().visuals.window_shadow);
        let r = egui::Modal::new(egui::Id::new("shortcuts_modal"))
            .backdrop_color(egui::Color32::from_black_alpha(150))
            .frame(frame)
            .show(ctx, |ui| {
                ui.set_width(720.0);
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("Keyboard shortcuts")
                            .font(theme::medium(16.0))
                            .color(theme::TEXT),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if widgets::icon_button(
                            ui,
                            widgets::Icon::Close,
                            "Close (Esc)",
                            false,
                            true,
                        )
                        .clicked()
                        {
                            self.shortcuts_open = false;
                        }
                    });
                });
                ui.add_space(12.0);
                let col_w = (ui.available_width() - 24.0) / 2.0;
                ui.horizontal_top(|ui| {
                    ui.spacing_mut().item_spacing.x = 24.0;
                    for pair in groups.chunks(2) {
                        ui.vertical(|ui| {
                            ui.set_width(col_w);
                            for (title, rows) in pair {
                                group(ui, title, rows);
                                ui.add_space(14.0);
                            }
                        });
                    }
                });
            });
        let esc = ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape));
        if esc || r.should_close() {
            self.shortcuts_open = false;
        }
    }
}

fn group(ui: &mut egui::Ui, title: &str, rows: &[(String, &str)]) {
    ui.label(
        RichText::new(title)
            .font(theme::medium(12.5))
            .color(theme::MUTED),
    );
    ui.add_space(4.0);
    egui::Grid::new(("shortcuts", title))
        .num_columns(2)
        .spacing(egui::vec2(14.0, 5.0))
        .min_col_width(96.0)
        .show(ui, |ui| {
            for (keys, what) in rows {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 3.0;
                    for cap in keys.split(' ') {
                        key_cap(ui, cap);
                    }
                });
                ui.label(RichText::new(*what).color(theme::TEXT));
                ui.end_row();
            }
        });
}

/// One key drawn as a small keycap.
fn key_cap(ui: &mut egui::Ui, text: &str) {
    let g = ui.painter().layout_no_wrap(
        text.to_string(),
        egui::FontId::proportional(11.5),
        theme::TEXT,
    );
    let size = egui::vec2((g.size().x + 10.0).max(20.0), 20.0);
    let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
    let p = ui.painter();
    p.rect(
        rect,
        4.0,
        theme::SURFACE,
        egui::Stroke::new(1.0, theme::TRACK),
        egui::StrokeKind::Inside,
    );
    p.galley(rect.center() - g.size() / 2.0, g, theme::TEXT);
}
