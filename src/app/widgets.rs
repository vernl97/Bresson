//! Shared controls and the dark graphite theme. Controls follow the plugin's
//! conventions: click a slider label (or double-click its track) to reset,
//! and typed values may exceed the slider range.

use eframe::egui::{self, Color32, FontId, Rect, Sense, Ui};

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
    /// Slider tracks: 3:1 against the panel, as WCAG asks of controls.
    pub const SLIDER_TRACK: Color32 = Color32::from_rgb(108, 108, 112);
    pub const ACCENT: Color32 = Color32::from_rgb(76, 141, 255);
    /// The accent behind white text: 4.7:1, where `ACCENT` only reaches 3.2:1.
    pub const ACCENT_FILL: Color32 = Color32::from_rgb(47, 111, 224);
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
        // Text fields stand out from both the panel and the BG banners, and
        // their hints read as hints rather than filled-in values.
        v.text_edit_bg_color = Some(SURFACE);
        v.weak_text_color = Some(FAINT);
        v.disabled_alpha = 0.4;
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
    // egui spells some keys out ("OpenBracket") off macOS; show the symbol.
    ctx.format_shortcut(&egui::KeyboardShortcut::new(mods, key))
        .replace("OpenBracket", "[")
        .replace("CloseBracket", "]")
        .replace("Escape", "Esc")
}

/// Accent-filled call-to-action button.
pub fn primary<'a>(text: &str) -> egui::Button<'a> {
    egui::Button::new(
        egui::RichText::new(text)
            .font(theme::medium(13.0))
            .color(Color32::WHITE),
    )
    .fill(theme::ACCENT_FILL)
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
/// `frozen` greys the plot out: it shows the last settled state.
pub fn histogram(
    ui: &mut Ui,
    h: &[[u32; 256]; 3],
    size: egui::Vec2,
    frozen: bool,
    clip_on: bool,
    values: Option<[String; 5]>,
) -> HistEvent {
    let mut ev = HistEvent::default();
    let id = ui.make_persistent_id("histogram");
    let live = values.is_some() && !frozen;
    let (rect, resp) = ui.allocate_exact_size(
        size,
        if live {
            Sense::click_and_drag()
        } else {
            Sense::hover()
        },
    );
    let p = ui.painter_at(rect);
    p.rect_filled(rect, 6.0, theme::BG);
    // Clipping corners: shadows left, highlights right. They are buttons.
    let corners = [
        Rect::from_min_size(rect.left_top(), egui::vec2(16.0, 16.0)),
        Rect::from_min_size(
            rect.right_top() - egui::vec2(16.0, 0.0),
            egui::vec2(16.0, 16.0),
        ),
    ];
    let over_corner = ui
        .ctx()
        .pointer_hover_pos()
        .is_some_and(|q| corners.iter().any(|c| c.contains(q)));
    // The region under the pointer (or the one being dragged) gets the
    // Lightroom treatment: lit background, name and value, drag to edit.
    let region_at = |x: f32| {
        let t = ((x - rect.left()) / rect.width()).clamp(0.0, 0.9999);
        HistRegion::ALL[(t * 5.0) as usize]
    };
    let drag_id = id.with("region");
    let mut region = None;
    if live {
        if resp.drag_started()
            && let Some(q) = resp.interact_pointer_pos()
        {
            ui.data_mut(|d| d.insert_temp(drag_id, region_at(q.x)));
        }
        if resp.dragged() {
            region = ui.data(|d| d.get_temp::<HistRegion>(drag_id));
        } else if !over_corner {
            region = resp.hover_pos().map(|q| region_at(q.x));
        }
    }
    if let Some(r) = region {
        let w = rect.width() / 5.0;
        let band = Rect::from_min_size(
            rect.left_top() + egui::vec2(w * r as usize as f32, 0.0),
            egui::vec2(w, rect.height()),
        );
        p.rect_filled(band, 0.0, Color32::from_white_alpha(14));
        ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
    }
    let plot = rect.shrink2(egui::vec2(6.0, 6.0));
    let max = h
        .iter()
        .flat_map(|c| c[2..254].iter())
        .copied()
        .max()
        .unwrap_or(1)
        .max(1) as f32;
    let cols = if frozen {
        [(95u8, 95u8, 95u8); 3]
    } else {
        [(235u8, 72u8, 72u8), (72, 200, 96), (80, 130, 255)]
    };
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
    // when more than 0.1 % of the pixels sit at 0 or 255; an outline when
    // nothing clips. Click to show the clipping overlay (J).
    let total = h[0].iter().sum::<u32>().max(1) as f32;
    for (k, (end, left)) in [(0usize, true), (255, false)].into_iter().enumerate() {
        let mut col = [0u8; 3];
        for c in 0..3 {
            if h[c][end] as f32 / total > 0.001 {
                col[c] = 255;
            }
        }
        let clipped = col != [0; 3];
        let fill = if col == [255; 3] {
            Color32::WHITE
        } else {
            Color32::from_rgb(col[0].max(60), col[1].max(60), col[2].max(60))
        };
        let (x0, dx) = if left {
            (rect.left() + 3.0, 8.0)
        } else {
            (rect.right() - 3.0, -8.0)
        };
        let y0 = rect.top() + 3.0;
        let tri = vec![
            egui::pos2(x0, y0),
            egui::pos2(x0 + dx, y0),
            egui::pos2(x0, y0 + 8.0),
        ];
        let hot = ui
            .interact(corners[k], id.with(("clip", k)), Sense::click())
            .on_hover_text("Show clipping (J)")
            .on_hover_cursor(egui::CursorIcon::PointingHand);
        if hot.clicked() {
            ev.clip_toggle = true;
        }
        let ring = if clip_on {
            theme::ACCENT
        } else if hot.hovered() {
            theme::TEXT
        } else {
            theme::FAINT
        };
        p.add(egui::Shape::convex_polygon(
            tri,
            if clipped { fill } else { Color32::TRANSPARENT },
            egui::Stroke::new(if clip_on { 1.5 } else { 1.0 }, ring),
        ));
    }
    if let (Some(r), Some(values)) = (region, values.as_ref()) {
        let g = p.layout_no_wrap(
            format!("{}  {}", r.name(), values[r as usize]),
            theme::medium(11.0),
            theme::TEXT,
        );
        p.galley(
            egui::pos2(rect.left() + 8.0, rect.bottom() - g.size().y - 5.0),
            g,
            theme::TEXT,
        );
        if resp.dragged() {
            ev.drag = Some((r, resp.drag_delta().x));
        }
        if resp.double_clicked() {
            ev.reset = Some(r);
        }
    }
    ev.active = resp.dragged();
    ev
}

/// The five tonal regions of the histogram, as in Lightroom.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum HistRegion {
    Blacks,
    Darks,
    Exposure,
    Lights,
    Whites,
}

