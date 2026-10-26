//! Shared controls mirroring the plugin's UI conventions:
//! click a label to reset, ⟳ to cycle a dropdown, typed values may exceed
//! the slider range.

use eframe::egui::{self, Color32, Rect, Sense, Ui};

pub const LABEL_W: f32 = 92.0;

pub fn gradient_rect(ui: &Ui, rect: Rect, a: Color32, b: Color32) {
    let mut mesh = egui::Mesh::default();
    mesh.colored_vertex(rect.left_top(), a);
    mesh.colored_vertex(rect.right_top(), b);
    mesh.colored_vertex(rect.right_bottom(), b);
    mesh.colored_vertex(rect.left_bottom(), a);
    mesh.add_triangle(0, 1, 2);
    mesh.add_triangle(0, 2, 3);
    ui.painter().add(egui::Shape::mesh(mesh));
}

/// Integer slider row (e.g. ranges 1..10).
pub fn slider_u8(
    ui: &mut Ui,
    label: &str,
    v: &mut u8,
    range: std::ops::RangeInclusive<u8>,
    default: u8,
    tip: &str,
) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        let lbl = ui.add_sized(
            [LABEL_W, 18.0],
            egui::Label::new(label).sense(Sense::click()),
        );
        let lbl = if tip.is_empty() {
            lbl
        } else {
            lbl.on_hover_text(tip)
        };
        if lbl.clicked() && *v != default {
            *v = default;
            changed = true;
        }
        ui.spacing_mut().slider_width = (ui.available_width() - 76.0).max(40.0);
        changed |= ui.add(egui::Slider::new(v, range)).changed();
    });
    changed
}

/// A dropdown with a ⟳ button that cycles to the next option.
pub fn combo<T: Copy + PartialEq>(
    ui: &mut Ui,
    label: &str,
    v: &mut T,
    all: &[T],
    name: impl Fn(T) -> &'static str,
    tip: &str,
) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        let lbl = ui.add_sized([LABEL_W, 18.0], egui::Label::new(label));
        if !tip.is_empty() {
            lbl.on_hover_text(tip);
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui
                .small_button("⟳")
                .on_hover_text("Cycle to the next option")
                .clicked()
            {
                let i = all.iter().position(|o| *o == *v).unwrap_or(0);
                *v = all[(i + 1) % all.len()];
                changed = true;
            }
            let w = (ui.available_width() - 4.0).max(60.0);
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                egui::ComboBox::from_id_salt(("combo", label, std::any::type_name::<T>(), ui.id()))
                    .width(w)
                    .selected_text(name(*v))
                    .show_ui(ui, |ui| {
                        for &o in all {
                            if ui.selectable_label(*v == o, name(o)).clicked() && *v != o {
                                *v = o;
                                changed = true;
                            }
                        }
                    });
            });
        });
    });
    changed
}

pub fn section<R>(ui: &mut Ui, title: &str, add: impl FnOnce(&mut Ui) -> R) -> R {
    egui::Frame::group(ui.style())
        .inner_margin(egui::Margin::same(8))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(
                egui::RichText::new(title)
                    .small()
                    .strong()
                    .color(ui.visuals().weak_text_color()),
            );
            ui.add_space(2.0);
            add(ui)
        })
        .inner
}

