//! Shared controls and the dark graphite theme. Controls follow the plugin's
//! conventions: click a slider label (or double-click its track) to reset,
//! and typed values may exceed the slider range.

use eframe::egui::{self, Color32, Rect, Sense, Ui};

// ---- Theme ------------------------------------------------------------------------

pub mod theme {
    use std::sync::Arc;

    use eframe::egui::{self, Color32, CornerRadius, FontFamily, FontId, Shadow, Stroke};

    /// App chrome behind everything.
    pub const BG: Color32 = Color32::from_rgb(28, 28, 30);
    /// Toolbars and the sidebar.
    pub const PANEL: Color32 = Color32::from_rgb(35, 35, 38);
    /// Neutral surround for judging the image.
    pub const CANVAS: Color32 = Color32::from_rgb(42, 42, 45);
    /// Inputs, buttons, selected states.
    pub const SURFACE: Color32 = Color32::from_rgb(47, 47, 51);
    pub const HOVER: Color32 = Color32::from_rgb(56, 56, 61);
    pub const PRESSED: Color32 = Color32::from_rgb(66, 66, 72);
    /// Hairlines (white at ~7 %).
    pub const LINE: Color32 = Color32::from_rgba_premultiplied(18, 18, 18, 18);
    pub const TEXT: Color32 = Color32::from_rgb(232, 232, 234);
    pub const MUTED: Color32 = Color32::from_rgb(142, 142, 147);
    pub const FAINT: Color32 = Color32::from_rgb(90, 90, 96);
    pub const TRACK: Color32 = Color32::from_rgb(64, 64, 70);
    pub const ACCENT: Color32 = Color32::from_rgb(76, 141, 255);
    pub const OK: Color32 = Color32::from_rgb(98, 186, 120);
    pub const WARN: Color32 = Color32::from_rgb(222, 168, 72);
    pub const ERR: Color32 = Color32::from_rgb(236, 96, 84);

    const MEDIUM: &str = "medium";

    /// Inter Medium at `size` (titles, section headers).
    pub fn medium(size: f32) -> FontId {
        FontId::new(size, FontFamily::Name(MEDIUM.into()))
    }

    /// Embed Inter so text renders identically on every OS. egui's own
    /// fonts stay behind it as fallback (emoji, rare symbols).
    pub fn install_fonts(ctx: &egui::Context) {
        let mut f = egui::FontDefinitions::default();
        f.font_data.insert(
            "Inter".into(),
            Arc::new(egui::FontData::from_static(include_bytes!(
                "../../assets/fonts/Inter-Regular.ttf"
            ))),
        );
        f.font_data.insert(
            "Inter-Medium".into(),
            Arc::new(egui::FontData::from_static(include_bytes!(
                "../../assets/fonts/Inter-Medium.ttf"
            ))),
        );
        let prop = f.families.entry(FontFamily::Proportional).or_default();
        let fallback = prop.clone();
        prop.insert(0, "Inter".into());
        let mut medium = vec!["Inter-Medium".to_string()];
        medium.extend(fallback);
        f.families.insert(FontFamily::Name(MEDIUM.into()), medium);
        ctx.set_fonts(f);
    }