impl HistRegion {
    pub const ALL: [HistRegion; 5] = [
        HistRegion::Blacks,
        HistRegion::Darks,
        HistRegion::Exposure,
        HistRegion::Lights,
        HistRegion::Whites,
    ];
    pub fn name(self) -> &'static str {
        match self {
            HistRegion::Blacks => "Blacks",
            HistRegion::Darks => "Darks",
            HistRegion::Exposure => "Exposure",
            HistRegion::Lights => "Lights",
            HistRegion::Whites => "Whites",
        }
    }
}

/// What the user did in the histogram this frame.
#[derive(Default)]
pub struct HistEvent {
    /// A clipping triangle was clicked.
    pub clip_toggle: bool,
    /// Horizontal drag (points) in a region.
    pub drag: Option<(HistRegion, f32)>,
    /// Double-click in a region.
    pub reset: Option<HistRegion>,
    /// A drag is in progress.
    pub active: bool,
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
    /// Four squares: the roll's grid view.
    Grid,
    Crop,
    RotateLeft,
    RotateRight,
    Lock,
    Unlock,
    Eyedropper,
    Folder,
    Warn,
    More,
    Close,
    /// A rejected frame: a struck circle, so it doesn't read as "close".
    Reject,
    /// View toggles of the editor's bottom bar.
    Before,
    Split,
    Clipping,
    Flag,
    /// Back to the default value: an arrow turning anticlockwise.
    Reset,
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
    match icon {
        Icon::Grid => {
            for (x, y) in [(-3.4, -3.4), (3.4, -3.4), (-3.4, 3.4), (3.4, 3.4)] {
                let r = Rect::from_min_max(q(x - 2.4, y - 2.4), q(x + 2.4, y + 2.4));
                p.rect_stroke(r, 1.0, st, egui::StrokeKind::Middle);
            }
        }
        Icon::RotateLeft | Icon::RotateRight => {
            let flip = if icon == Icon::RotateLeft { -1.0 } else { 1.0 };
            // A frame with an arrow turning over it, so it can't read as undo:
            // clockwise on screen, ending in a head at the right.
            let r = 7.0 * s;
            let (a0, a1) = (-2.9_f32, -0.1_f32);
            let mx = |pt: egui::Pos2| pos2(c.x + (pt.x - c.x) * flip, pt.y);
            let frame = Rect::from_two_pos(mx(q(-7.5, 0.5)), mx(q(2.0, 8.5)));
            p.rect_stroke(frame, 1.0, st, egui::StrokeKind::Middle);
            let o = q(-1.0, 0.0);
            let pts: Vec<_> = arc(o, r, a0, a1, 20).into_iter().map(mx).collect();
            p.add(line(pts));
            let end = mx(pos2(o.x + r * a1.cos(), o.y + r * a1.sin()));
            let tangent = mx(pos2(c.x - a1.sin(), c.y + a1.cos())) - mx(c);
            let t = tangent.normalized();
            let n = vec2(-t.y, t.x);
            let tip = end + t * 3.2 * s;
            let base = end - t * 1.2 * s;
            p.add(Shape::convex_polygon(
                vec![tip, base + n * 3.4 * s, base - n * 3.4 * s],
                color,
                Stroke::NONE,
            ));
        }
        Icon::Before => {
            // A picture with an arrow pointing back: the earlier state.
            let r = Rect::from_center_size(c, vec2(16.0 * s, 13.0 * s));
            p.rect_stroke(r, 2.0, st, egui::StrokeKind::Middle);
            let (tip, end) = (c - vec2(4.0 * s, 0.0), c + vec2(4.5 * s, 0.0));
            p.line_segment([tip, end], st);
            let h = 3.0 * s;
            p.add(Shape::line(
                vec![tip + vec2(h, -h), tip, tip + vec2(h, h)],
                st,
            ));
        }
        Icon::Split => {
            let r = Rect::from_center_size(c, vec2(16.0 * s, 13.0 * s));
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
        Icon::Clipping => {
            // A histogram piled up against both ends: tones clipped.
            let base = c.y + 6.5 * s;
            for (k, h) in [13.0, 4.0, 6.0, 4.0, 13.0].into_iter().enumerate() {
                let x = c.x + (k as f32 - 2.0) * 3.5 * s;
                p.rect_filled(
                    Rect::from_min_max(pos2(x - 1.1 * s, base - h * s), pos2(x + 1.1 * s, base)),
                    0.5,
                    color,
                );
            }
        }
        Icon::Reset => {
            let r = 5.0 * s;
            let (a0, a1) = (-2.4f32, 2.2f32);
            p.add(Shape::line(arc(c, r, a0, a1, 18), st));
            let tip = c + r * vec2(a0.cos(), a0.sin());
            let h = 3.0 * s;
            p.add(Shape::line(
                vec![tip + vec2(0.0, -h), tip, tip + vec2(h, 0.0)],
                st,
            ));
        }
        Icon::Flag => {
            p.add(line(vec![q(-5.0, -8.0), q(-5.0, 8.0)]));
            p.add(Shape::closed_line(
                vec![q(-5.0, -8.0), q(7.0, -4.0), q(-5.0, 0.0)],
                st,
            ));
        }
        Icon::Lock | Icon::Unlock => {
            let locked = icon == Icon::Lock;
            let body = Rect::from_min_max(q(-6.0, -1.0), q(6.0, 8.0));
            p.rect_stroke(body, 1.5, st, egui::StrokeKind::Middle);
            let lift = if locked { 0.0 } else { -3.0 };
            let mut pts = arc(
                q(0.0, -1.0 + lift),
                4.0 * s,
                std::f32::consts::PI,
                std::f32::consts::TAU,
                10,
            );
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
        Icon::Reject => {
            p.circle_stroke(c, 7.0 * s, st);
            p.line_segment([q(-4.9, -4.9), q(4.9, 4.9)], st);
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
    sized_icon_button(
        ui,
        icon,
        tip,
        selected,
        enabled,
        tint,
        egui::vec2(32.0, 28.0),
        8.0,
    )
}

/// Larger [`icon_button_tinted`] for the editor's top bar.
pub fn bar_button(
    ui: &mut Ui,
    icon: Icon,
    tip: &str,
    selected: bool,
    enabled: bool,
    tint: Option<Color32>,
) -> egui::Response {
    sized_icon_button(
        ui,
        icon,
        tip,
        selected,
        enabled,
        tint,
        egui::vec2(40.0, 34.0),
        7.0,
    )
}

#[allow(clippy::too_many_arguments)]
fn sized_icon_button(
    ui: &mut Ui,
    icon: Icon,
    tip: &str,
    selected: bool,
    enabled: bool,
    tint: Option<Color32>,
    size: egui::Vec2,
    pad: f32,
) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(
        size,
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
    paint_icon(p, rect.shrink2(egui::vec2(pad, pad - 2.0)), icon, col);
    if hot {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resp.on_hover_text(tip)
}

/// The app's dropdown: a filled box with the sections' thin chevron.
pub fn combo(id: impl std::hash::Hash + std::fmt::Debug) -> egui::ComboBox {
    egui::ComboBox::from_id_salt(id).icon(|ui, rect, visuals, _open| {
        chevron(ui.painter(), rect.center(), true, visuals.fg_stroke.color);
    })
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
    combo(("fl_combo", label, std::any::type_name::<T>(), ui.id()))
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

/// A film stock field: free text with a dropdown of the bundled films
/// (the crosstalk folder). Typing filters the list; any text is allowed.
pub fn stock_combo(ui: &mut Ui, value: &mut String, hint: &str, width: f32) -> egui::Response {
    let mut changed = false;
    let pad = ui.spacing().button_padding;
    let edit = ui.add(
        egui::TextEdit::singleline(value)
            .hint_text(hint)
            // Keep typed text clear of the chevron drawn inside the field.
            .margin(egui::Margin {
                left: pad.x as i8,
                right: 22,
                top: pad.y as i8,
                bottom: pad.y as i8,
            })
            .desired_width((width - 8.0).max(40.0)),
    );
    changed |= edit.changed();
    let arrow_rect = egui::Rect::from_min_max(
        edit.rect.right_top() - egui::vec2(22.0, 0.0),
        edit.rect.right_bottom(),
    );
    let arrow = ui
        .interact(arrow_rect, ui.id().with("stock_arrow"), Sense::click())
        .on_hover_text("Choose a film stock");
    let col = if arrow.hovered() {
        theme::TEXT
    } else {
        theme::MUTED
    };
    chevron(ui.painter(), arrow.rect.center(), true, col);
    let names = crate::engine::presets::film_stock_names();
    let q = value.trim().to_lowercase();
    let typed = !q.is_empty() && !names.iter().any(|n| n.to_lowercase() == q);
    egui::Popup::menu(&arrow).show(|ui| {
        ui.set_min_width(220.0);
        egui::ScrollArea::vertical()
            .max_height(320.0)
            .show(ui, |ui| {
                for n in names
                    .iter()
                    .filter(|n| !typed || n.to_lowercase().contains(&q))
                {
                    if ui.selectable_label(*value == *n, n).clicked() {
                        *value = n.clone();
                        changed = true;
                    }
                }
            });
    });
    let mut r = edit;
    if changed {
        r.mark_changed();
    }
    r
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

/// Width of the label column of slider rows.
pub const LABEL_W: f32 = 90.0;
/// Gap between the label and the track, wide enough that a knob at the
/// track's left end (radius 7 plus its ring) stays clear of the label.
const TRACK_GAP: f32 = 11.0;

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
    // One compact row, as in Lightroom: label, track, value.
    const VALUE_W: f32 = 42.0;
    let (row, resp) = ui.allocate_exact_size(
        egui::vec2(ui.available_width() - 14.0, 24.0),
        Sense::click_and_drag(),
    );
    let label_rect = Rect::from_min_size(row.min, egui::vec2(LABEL_W, row.height()));
    let value_rect = Rect::from_min_max(egui::pos2(row.right() - VALUE_W, row.top()), row.max);
    let label_left = ui
        .scope_builder(
            egui::UiBuilder::new()
                .max_rect(label_rect)
                .layout(egui::Layout::right_to_left(egui::Align::Center)),
            |ui| {
                let l = ui.add(
                    egui::Label::new(egui::RichText::new(label).color(if modified {
                        theme::TEXT
                    } else {
                        theme::MUTED
                    }))
                    .truncate()
                    .sense(Sense::click()),
                );
                let left = l.rect.left();
                let l = l.on_hover_text(if tip.is_empty() {
                    "Click to reset".to_string()
                } else {
                    format!("{tip}\n\nClick the label to reset.")
                });
                if l.clicked() && modified {
                    *v = default;
                    changed = true;
                }
                left
            },
        )
        .inner;
    // A dot left of the label marks the edit; hovering the row turns it into
    // a reset button, like the section headers.
    if modified {
        let c = egui::pos2(label_left - 9.0, row.center().y);
        let hit = Rect::from_center_size(c, egui::vec2(16.0, row.height()));
        let p = ui.painter().with_clip_rect(bleed(row));
        if ui.rect_contains_pointer(row.union(hit)) {
            let r = ui
                .interact(hit, ui.id().with((label, "reset")), Sense::click())
                .on_hover_text("Reset")
                .on_hover_cursor(egui::CursorIcon::PointingHand);
            let color = if r.hovered() {
                theme::TEXT
            } else {
                theme::MUTED
            };
            paint_icon(
                &p,
                Rect::from_center_size(c, egui::vec2(12.0, 12.0)),
                Icon::Reset,
                color,
            );
            if r.clicked() {
                *v = default;
                changed = true;
            }
        } else {
            p.circle_filled(c, 2.5, theme::ACCENT);
        }
    }
    ui.scope_builder(
        egui::UiBuilder::new()
            .max_rect(value_rect)
            .layout(egui::Layout::right_to_left(egui::Align::Center)),
        |ui| {
            // Bipolar values show their sign, whatever the slider resets to.
            let signed = default == 0.0 || gradient.is_some();
            let fmt = move |x: f64, _r: std::ops::RangeInclusive<usize>| {
                let s = format!("{:.*}", decimals, x);
                if x > 0.0 && signed {
                    format!("+{s}")
                } else {
                    s
                }
            };
            // Frameless value, editable by click (type) or horizontal drag.
            // The box hugs its text and is as tall as the row, so values
            // right-align and sit on the label's baseline.
            ui.spacing_mut().interact_size = egui::vec2(0.0, 24.0);
            ui.spacing_mut().button_padding = egui::vec2(0.0, 0.0);
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
        },
    );
    let track = Rect::from_min_max(
        egui::pos2(label_rect.right() + TRACK_GAP, row.center().y - 1.0),
        egui::pos2(value_rect.left() - 5.0, row.center().y + 1.0),
    );
    let to_x = |val: f32| track.left() + track.width() * ((val - lo) / (hi - lo)).clamp(0.0, 1.0);
    if resp.double_clicked() {
        *v = default;
        changed = true;
    } else if resp.dragged() || resp.clicked() {
        if let Some(pos) = resp.interact_pointer_pos() {
            // Shift while dragging: relative moves at a tenth of the speed.
            let fine = resp.dragged() && ui.input(|i| i.modifiers.shift);
            let mut nv = if fine {
                (*v + resp.drag_delta().x / track.width() * (hi - lo) * 0.1).clamp(lo, hi)
            } else {
                let t = ((pos.x - track.left()) / track.width()).clamp(0.0, 1.0);
                lo + t * (hi - lo)
            };
            let places = decimals + usize::from(fine);
            let m = 10f32.powi(places as i32);
            nv = (nv * m).round() / m;
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
            p.rect_filled(track, 1.0, theme::SLIDER_TRACK);
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
    // A tick at the default value, so the neutral position is visible.
    if default > lo && default < hi && (*v - default).abs() > (hi - lo) * 0.02 {
        let x = to_x(default);
        p.vline(
            x,
            track.center().y - 4.0..=track.center().y + 4.0,
            egui::Stroke::new(1.0, theme::FAINT),
        );
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

/// Collapsible section header with a chevron, starting collapsed when
/// `default_open` is false. Returns whether the section is open; the state
/// is remembered per title.
pub fn fold_with(ui: &mut Ui, title: &str, default_open: bool) -> bool {
    fold_section(ui, title, default_open, false).0
}

/// A section header's band and title for a panel that doesn't collapse.
pub fn panel_header(ui: &mut Ui, title: &str) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 32.0), Sense::hover());
    let band = bleed(rect);
    let p = ui
        .painter()
        .with_clip_rect(band.expand2(egui::vec2(0.0, 2.0)));
    p.rect_filled(band, 0.0, theme::BG);
    p.hline(
        band.x_range(),
        band.bottom() - 0.5,
        egui::Stroke::new(1.0, Color32::from_rgb(46, 46, 50)),
    );
    p.text(
        rect.right_center() - egui::vec2(32.0, 0.0),
        egui::Align2::RIGHT_CENTER,
        title,
        theme::medium(14.5),
        theme::TEXT,
    );
}

/// The next section header follows flush, with no open-section spacing.
pub fn reset_section_gap(ui: &mut Ui) {
    ui.data_mut(|d| d.insert_temp(egui::Id::new("section_prev_open"), false));
}

/// Fades the bottom of `rect` into the panel, to show that it scrolls on.
pub fn scroll_fade(ui: &Ui, rect: Rect) {
    const H: f32 = 28.0;
    let r = Rect::from_min_max(egui::pos2(rect.left(), rect.bottom() - H), rect.max);
    let mut mesh = egui::Mesh::default();
    for (pos, col) in [
        (r.left_top(), Color32::TRANSPARENT),
        (r.right_top(), Color32::TRANSPARENT),
        (r.right_bottom(), theme::PANEL),
        (r.left_bottom(), theme::PANEL),
    ] {
        mesh.colored_vertex(pos, col);
    }
    mesh.add_triangle(0, 1, 2);
    mesh.add_triangle(0, 2, 3);
    ui.painter().add(egui::Shape::mesh(mesh));
}

/// A sidebar section header. When `modified`, a dot after the title marks
/// the edit and hovering the header offers Reset; returns (open, reset).
pub fn fold_section(ui: &mut Ui, title: &str, default_open: bool, modified: bool) -> (bool, bool) {
    let id = ui.make_persistent_id(("fold", title));
    let mut open = ui.data_mut(|d| *d.get_persisted_mut_or(id, default_open));
    // Flat bands that run edge to edge, like Lightroom's panel headers. A
    // little room after an open body; closed headers stack flush.
    let prev_open: bool = ui
        .data(|d| d.get_temp(egui::Id::new("section_prev_open")))
        .unwrap_or(false);
    ui.data_mut(|d| d.insert_temp(egui::Id::new("section_prev_open"), open));
    ui.add_space(if prev_open { 8.0 } else { 0.0 });
    let (rect, resp) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), 32.0), Sense::click());
    // Cancel the row spacing egui adds after a widget, so closed headers touch.
    ui.add_space(-ui.spacing().item_spacing.y);
    let hot = resp.hovered();
    let p = ui.painter();
    let band = bleed(rect);
    p.with_clip_rect(band.expand2(egui::vec2(0.0, 2.0)))
        .rect_filled(
            band,
            0.0,
            if hot {
                Color32::from_rgb(33, 33, 36)
            } else {
                theme::BG
            },
        );
    // A fine rule along the bottom edge keeps stacked headers apart.
    p.with_clip_rect(band.expand2(egui::vec2(0.0, 2.0))).hline(
        band.x_range(),
        band.bottom() - 0.5,
        egui::Stroke::new(1.0, Color32::from_rgb(46, 46, 50)),
    );
    let title_rect = p.text(
        rect.right_center() - egui::vec2(32.0, 0.0),
        egui::Align2::RIGHT_CENTER,
        title,
        theme::medium(14.5),
        theme::TEXT,
    );
    if modified {
        p.circle_filled(
            egui::pos2(title_rect.left() - 9.0, rect.center().y),
            2.5,
            theme::ACCENT,
        );
    }
    // Reset sits at the left end while the header is hovered.
    let mut reset_rect = None;
    if modified && hot {
        let g = p.layout_no_wrap("Reset".into(), FontId::proportional(11.0), theme::MUTED);
        let r = Rect::from_min_size(
            egui::pos2(rect.left() + 12.0, rect.center().y - g.size().y / 2.0),
            g.size(),
        );
        let over = ui
            .ctx()
            .pointer_hover_pos()
            .is_some_and(|q| r.expand(4.0).contains(q));
        p.galley_with_override_text_color(
            r.min,
            g,
            if over { theme::ACCENT } else { theme::MUTED },
        );
        reset_rect = Some(r.expand(4.0));
    }
    chevron(
        p,
        egui::pos2(rect.right() - 14.0, rect.center().y),
        open,
        if hot { theme::TEXT } else { theme::MUTED },
    );
    let mut reset = false;
    if resp.clicked() {
        let on_reset = reset_rect
            .zip(resp.interact_pointer_pos())
            .is_some_and(|(r, q)| r.contains(q));
        if on_reset {
            reset = true;
        } else {
            open = !open;
            ui.data_mut(|d| d.insert_persisted(id, open));
        }
    }
    if hot {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        if modified {
            resp.on_hover_text(format!("{title} has been edited"));
        }
    }
    if open {
        ui.add_space(4.0);
    }
    (open, reset)
}

/// Bare slim slider (track and knob, no label or value) for toolbars,
/// drawn like [`fl_slider`].
pub fn mini_slider(
    ui: &mut Ui,
    v: &mut f32,
    range: std::ops::RangeInclusive<f32>,
    width: f32,
) -> egui::Response {
    let (lo, hi) = (*range.start(), *range.end());
    let (rect, mut resp) = ui.allocate_exact_size(egui::vec2(width, 18.0), Sense::click_and_drag());
    let track = Rect::from_min_max(
        egui::pos2(rect.left() + 6.0, rect.center().y - 1.0),
        egui::pos2(rect.right() - 6.0, rect.center().y + 1.0),
    );
    if (resp.dragged() || resp.clicked())
        && let Some(pos) = resp.interact_pointer_pos()
    {
        let t = ((pos.x - track.left()) / track.width()).clamp(0.0, 1.0);
        let nv = lo + t * (hi - lo);
        if (nv - *v).abs() > 1e-3 {
            *v = nv;
            resp.mark_changed();
        }
    }
    let hot = resp.hovered() || resp.dragged();
    let p = ui.painter();
    let x = track.left() + track.width() * ((*v - lo) / (hi - lo)).clamp(0.0, 1.0);
    p.rect_filled(track, 1.0, theme::SLIDER_TRACK);
    p.rect_filled(
        Rect::from_min_max(track.left_top(), egui::pos2(x, track.bottom())),
        1.0,
        theme::MUTED,
    );
    if hot {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    p.circle(
        egui::pos2(x, track.center().y),
        if hot { 6.0 } else { 5.0 },
        theme::TEXT,
        egui::Stroke::new(2.0, theme::BG),
    );
    resp
}

/// Small faint path that keeps its end when it doesn't fit ("…/roll/Positives"),
/// with the home folder shown as `~` and the full path on hover.
pub fn path_label(ui: &mut Ui, path: &str) -> egui::Response {
    let home = dirs::home_dir().map(|h| h.display().to_string());
    let short = match home.as_deref().and_then(|h| path.strip_prefix(h)) {
        Some(rest) if rest.starts_with('/') || rest.is_empty() => format!("~{rest}"),
        _ => path.to_string(),
    };
    let font = FontId::proportional(11.0);
    let max_w = ui.available_width();
    let fits = |t: &str| {
        ui.fonts_mut(|f| {
            f.layout_no_wrap(t.to_string(), font.clone(), theme::FAINT)
                .size()
                .x
        }) <= max_w
    };
    let mut text = short.clone();
    let mut cut = 0;
    while !fits(&text) && cut < short.len() {
        cut = short[cut..]
            .char_indices()
            .nth(1)
            .map_or(short.len(), |(k, _)| cut + k);
        text = format!("…{}", &short[cut..]);
    }
    ui.add(egui::Label::new(
        egui::RichText::new(text).font(font).color(theme::FAINT),
    ))
    .on_hover_text(path)
}

/// Checkbox in the app's style: an outlined box that fills with the accent
/// colour when checked (egui's own box is nearly invisible on the panels).
pub fn checkbox(ui: &mut Ui, checked: &mut bool, text: &str) -> egui::Response {
    let g = ui
        .painter()
        .layout_no_wrap(text.to_string(), FontId::proportional(13.0), theme::TEXT);
    let size = egui::vec2(14.0 + 8.0 + g.size().x, ui.spacing().interact_size.y);
    let (rect, mut resp) = ui.allocate_exact_size(size, Sense::click());
    if resp.clicked() {
        *checked = !*checked;
        resp.mark_changed();
    }
    let on = *checked;
    let enabled = ui.is_enabled();
    resp.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::Checkbox, enabled, on, text));
    if ui.is_rect_visible(rect) {
        let p = ui.painter();
        let b = Rect::from_center_size(
            egui::pos2(rect.left() + 7.0, rect.center().y),
            egui::vec2(14.0, 14.0),
        );
        let hot = resp.hovered();
        if on {
            p.rect_filled(b, 3.5, if hot { ACCENT_HOVER } else { theme::ACCENT });
            p.add(egui::Shape::line(
                vec![
                    b.center() + egui::vec2(-3.5, 0.2),
                    b.center() + egui::vec2(-1.0, 2.8),
                    b.center() + egui::vec2(3.6, -2.6),
                ],
                egui::Stroke::new(1.8, Color32::WHITE),
            ));
        } else {
            p.rect(
                b,
                3.5,
                theme::SURFACE,
                egui::Stroke::new(1.2, if hot { theme::MUTED } else { theme::FAINT }),
                egui::StrokeKind::Inside,
            );
        }
        let col = if enabled { theme::TEXT } else { theme::FAINT };
        p.galley_with_override_text_color(
            egui::pos2(b.right() + 8.0, rect.center().y - g.size().y / 2.0),
            g,
            col,
        );
    }
    resp
}

const ACCENT_HOVER: Color32 = Color32::from_rgb(102, 158, 255);

/// Small line chevron centred on `c`: pointing down when `down`, else right.
pub fn chevron(p: &egui::Painter, c: egui::Pos2, down: bool, color: Color32) {
    let pts = if down {
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
    p.add(egui::Shape::line(pts, egui::Stroke::new(1.4, color)));
}

/// Rounded translucent label drawn over the canvas.
pub fn pill(p: &egui::Painter, center: egui::Pos2, text: &str) -> Rect {
    let g = p.layout_no_wrap(text.to_string(), theme::medium(12.0), theme::TEXT);
    let r = Rect::from_center_size(center, g.size() + egui::vec2(22.0, 10.0));
    p.rect_filled(r, r.height() / 2.0, Color32::from_black_alpha(170));
    p.galley(r.center() - g.size() / 2.0, g, theme::TEXT);
    r
}

/// How far the sidebar's content is inset from the panel edge (left, right
/// including the scrollbar gutter): bands and rules paint out to the edge.
const SIDEBAR_INSET: (f32, f32) = (16.0, 18.0);

/// `rect` stretched across the whole sidebar width.
pub fn bleed(rect: Rect) -> Rect {
    Rect::from_min_max(
        egui::pos2(rect.left() - SIDEBAR_INSET.0, rect.top()),
        egui::pos2(rect.right() + SIDEBAR_INSET.1, rect.bottom()),
    )
}

/// A dark full-width rule between groups of slider rows.
pub fn group_divider(ui: &mut Ui) {
    ui.add_space(5.0);
    let (rect, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 2.0), Sense::hover());
    let band = bleed(rect);
    ui.painter()
        .with_clip_rect(band.expand(1.0))
        .rect_filled(band, 0.0, theme::BG);
    ui.add_space(5.0);
}

/// A dropdown in a slider's columns: the label right-aligned in the label
/// column, the box from the track to `trailing` short of the row's end.
pub fn combo_row<T: Copy + PartialEq>(
    ui: &mut Ui,
    label: &str,
    v: &mut T,
    all: &[T],
    name: impl Fn(T) -> &'static str,
    trailing: f32,
) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.allocate_ui_with_layout(
            egui::vec2(LABEL_W, 26.0),
            egui::Layout::right_to_left(egui::Align::Center),
            |ui| ui.label(egui::RichText::new(label).color(theme::MUTED)),
        );
        ui.add_space(TRACK_GAP);
        combo(("combo_row", label, std::any::type_name::<T>()))
            .width((ui.available_width() - 14.0 - trailing).max(60.0))
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
    });
    changed
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_kittest::{Harness, kittest::Queryable};

    /// The dot beside an edited slider's label becomes a reset button on hover.
    #[test]
    fn edited_slider_resets_from_the_dot_beside_its_label() {
        let mut h = Harness::new_ui_state(
            |ui, v: &mut f32| {
                ui.add_space(0.0);
                ui.horizontal(|ui| {
                    ui.add_space(40.0);
                    ui.vertical(|ui| {
                        fl_slider(ui, "Exposure", v, -5.0..=5.0, 0.0, 1, "", None);
                    });
                });
            },
            2.0f32,
        );
        h.run();
        let label = h.get_by_label("Exposure").rect();
        let dot = egui::pos2(
            label.min.x as f32 - 9.0,
            (label.min.y + label.max.y) as f32 / 2.0,
        );
        h.event(egui::Event::PointerMoved(dot));
        h.run();
        for pressed in [true, false] {
            h.event(egui::Event::PointerButton {
                pos: dot,
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: egui::Modifiers::NONE,
            });
            h.run();
        }
        assert_eq!(*h.state(), 0.0);
    }
}