/// Paint an RGB histogram (counts per channel, 256 bins) as soft filled
/// channel areas over a neutral card.
pub fn histogram(ui: &mut Ui, h: &[[u32; 256]; 3], size: egui::Vec2) {
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    let p = ui.painter_at(rect);
    p.rect_filled(rect, 8.0, theme::card());
    let inner = rect.shrink2(egui::vec2(6.0, 6.0));
    let max = h.iter().flat_map(|c| c[2..254].iter()).copied().max().unwrap_or(1).max(1) as f32;
    let a = if theme::is_dark() { 70 } else { 55 };
    let cols = [
        (Color32::from_rgba_unmultiplied(235, 72, 72, a), Color32::from_rgb(225, 80, 80)),
        (Color32::from_rgba_unmultiplied(64, 190, 96, a), Color32::from_rgb(60, 170, 90)),
        (Color32::from_rgba_unmultiplied(70, 120, 245, a), Color32::from_rgb(70, 120, 235)),
    ];
    for (c, (fill, stroke)) in cols.iter().enumerate() {
        // Smooth a little so the outline reads as a curve, not a comb.
        let v: Vec<f32> = (0..256)
            .map(|i| {
                let lo = i.saturating_sub(2);
                let hi = (i + 2).min(255);
                let s: u32 = h[c][lo..=hi].iter().sum();
                (s as f32 / (hi - lo + 1) as f32 / max).sqrt().min(1.0)
            })
            .collect();
        let pt = |i: usize| egui::pos2(inner.left() + inner.width() * i as f32 / 255.0, inner.bottom() - inner.height() * v[i]);
        let mut mesh = egui::Mesh::default();
        for i in 0..256 {
            mesh.colored_vertex(egui::pos2(pt(i).x, inner.bottom()), *fill);
            mesh.colored_vertex(pt(i), *fill);
            if i > 0 {
                let k = (i * 2) as u32;
                mesh.add_triangle(k - 2, k - 1, k);
                mesh.add_triangle(k - 1, k, k + 1);
            }
        }
        p.add(egui::Shape::mesh(mesh));
        p.add(egui::Shape::line((0..256).map(pt).collect(), egui::Stroke::new(1.0, stroke.gamma_multiply(0.9))));
    }
}

/// Plot the three channel curves (x = negative input, y = positive output).
pub fn curves_plot(ui: &mut Ui, samples: &[[f32; 4]], size: egui::Vec2) {
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    let p = ui.painter_at(rect);
    p.rect_filled(rect, 8.0, theme::card());
    for i in 1..4 {
        let t = i as f32 / 4.0;
        let s = egui::Stroke::new(0.5, theme::line());
        p.line_segment(
            [
                egui::pos2(rect.left() + rect.width() * t, rect.top()),
                egui::pos2(rect.left() + rect.width() * t, rect.bottom()),
            ],
            s,
        );
        p.line_segment(
            [
                egui::pos2(rect.left(), rect.top() + rect.height() * t),
                egui::pos2(rect.right(), rect.top() + rect.height() * t),
            ],
            s,
        );
    }
    let cols = [
        Color32::from_rgb(240, 80, 80),
        Color32::from_rgb(80, 220, 80),
        Color32::from_rgb(90, 140, 255),
    ];
    for c in 0..3 {
        let pts: Vec<egui::Pos2> = samples
            .iter()
            .map(|s| {
                egui::pos2(
                    rect.left() + s[0] * rect.width(),
                    rect.bottom() - s[c + 1] * rect.height(),
                )
            })
            .collect();
        p.add(egui::Shape::line(pts, egui::Stroke::new(1.5, cols[c])));
    }
}

/// Colours for the opposing-colour sliders.
pub mod grad {
    use eframe::egui::Color32;
    pub const TEMP: (Color32, Color32) = (
        Color32::from_rgb(60, 110, 230),
        Color32::from_rgb(230, 200, 40),
    );
    pub const TINT: (Color32, Color32) = (
        Color32::from_rgb(50, 190, 70),
        Color32::from_rgb(210, 60, 200),
    );
    pub const RED_CYAN: (Color32, Color32) = (
        Color32::from_rgb(220, 60, 60),
        Color32::from_rgb(40, 200, 210),
    );
    pub const GREEN_MAGENTA: (Color32, Color32) = (
        Color32::from_rgb(50, 190, 70),
        Color32::from_rgb(210, 60, 200),
    );
    pub const BLUE_YELLOW: (Color32, Color32) = (
        Color32::from_rgb(60, 110, 230),
        Color32::from_rgb(230, 200, 40),
    );
}

// ---- Light editor theme --------------------------------------------------------

pub mod theme {
    //! Design tokens: a light and a dark palette, San Francisco typography.
    use std::sync::atomic::{AtomicBool, Ordering};