    fn visuals() -> egui::Visuals {
        let mut v = egui::Visuals::dark();
        v.panel_fill = PANEL;
        v.window_fill = PANEL;
        v.window_stroke = Stroke::new(1.0, LINE);
        v.extreme_bg_color = BG;
        v.faint_bg_color = SURFACE;
        v.code_bg_color = SURFACE;
        v.hyperlink_color = ACCENT;
        v.warn_fg_color = WARN;
        v.error_fg_color = ERR;
        v.selection.bg_fill = ACCENT.gamma_multiply(0.55);
        v.selection.stroke = Stroke::new(1.0, TEXT);
        v.window_corner_radius = CornerRadius::same(12);
        v.menu_corner_radius = CornerRadius::same(8);
        v.window_shadow = Shadow {
            offset: [0, 10],
            blur: 32,
            spread: 0,
            color: Color32::from_black_alpha(120),
        };
        v.popup_shadow = Shadow {
            offset: [0, 4],
            blur: 16,
            spread: 0,
            color: Color32::from_black_alpha(100),
        };
        v.handle_shape = egui::style::HandleShape::Circle;
        let w = &mut v.widgets;
        w.noninteractive.bg_fill = PANEL;
        w.noninteractive.weak_bg_fill = PANEL;
        w.noninteractive.bg_stroke = Stroke::new(1.0, LINE);
        w.noninteractive.fg_stroke = Stroke::new(1.0, TEXT);
        w.inactive.bg_fill = SURFACE;
        w.inactive.weak_bg_fill = SURFACE;
        w.inactive.bg_stroke = Stroke::NONE;
        w.inactive.fg_stroke = Stroke::new(1.2, TEXT);
        w.hovered.bg_fill = HOVER;
        w.hovered.weak_bg_fill = HOVER;
        w.hovered.bg_stroke = Stroke::NONE;
        w.hovered.fg_stroke = Stroke::new(1.4, TEXT);
        w.active.bg_fill = PRESSED;
        w.active.weak_bg_fill = PRESSED;
        w.active.bg_stroke = Stroke::NONE;
        w.active.fg_stroke = Stroke::new(1.4, TEXT);
        w.open.bg_fill = HOVER;
        w.open.weak_bg_fill = HOVER;
        w.open.bg_stroke = Stroke::NONE;
        w.open.fg_stroke = Stroke::new(1.2, TEXT);
        for s in [
            &mut w.noninteractive,
            &mut w.inactive,
            &mut w.hovered,
            &mut w.active,
            &mut w.open,
        ] {
            s.corner_radius = CornerRadius::same(6);
            s.expansion = 0.0;
        }
        v
    }

    /// Install fonts and the dark graphite style. Both egui themes get the
    /// same visuals so the OS light/dark setting cannot switch them.
    pub fn apply(ctx: &egui::Context) {
        install_fonts(ctx);
        ctx.set_theme(egui::Theme::Dark);
        let v = visuals();
        ctx.all_styles_mut(|s| {
            s.visuals = v.clone();
            s.spacing.item_spacing = egui::vec2(8.0, 6.0);
            s.spacing.button_padding = egui::vec2(12.0, 4.0);
            s.spacing.interact_size.y = 26.0;
            s.spacing.menu_margin = egui::Margin::same(6);
            s.spacing.window_margin = egui::Margin::same(20);
            use egui::TextStyle::*;
            for (style, font) in [
                (Body, FontId::proportional(13.0)),
                (Button, FontId::proportional(13.0)),
                (Small, FontId::proportional(11.0)),
                (Heading, medium(17.0)),
                (Monospace, FontId::monospace(12.0)),
            ] {
                s.text_styles.insert(style, font);
            }
        });
    }
}

/// Platform-correct label for a shortcut (`⌘Z` on macOS, `Ctrl+Z` elsewhere).
pub fn shortcut(ctx: &egui::Context, mods: egui::Modifiers, key: egui::Key) -> String {
    ctx.format_shortcut(&egui::KeyboardShortcut::new(mods, key))
}

/// Accent-filled call-to-action button.
pub fn primary<'a>(text: &str) -> egui::Button<'a> {
    egui::Button::new(
        egui::RichText::new(text)
            .font(theme::medium(13.0))
            .color(Color32::WHITE),
    )
    .fill(theme::ACCENT)
    .corner_radius(6.0)
    .min_size(egui::vec2(0.0, 28.0))
}

/// Quiet secondary button.
pub fn ghost<'a>(text: &str) -> egui::Button<'a> {
    egui::Button::new(egui::RichText::new(text).color(theme::TEXT))
        .corner_radius(6.0)
        .min_size(egui::vec2(0.0, 28.0))
}

/// Small muted icon + text line (status messages).
pub fn status_line(ui: &mut Ui, icon: Icon, color: Color32, text: &str) -> egui::Response {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 6.0;
        let (r, _) = ui.allocate_exact_size(egui::vec2(14.0, 14.0), Sense::hover());
        paint_icon(ui.painter(), r, icon, color);
        ui.add(egui::Label::new(egui::RichText::new(text).small().color(theme::MUTED)).wrap());
    })
    .response
}

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