    use eframe::egui::{self, Color32, CornerRadius, FontFamily, FontId, Stroke};

    static DARK: AtomicBool = AtomicBool::new(false);

    pub fn is_dark() -> bool {
        DARK.load(Ordering::Relaxed)
    }

    macro_rules! tok {
        ($($name:ident: $light:expr, $dark:expr;)*) => {
            $(
                #[inline]
                pub fn $name() -> Color32 {
                    if is_dark() { $dark } else { $light }
                }
            )*
        };
    }

    const fn c(r: u8, g: u8, b: u8) -> Color32 {
        Color32::from_rgb(r, g, b)
    }

    tok! {
        toolbar:      c(236, 236, 238), c(30, 31, 34);
        sidebar:      c(247, 247, 248), c(37, 38, 41);
        canvas:       c(142, 142, 145), c(50, 50, 53);
        surface:      c(255, 255, 255), c(46, 47, 51);
        text:         c(29, 29, 31),    c(236, 236, 240);
        muted:        c(110, 110, 116), c(152, 152, 160);
        faint:        c(170, 170, 176), c(98, 98, 106);
        line:         c(218, 218, 222), c(52, 53, 58);
        track:        c(214, 214, 218), c(70, 71, 77);
        track_fill:   c(72, 72, 78),    c(214, 214, 220);
        accent:       c(10, 116, 245),  c(64, 148, 255);
        warn:         c(190, 112, 0),   c(240, 172, 72);
        ok:           c(30, 142, 66),   c(92, 200, 124);
        hover:        c(222, 222, 226), c(52, 53, 58);
        pressed:      c(208, 208, 214), c(62, 63, 69);
        control_bg:   c(226, 226, 230), c(28, 29, 32);
        card:         c(239, 239, 241), c(44, 45, 49);
        card_hover:   c(230, 230, 233), c(52, 53, 58);
        selected_bg:  c(219, 233, 254), c(33, 52, 82);
        knob:         c(255, 255, 255), c(246, 246, 248);
        knob_ring:    c(120, 120, 126), c(30, 31, 34);
        input_stroke: c(200, 200, 205), c(64, 65, 71);
    }

    /// Semibold / medium font families (San Francisco when available).
    pub fn semibold(size: f32) -> FontId {
        FontId::new(size, FontFamily::Name("semibold".into()))
    }

    pub fn medium(size: f32) -> FontId {
        FontId::new(size, FontFamily::Name("medium".into()))
    }

    /// Load San Francisco (system font, not bundled) in three weights.
    pub fn install_fonts(ctx: &egui::Context) {
        let mut fonts = egui::FontDefinitions::default();
        let sf = std::fs::read("/System/Library/Fonts/SFNS.ttf").ok();
        let mono = std::fs::read("/System/Library/Fonts/SFNSMono.ttf").ok();
        let weighted = |bytes: &Vec<u8>, w: f32| {
            let mut fd = egui::FontData::from_owned(bytes.clone());
            fd.tweak.coords = egui::epaint::text::VariationCoords::new([(b"wght", w)]);
            std::sync::Arc::new(fd)
        };
        let fallback = fonts.families.get(&FontFamily::Proportional).cloned().unwrap_or_default();
        let mut add = |family: FontFamily, key: &str, data: Option<std::sync::Arc<egui::FontData>>| {
            let mut list = Vec::new();
            if let Some(d) = data {
                fonts.font_data.insert(key.to_string(), d);
                list.push(key.to_string());
            }
            list.extend(fallback.iter().cloned());
            fonts.families.insert(family, list);
        };
        add(FontFamily::Proportional, "sf-regular", sf.as_ref().map(|b| weighted(b, 430.0)));
        add(FontFamily::Name("medium".into()), "sf-medium", sf.as_ref().map(|b| weighted(b, 530.0)));
        add(FontFamily::Name("semibold".into()), "sf-semibold", sf.as_ref().map(|b| weighted(b, 640.0)));
        if let Some(m) = mono {
            fonts.font_data.insert("sf-mono".into(), std::sync::Arc::new(egui::FontData::from_owned(m)));
            fonts.families.entry(FontFamily::Monospace).or_default().insert(0, "sf-mono".into());
        }
        ctx.set_fonts(fonts);
    }

    /// Apply the palette for light or dark mode.
    pub fn apply(ctx: &egui::Context, dark: bool) {
        DARK.store(dark, Ordering::Relaxed);
        let mut v = if dark { egui::Visuals::dark() } else { egui::Visuals::light() };
        v.panel_fill = sidebar();
        v.window_fill = surface();
        v.window_stroke = Stroke::new(1.0, line());
        v.extreme_bg_color = if dark { c(28, 29, 32) } else { Color32::WHITE };
        v.faint_bg_color = card();
        v.override_text_color = Some(text());
        v.weak_text_color = Some(muted());
        v.hyperlink_color = accent();
        v.selection.bg_fill = accent();
        v.selection.stroke = Stroke::new(1.0, Color32::WHITE);
        v.window_corner_radius = CornerRadius::same(12);
        v.menu_corner_radius = CornerRadius::same(10);
        v.popup_shadow = egui::Shadow { offset: [0, 6], blur: 24, spread: 0, color: Color32::from_black_alpha(if dark { 120 } else { 40 }) };
        v.window_shadow = v.popup_shadow;
        let r = CornerRadius::same(7);
        for w in [&mut v.widgets.noninteractive, &mut v.widgets.inactive, &mut v.widgets.hovered, &mut v.widgets.active, &mut v.widgets.open] {
            w.corner_radius = r;
            w.expansion = 0.0;
        }
        v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, line());
        v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, text());
        v.widgets.inactive.weak_bg_fill = surface();
        v.widgets.inactive.bg_fill = surface();
        v.widgets.inactive.bg_stroke = Stroke::new(1.0, input_stroke());
        v.widgets.inactive.fg_stroke = Stroke::new(1.2, text());
        v.widgets.hovered.weak_bg_fill = hover();
        v.widgets.hovered.bg_fill = hover();
        v.widgets.hovered.bg_stroke = Stroke::new(1.0, faint());
        v.widgets.hovered.fg_stroke = Stroke::new(1.3, text());
        v.widgets.active.weak_bg_fill = pressed();
        v.widgets.active.bg_fill = pressed();
        v.widgets.active.fg_stroke = Stroke::new(1.3, text());
        v.widgets.open.weak_bg_fill = surface();
        v.widgets.open.bg_stroke = Stroke::new(1.0, accent());
        ctx.set_visuals(v);
        ctx.global_style_mut(|s| {
            s.spacing.item_spacing = egui::vec2(8.0, 6.0);
            s.spacing.button_padding = egui::vec2(12.0, 5.0);
            s.spacing.interact_size.y = 28.0;
            s.spacing.menu_margin = egui::Margin::same(6);
            for (ts, size) in [(egui::TextStyle::Body, 13.0), (egui::TextStyle::Button, 13.0), (egui::TextStyle::Small, 11.0), (egui::TextStyle::Heading, 20.0)] {
                if let Some(f) = s.text_styles.get_mut(&ts) {
                    f.size = size;
                }
            }
            if let Some(f) = s.text_styles.get_mut(&egui::TextStyle::Heading) {
                f.family = FontFamily::Name("semibold".into());
            }
        });
    }
}

// ---- Vector icons ----------------------------------------------------------------

#[derive(Clone, Copy, PartialEq)]
pub enum Icon {
    Back,
    Prev,
    Next,
    ZoomOut,
    ZoomIn,
    Undo,
    Redo,
    Compare,
    RotateLeft,
    RotateRight,
    Flip,
    Crop,
    Export,
    Eyedropper,
    Folder,
}

fn arc(c: egui::Pos2, r: f32, a0: f32, a1: f32, n: usize) -> Vec<egui::Pos2> {
    (0..=n)
        .map(|i| {
            let a = a0 + (a1 - a0) * i as f32 / n as f32;
            egui::pos2(c.x + r * a.cos(), c.y + r * a.sin())
        })
        .collect()
}