/// Paint an RGB histogram (counts per channel, 256 bins) as additive
/// translucent fills, so overlapping channels sum towards white.
pub fn histogram(ui: &mut Ui, h: &[[u32; 256]; 3], size: egui::Vec2) {
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    let p = ui.painter_at(rect);
    p.rect_filled(rect, 6.0, theme::BG);
    let plot = rect.shrink2(egui::vec2(6.0, 6.0));
    let max = h
        .iter()
        .flat_map(|c| c[2..254].iter())
        .copied()
        .max()
        .unwrap_or(1)
        .max(1) as f32;
    let cols = [(235u8, 72u8, 72u8), (72, 200, 96), (80, 130, 255)];
    for (c, &(r, g, b)) in cols.iter().enumerate() {
        let y = |v: u32| plot.bottom() - plot.height() * (v as f32 / max).sqrt().min(1.0);
        let x = |i: usize| plot.left() + plot.width() * i as f32 / 255.0;
        // Additive colour: alpha 0 in premultiplied space.
        let fill = Color32::from_rgba_premultiplied(
            (r as f32 * 0.38) as u8,
            (g as f32 * 0.38) as u8,
            (b as f32 * 0.38) as u8,
            0,
        );
        let mut mesh = egui::Mesh::default();
        for (i, &v) in h[c].iter().enumerate() {
            mesh.colored_vertex(egui::pos2(x(i), plot.bottom()), fill);
            mesh.colored_vertex(egui::pos2(x(i), y(v)), fill);
            if i > 0 {
                let k = (i * 2) as u32;
                mesh.add_triangle(k - 2, k - 1, k);
                mesh.add_triangle(k - 1, k, k + 1);
            }
        }
        p.add(egui::Shape::mesh(mesh));
        let pts: Vec<_> = h[c]
            .iter()
            .enumerate()
            .map(|(i, &v)| egui::pos2(x(i), y(v)))
            .collect();
        p.add(egui::Shape::line(
            pts,
            egui::Stroke::new(
                1.0,
                Color32::from_rgba_premultiplied(r / 2, g / 2, b / 2, 0),
            ),
        ));
    }
    // Clipping warnings: a corner triangle in the clipped channels' colour
    // when more than 0.1 % of the pixels sit at 0 or 255.
    let total = h[0].iter().sum::<u32>().max(1) as f32;
    for (end, left) in [(0usize, true), (255, false)] {
        let mut col = [0u8; 3];
        for c in 0..3 {
            if h[c][end] as f32 / total > 0.001 {
                col[c] = 255;
            }
        }
        if col == [0; 3] {
            continue;
        }
        let fill = if col == [255; 3] {
            Color32::WHITE
        } else {
            Color32::from_rgb(col[0].max(60), col[1].max(60), col[2].max(60))
        };
        let (x0, dx) = if left {
            (rect.left() + 3.0, 7.0)
        } else {
            (rect.right() - 3.0, -7.0)
        };
        let y0 = rect.top() + 3.0;
        p.add(egui::Shape::convex_polygon(
            vec![
                egui::pos2(x0, y0),
                egui::pos2(x0 + dx, y0),
                egui::pos2(x0, y0 + 7.0),
            ],
            fill,
            egui::Stroke::NONE,
        ));
    }
}