pub fn paint_icon(p: &egui::Painter, rect: Rect, icon: Icon, color: Color32) {
    use egui::{Shape, Stroke, pos2, vec2};
    let st = Stroke::new(1.7, color);
    let c = rect.center();
    let s = rect.width().min(rect.height()) / 20.0; // unit
    let q = |x: f32, y: f32| pos2(c.x + x * s, c.y + y * s);
    let line = |pts: Vec<egui::Pos2>| Shape::line(pts, st);
    let arrow_head = |tip: egui::Pos2, dir: egui::Vec2| {
        let d = dir.normalized() * 4.0 * s;
        let n = vec2(-d.y, d.x);
        line(vec![tip - d + n, tip, tip - d - n])
    };
    match icon {
        Icon::Back => {
            p.add(line(vec![q(7.0, 0.0), q(-7.0, 0.0)]));
            p.add(line(vec![q(-1.5, -5.5), q(-7.0, 0.0), q(-1.5, 5.5)]));
        }
        Icon::Prev => {
            p.add(line(vec![q(3.0, -7.0), q(-4.0, 0.0), q(3.0, 7.0)]));
        }
        Icon::Next => {
            p.add(line(vec![q(-3.0, -7.0), q(4.0, 0.0), q(-3.0, 7.0)]));
        }
        Icon::ZoomIn | Icon::ZoomOut => {
            p.circle_stroke(q(-1.5, -1.5), 6.0 * s, st);
            p.add(line(vec![q(3.0, 3.0), q(7.5, 7.5)]));
            p.add(line(vec![q(-4.5, -1.5), q(1.5, -1.5)]));
            if icon == Icon::ZoomIn {
                p.add(line(vec![q(-1.5, -4.5), q(-1.5, 1.5)]));
            }
        }
        Icon::Undo | Icon::Redo => {
            let flip = if icon == Icon::Undo { 1.0 } else { -1.0 };
            let pts: Vec<_> = arc(
                q(0.0, 2.0),
                6.0 * s,
                std::f32::consts::PI * 1.0,
                std::f32::consts::PI * 2.0,
                16,
            )
            .into_iter()
            .map(|pt| pos2(c.x + (pt.x - c.x) * flip, pt.y))
            .collect();
            let tip = pts[0];
            p.add(line(pts));
            p.add(arrow_head(tip + vec2(0.0, 1.0 * s), vec2(0.0, 1.0)));
        }
        Icon::RotateLeft | Icon::RotateRight => {
            let flip = if icon == Icon::RotateLeft { -1.0 } else { 1.0 };
            let pts: Vec<_> = arc(c, 7.0 * s, -1.2, 4.0, 24)
                .into_iter()
                .map(|pt| pos2(c.x + (pt.x - c.x) * flip, pt.y))
                .collect();
            let tip = *pts.last().unwrap();
            let prev = pts[pts.len() - 3];
            p.add(line(pts));
            p.add(arrow_head(tip, tip - prev));
        }
        Icon::Flip => {
            for k in 0..5 {
                let y0 = -8.0 + k as f32 * 3.6;
                p.add(line(vec![q(0.0, y0), q(0.0, y0 + 1.8)]));
            }
            p.add(Shape::closed_line(
                vec![q(-2.5, -6.0), q(-2.5, 6.0), q(-8.5, 0.0)],
                st,
            ));
            p.add(Shape::convex_polygon(
                vec![q(2.5, -6.0), q(8.5, 0.0), q(2.5, 6.0)],
                color,
                Stroke::NONE,
            ));
        }
        Icon::Crop => {
            p.add(line(vec![q(-4.0, -8.0), q(-4.0, 4.0), q(8.0, 4.0)]));
            p.add(line(vec![q(-8.0, -4.0), q(4.0, -4.0), q(4.0, 8.0)]));
        }
        Icon::Export => {
            p.add(line(vec![
                q(-4.0, -2.0),
                q(-7.0, -2.0),
                q(-7.0, 8.0),
                q(7.0, 8.0),
                q(7.0, -2.0),
                q(4.0, -2.0),
            ]));
            p.add(line(vec![q(0.0, 3.0), q(0.0, -8.0)]));
            p.add(line(vec![q(-3.5, -4.5), q(0.0, -8.0), q(3.5, -4.5)]));
        }
        Icon::Compare => {
            let r = Rect::from_center_size(c, vec2(15.0 * s, 13.0 * s));
            p.rect_stroke(r, 2.0, st, egui::StrokeKind::Middle);
            p.add(Shape::convex_polygon(
                vec![
                    r.left_top(),
                    pos2(r.center().x, r.top()),
                    pos2(r.center().x, r.bottom()),
                    r.left_bottom(),
                ],
                color,
                Stroke::NONE,
            ));
        }
        Icon::Folder => {
            p.add(Shape::closed_line(
                vec![
                    q(-8.0, 6.5),
                    q(-8.0, -6.0),
                    q(-3.0, -6.0),
                    q(-1.5, -4.0),
                    q(6.0, -4.0),
                    q(6.0, -1.5),
                ],
                st,
            ));
            p.add(Shape::convex_polygon(
                vec![q(-8.0, 6.5), q(-5.0, -1.5), q(9.0, -1.5), q(6.0, 6.5)],
                color,
                Stroke::NONE,
            ));
        }
        Icon::Eyedropper => {
            p.add(line(vec![q(-7.0, 7.0), q(3.0, -3.0)]));
            p.add(Shape::line(
                vec![q(1.0, -5.0), q(5.0, -1.0)],
                Stroke::new(3.0, color),
            ));
            p.circle_filled(q(5.0, -5.0), 2.5 * s, color);
        }
    }
}