/// Plot the three channel curves (x = negative input, y = positive output).
pub fn curves_plot(ui: &mut Ui, samples: &[[f32; 4]], size: egui::Vec2) {
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    let p = ui.painter_at(rect);
    p.rect_filled(rect, 6.0, theme::BG);
    for i in 1..4 {
        let t = i as f32 / 4.0;
        let s = egui::Stroke::new(1.0, theme::LINE);
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
        Color32::from_rgb(235, 88, 88),
        Color32::from_rgb(88, 205, 110),
        Color32::from_rgb(96, 145, 255),
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

/// Colours for the opposing-colour sliders (muted for the dark theme).
pub mod grad {
    use eframe::egui::Color32;
    const BLUE: Color32 = Color32::from_rgb(70, 112, 200);
    const YELLOW: Color32 = Color32::from_rgb(208, 184, 70);
    const GREEN: Color32 = Color32::from_rgb(72, 168, 88);
    const MAGENTA: Color32 = Color32::from_rgb(190, 78, 182);
    pub const TEMP: (Color32, Color32) = (BLUE, YELLOW);
    pub const TINT: (Color32, Color32) = (GREEN, MAGENTA);
    pub const RED_CYAN: (Color32, Color32) = (
        Color32::from_rgb(200, 76, 76),
        Color32::from_rgb(62, 180, 190),
    );
    pub const GREEN_MAGENTA: (Color32, Color32) = (GREEN, MAGENTA);
    pub const BLUE_YELLOW: (Color32, Color32) = (BLUE, YELLOW);
}

// ---- Vector icons ----------------------------------------------------------------

#[derive(Clone, Copy, PartialEq)]
pub enum Icon {
    Back,
    Prev,
    Next,
    Undo,
    Redo,
    Compare,
    RotateLeft,
    RotateRight,
    Flip,
    Crop,
    Lock,
    Unlock,
    Export,
    Eyedropper,
    Folder,
    Warn,
    More,
    Close,
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
    let c = rect.center();
    let s = rect.width().min(rect.height()) / 20.0; // unit
    let st = Stroke::new((1.6 * s * 1.25).clamp(1.2, 1.6), color);
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
        Icon::Lock | Icon::Unlock => {
            let locked = icon == Icon::Lock;
            let body = Rect::from_min_max(q(-6.0, -1.0), q(6.0, 8.0));
            p.rect_stroke(body, 1.5, st, egui::StrokeKind::Middle);
            let lift = if locked { 0.0 } else { -3.0 };
            let mut pts = arc(q(0.0, -1.0 + lift), 4.0 * s, std::f32::consts::PI, std::f32::consts::TAU, 10);
            if locked {
                pts.push(q(4.0, -1.0));
            }
            pts.insert(0, q(-4.0, -1.0));
            p.add(line(pts));
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
                    q(8.0, -4.0),
                    q(8.0, 6.5),
                ],
                st,
            ));
            p.add(line(vec![q(-8.0, -1.0), q(8.0, -1.0)]));
        }
        Icon::Eyedropper => {
            p.add(line(vec![q(-7.0, 7.0), q(3.0, -3.0)]));
            p.add(Shape::line(
                vec![q(1.0, -5.0), q(5.0, -1.0)],
                Stroke::new(3.0 * s.max(0.6), color),
            ));
            p.circle_filled(q(5.0, -5.0), 2.5 * s, color);
        }
        Icon::Warn => {
            p.add(Shape::closed_line(
                vec![q(0.0, -8.0), q(8.5, 7.0), q(-8.5, 7.0)],
                st,
            ));
            p.add(line(vec![q(0.0, -2.5), q(0.0, 2.0)]));
            p.circle_filled(q(0.0, 4.6), 1.0 * s.max(0.8), color);
        }
        Icon::More => {
            for x in [-6.0, 0.0, 6.0] {
                p.circle_filled(q(x, 0.0), 1.6 * s.max(0.8), color);
            }
        }
        Icon::Close => {
            p.line_segment([q(-5.0, -5.0), q(5.0, 5.0)], st);
            p.line_segment([q(-5.0, 5.0), q(5.0, -5.0)], st);
        }
    }
}

/// Toolbar icon button: muted glyph that brightens on hover; `selected`
/// shows a soft surface behind it.
pub fn icon_button(
    ui: &mut Ui,
    icon: Icon,
    tip: &str,
    selected: bool,
    enabled: bool,
) -> egui::Response {
    icon_button_tinted(ui, icon, tip, selected, enabled, None)
}

/// [`icon_button`] with a fixed glyph colour (e.g. the accent export icon).
pub fn icon_button_tinted(
    ui: &mut Ui,
    icon: Icon,
    tip: &str,
    selected: bool,
    enabled: bool,
    tint: Option<Color32>,
) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(
        egui::vec2(32.0, 28.0),
        if enabled {
            Sense::click()
        } else {
            Sense::hover()
        },
    );
    let p = ui.painter();
    let hot = resp.hovered() && enabled;
    if selected {
        p.rect_filled(rect, 6.0, theme::HOVER);
    } else if hot {
        p.rect_filled(rect, 6.0, theme::SURFACE);
    }
    let col = match (enabled, tint) {
        (false, _) => theme::FAINT,
        (true, Some(t)) => t,
        (true, None) if hot || selected => theme::TEXT,
        (true, None) => theme::MUTED,
    };
    paint_icon(p, rect.shrink2(egui::vec2(8.0, 6.0)), icon, col);
    if hot {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resp.on_hover_text(tip)
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
        let l = ui.label(egui::RichText::new(label).small().color(theme::MUTED));
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
/// `origin` optionally defines where the track fill highlight starts (defaulting to `default`).
pub fn fl_slider_origin(
    ui: &mut Ui,
    label: &str,
    v: &mut f32,
    range: std::ops::RangeInclusive<f32>,
    default: f32,
    origin: Option<f32>,
    decimals: usize,
    tip: &str,
    gradient: Option<(Color32, Color32)>,
) -> bool {
    fl_slider_active(
        ui, label, v, range, default, origin, decimals, tip, gradient,
    )
    .0
}

/// `fl_slider_origin` that also reports whether the slider is being held
/// (track drag or value drag): `(changed, active)`.
#[allow(clippy::too_many_arguments)]
pub fn fl_slider_active(
    ui: &mut Ui,
    label: &str,
    v: &mut f32,
    range: std::ops::RangeInclusive<f32>,
    default: f32,
    origin: Option<f32>,
    decimals: usize,
    tip: &str,
    gradient: Option<(Color32, Color32)>,
) -> (bool, bool) {
    let mut changed = false;
    let mut active = false;
    let (lo, hi) = (*range.start(), *range.end());
    let modified = (*v - default).abs() > 1e-6;
    ui.horizontal(|ui| {
        let l = ui.add(
            egui::Label::new(egui::RichText::new(label).color(if modified {
                theme::TEXT
            } else {
                theme::MUTED
            }))
            .sense(Sense::click()),
        );
        let l = l.on_hover_text(if tip.is_empty() {
            "Click to reset".to_string()
        } else {
            format!("{tip}\n\nClick the label to reset.")
        });
        if l.clicked() && modified {
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
            vis.inactive.fg_stroke.color = theme::MUTED;
            let dv = egui::DragValue::new(v)
                .speed((hi - lo) as f64 / 400.0)
                .max_decimals(decimals)
                .custom_formatter(fmt);
            let r = ui.add(dv.update_while_editing(false));
            if r.changed() {
                changed = true;
            }
            active |= r.dragged() || r.is_pointer_button_down_on();
        });
    });
    let w = ui.available_width() - 6.0;
    ui.add_space(-6.0);
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(w, 18.0), Sense::click_and_drag());
    let track = Rect::from_min_max(
        egui::pos2(rect.left() + 7.0, rect.center().y - 1.0),
        egui::pos2(rect.right() - 7.0, rect.center().y + 1.0),
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
    let hot = resp.hovered() || resp.dragged();
    let p = ui.painter();
    match gradient {
        Some((a, b)) => {
            let a = a.gamma_multiply(0.85);
            let b = b.gamma_multiply(0.85);
            gradient_rect(ui, track.expand2(egui::vec2(0.0, 0.5)), a, b)
        }
        None => {
            p.rect_filled(track, 1.0, theme::TRACK);
            let fill_base = origin.unwrap_or(default);
            let z = to_x(fill_base.clamp(lo, hi));
            let k = to_x(*v);
            let fill = Rect::from_min_max(
                egui::pos2(z.min(k), track.top()),
                egui::pos2(z.max(k), track.bottom()),
            );
            p.rect_filled(fill, 1.0, theme::ACCENT);
        }
    }
    let knob = egui::pos2(to_x(*v), track.center().y);
    if hot {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    p.circle(
        knob,
        if hot { 7.0 } else { 6.0 },
        theme::TEXT,
        egui::Stroke::new(2.0, theme::PANEL),
    );
    ui.add_space(4.0);
    (
        changed,
        active || resp.dragged() || resp.is_pointer_button_down_on(),
    )
}

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
    fl_slider_origin(ui, label, v, range, default, None, decimals, tip, gradient)
}