/// Toolbar icon button.
pub fn icon_button(
    ui: &mut Ui,
    icon: Icon,
    tip: &str,
    selected: bool,
    enabled: bool,
) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(
        egui::vec2(34.0, 30.0),
        if enabled {
            Sense::click()
        } else {
            Sense::hover()
        },
    );
    let p = ui.painter();
    if selected {
        p.rect_filled(rect, 7.0, theme::pressed());
    } else if resp.hovered() && enabled {
        p.rect_filled(rect, 7.0, theme::hover());
    }
    let col = if !enabled {
        theme::faint()
    } else if selected {
        theme::accent()
    } else {
        theme::text()
    };
    paint_icon(p, rect.shrink2(egui::vec2(7.0, 5.0)), icon, col);
    resp.on_hover_text(tip)
}

/// Bold caps section header.
pub fn header(ui: &mut Ui, title: &str) {
    ui.add_space(14.0);
    ui.label(
        egui::RichText::new(title)
            .strong()
            .size(12.5)
            .color(theme::text()),
    );
    ui.add_space(4.0);
}

/// Label-above dropdown, full width.
pub fn fl_combo<T: Copy + PartialEq>(
    ui: &mut Ui,
    label: &str,
    v: &mut T,
    all: &[T],
    name: impl Fn(T) -> &'static str,
    tip: &str,
) -> bool {
    let mut changed = false;
    if !label.is_empty() {
        let l = ui.label(egui::RichText::new(label).color(theme::text()));
        if !tip.is_empty() {
            l.on_hover_text(tip);
        }
    }
    egui::ComboBox::from_id_salt(("fl_combo", label, std::any::type_name::<T>(), ui.id()))
        .width((ui.available_width() - 4.0).max(60.0))
        .height(400.0)
        .selected_text(name(*v))
        .show_ui(ui, |ui| {
            for &o in all {
                if ui.selectable_label(*v == o, name(o)).clicked() && *v != o {
                    *v = o;
                    changed = true;
                }
            }
        });
    ui.add_space(4.0);
    changed
}

/// Slim slider: label + value on one row, track below. Double-click the
/// track or click the label to reset; the value can be typed past the range.
pub fn fl_slider(
    ui: &mut Ui,
    label: &str,
    v: &mut f32,
    range: std::ops::RangeInclusive<f32>,
    default: f32,
    decimals: usize,
    tip: &str,
    gradient: Option<(Color32, Color32)>,
) -> bool {
    let mut changed = false;
    let (lo, hi) = (*range.start(), *range.end());
    ui.horizontal(|ui| {
        let l = ui.add(
            egui::Label::new(egui::RichText::new(label).color(theme::text())).sense(Sense::click()),
        );
        let l = l.on_hover_text(if tip.is_empty() {
            "Click to reset".to_string()
        } else {
            format!("{tip}\n\nClick the label to reset.")
        });
        if l.clicked() && *v != default {
            *v = default;
            changed = true;
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let fmt = move |x: f64, _r: std::ops::RangeInclusive<usize>| {
                let s = format!("{:.*}", decimals, x);
                if x > 0.0 && default == 0.0 {
                    format!("+{s}")
                } else {
                    s
                }
            };
            // Frameless value, editable by click (type) or horizontal drag.
            let vis = &mut ui.visuals_mut().widgets;
            for w in [&mut vis.inactive, &mut vis.hovered] {
                w.bg_stroke = egui::Stroke::NONE;
                w.weak_bg_fill = Color32::TRANSPARENT;
                w.bg_fill = Color32::TRANSPARENT;
            }
            let dv = egui::DragValue::new(v)
                .speed((hi - lo) as f64 / 400.0)
                .max_decimals(decimals)
                .custom_formatter(fmt);
            if ui.add(dv.update_while_editing(false)).changed() {
                changed = true;
            }
        });
    });
    let w = ui.available_width() - 6.0;
    ui.add_space(-6.0);
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(w, 20.0), Sense::click_and_drag());
    let track = Rect::from_min_max(
        egui::pos2(rect.left() + 9.0, rect.center().y - 1.0),
        egui::pos2(rect.right() - 9.0, rect.center().y + 1.0),
    );
    let to_x = |val: f32| track.left() + track.width() * ((val - lo) / (hi - lo)).clamp(0.0, 1.0);
    if resp.double_clicked() {
        *v = default;
        changed = true;
    } else if resp.dragged() || resp.clicked() {
        if let Some(pos) = resp.interact_pointer_pos() {
            let t = ((pos.x - track.left()) / track.width()).clamp(0.0, 1.0);
            let mut nv = lo + t * (hi - lo);
            if decimals == 0 {
                nv = nv.round();
            } else {
                let m = 10f32.powi(decimals as i32);
                nv = (nv * m).round() / m;
            }
            if (nv - *v).abs() > 1e-6 {
                *v = nv;
                changed = true;
            }
        }
    }
    let p = ui.painter();
    match gradient {
        Some((a, b)) => gradient_rect(ui, track.expand2(egui::vec2(0.0, 1.0)), a, b),
        None => {
            p.rect_filled(track, 1.0, theme::track());
            let z = to_x(default.clamp(lo, hi));
            let k = to_x(*v);
            let fill = Rect::from_min_max(
                egui::pos2(z.min(k), track.top()),
                egui::pos2(z.max(k), track.bottom()),
            );
            p.rect_filled(fill, 1.0, theme::track_fill());
        }
    }
    let knob = egui::pos2(to_x(*v), track.center().y);
    let hot = resp.hovered() || resp.dragged();
    // Soft shadow + knob.
    p.circle_filled(knob + egui::vec2(0.0, 1.0), 8.5, Color32::from_black_alpha(if theme::is_dark() { 90 } else { 30 }));
    p.circle(
        knob,
        if hot { 8.5 } else { 8.0 },
        theme::knob(),
        egui::Stroke::new(if hot { 1.5 } else { 1.0 }, if hot { theme::accent() } else { theme::knob_ring() }),
    );
    ui.add_space(4.0);
    changed
}

pub fn fl_slider_u8(
    ui: &mut Ui,
    label: &str,
    v: &mut u8,
    range: std::ops::RangeInclusive<u8>,
    default: u8,
    tip: &str,
) -> bool {
    let mut f = *v as f32;
    let ch = fl_slider(
        ui,
        label,
        &mut f,
        *range.start() as f32..=*range.end() as f32,
        default as f32,
        0,
        tip,
        None,
    );
    if ch {
        *v = f.round().clamp(*range.start() as f32, *range.end() as f32) as u8;
    }
    ch
}