pub fn fl_slider_u8(
    ui: &mut Ui,
    label: &str,
    v: &mut u8,
    range: std::ops::RangeInclusive<u8>,
    default: u8,
    tip: &str,
) -> bool {
    fl_slider_u8_origin(ui, label, v, range, default, None, tip)
}

pub fn fl_slider_u8_origin(
    ui: &mut Ui,
    label: &str,
    v: &mut u8,
    range: std::ops::RangeInclusive<u8>,
    default: u8,
    origin: Option<f32>,
    tip: &str,
) -> bool {
    let mut f = *v as f32;
    let ch = fl_slider_origin(
        ui,
        label,
        &mut f,
        *range.start() as f32..=*range.end() as f32,
        default as f32,
        origin,
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
    let w = (ui.available_width() - 4.0) / items.len() as f32;
    egui::Frame::new()
        .fill(theme::BG)
        .corner_radius(7.0)
        .inner_margin(2.0)
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            ui.horizontal(|ui| {
                for (t, name) in items {
                    let sel = *v == *t;
                    let is_locked = locked.contains(t);
                    let (rect, resp) = ui.allocate_exact_size(
                        egui::vec2(w - 0.5, 24.0),
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
                        ui.painter().rect_filled(rect, 6.0, theme::HOVER);
                    }
                    let hot = resp.hovered() && !is_locked;
                    ui.painter().text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        *name,
                        if sel {
                            theme::medium(12.5)
                        } else {
                            egui::FontId::proportional(12.5)
                        },
                        if is_locked {
                            theme::FAINT
                        } else if sel || hot {
                            theme::TEXT
                        } else {
                            theme::MUTED
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

/// Collapsible section header with a chevron. Returns whether the section
/// is open; the state is remembered per title.
pub fn fold(ui: &mut Ui, title: &str) -> bool {
    fold_with(ui, title, true)
}

/// A [`fold`] that starts collapsed when `default_open` is false.
pub fn fold_with(ui: &mut Ui, title: &str, default_open: bool) -> bool {
    let id = ui.make_persistent_id(("fold", title));
    let mut open = ui.data_mut(|d| *d.get_persisted_mut_or(id, default_open));
    ui.add_space(10.0);
    let line_y = ui.cursor().top();
    ui.painter().hline(
        ui.max_rect().x_range(),
        line_y,
        egui::Stroke::new(1.0, theme::LINE),
    );
    ui.add_space(8.0);
    let (rect, resp) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), 20.0), Sense::click());
    let hot = resp.hovered();
    let p = ui.painter();
    p.text(
        rect.left_center(),
        egui::Align2::LEFT_CENTER,
        title,
        theme::medium(12.5),
        if hot { theme::TEXT } else { theme::MUTED },
    );
    let c = egui::pos2(rect.right() - 8.0, rect.center().y);
    let pts = if open {
        vec![
            c + egui::vec2(-3.5, -1.75),
            c + egui::vec2(0.0, 1.75),
            c + egui::vec2(3.5, -1.75),
        ]
    } else {
        vec![
            c + egui::vec2(-1.75, -3.5),
            c + egui::vec2(1.75, 0.0),
            c + egui::vec2(-1.75, 3.5),
        ]
    };
    p.add(egui::Shape::line(
        pts,
        egui::Stroke::new(1.4, if hot { theme::TEXT } else { theme::FAINT }),
    ));
    if resp.clicked() {
        open = !open;
        ui.data_mut(|d| d.insert_persisted(id, open));
    }
    if hot {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    if open {
        ui.add_space(4.0);
    }
    open
}

/// Rounded translucent label drawn over the canvas.
pub fn pill(p: &egui::Painter, center: egui::Pos2, text: &str) -> Rect {
    let g = p.layout_no_wrap(text.to_string(), theme::medium(12.0), theme::TEXT);
    let r = Rect::from_center_size(center, g.size() + egui::vec2(22.0, 10.0));
    p.rect_filled(r, r.height() / 2.0, Color32::from_black_alpha(170));
    p.galley(r.center() - g.size() / 2.0, g, theme::TEXT);
    r
}