/// Segmented control (pill buttons).
pub fn segmented<T: Copy + PartialEq>(ui: &mut Ui, v: &mut T, items: &[(T, &str)]) -> bool {
    segmented_locked(ui, v, items, &[], "")
}

/// Segmented control where the options in `locked` are greyed out and not
/// selectable (`lock_tip` explains why on hover).
pub fn segmented_locked<T: Copy + PartialEq>(
    ui: &mut Ui,
    v: &mut T,
    items: &[(T, &str)],
    locked: &[T],
    lock_tip: &str,
) -> bool {
    let mut changed = false;
    let w = (ui.available_width() - 6.0) / items.len() as f32;
    egui::Frame::new()
        .fill(theme::control_bg())
        .corner_radius(8.0)
        .inner_margin(2.0)
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            ui.horizontal(|ui| {
                for (t, name) in items {
                    let sel = *v == *t;
                    let is_locked = locked.contains(t);
                    let (rect, resp) = ui.allocate_exact_size(
                        egui::vec2(w - 1.0, 24.0),
                        if is_locked {
                            Sense::hover()
                        } else {
                            Sense::click()
                        },
                    );
                    let resp = if is_locked && !lock_tip.is_empty() {
                        resp.on_hover_text(lock_tip)
                    } else {
                        resp
                    };
                    if sel {
                        ui.painter().rect_filled(rect.translate(egui::vec2(0.0, 0.5)), 7.0, Color32::from_black_alpha(if theme::is_dark() { 60 } else { 18 }));
                        ui.painter().rect(rect, 7.0, theme::surface(), egui::Stroke::new(0.5, theme::line()), egui::StrokeKind::Inside);
                    }
                    ui.painter().text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        *name,
                        if sel { theme::medium(12.5) } else { egui::FontId::proportional(12.5) },
                        if is_locked {
                            theme::faint()
                        } else if sel {
                            theme::text()
                        } else {
                            theme::muted()
                        },
                    );
                    if resp.clicked() && !sel && !is_locked {
                        *v = *t;
                        changed = true;
                    }
                }
            });
        });
    changed
}

/// Collapsible caps section header with a chevron. Returns whether the
/// section is open; the state is remembered per title.
pub fn fold(ui: &mut Ui, title: &str) -> bool {
    let id = ui.make_persistent_id(("fold", title));
    let mut open = ui.data_mut(|d| *d.get_persisted_mut_or(id, true));
    ui.add_space(12.0);
    let (rect, resp) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), 20.0), Sense::click());
    let hot = resp.hovered();
    let col = if hot { theme::accent() } else { theme::text() };
    let p = ui.painter();
    p.text(
        rect.left_center(),
        egui::Align2::LEFT_CENTER,
        title,
        egui::FontId::proportional(12.5),
        col,
    );
    let c = egui::pos2(rect.right() - 8.0, rect.center().y);
    let pts = if open {
        vec![
            c + egui::vec2(-4.0, -2.0),
            c + egui::vec2(0.0, 2.0),
            c + egui::vec2(4.0, -2.0),
        ]
    } else {
        vec![
            c + egui::vec2(-2.0, -4.0),
            c + egui::vec2(2.0, 0.0),
            c + egui::vec2(-2.0, 4.0),
        ]
    };
    p.add(egui::Shape::line(pts, egui::Stroke::new(1.6, col)));
    if resp.clicked() {
        open = !open;
        ui.data_mut(|d| d.insert_persisted(id, open));
    }
    if hot {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    if open {
        ui.add_space(4.0);
    } else {
        ui.painter().line_segment(
            [
                egui::pos2(rect.left(), rect.bottom() + 3.0),
                egui::pos2(rect.right(), rect.bottom() + 3.0),
            ],
            egui::Stroke::new(0.5, theme::line()),
        );
    }
    open
}
