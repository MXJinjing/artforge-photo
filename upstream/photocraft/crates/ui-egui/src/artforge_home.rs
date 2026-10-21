//! ArtForge Photo's native start screen.
//!
//! The HarmonyOS build uses this screen for every theme, deriving its palette from each theme's
//! tokens. It keeps the upstream start screen and every command it exposes intact, but composes
//! them as a small native art studio: an editorial hero, a responsive recent-work rail, and a
//! quiet static composition drawn entirely with egui shapes and the existing brand texture.

use egui::{Align, Align2, Color32, FontId, Pos2, Rect, Response, Sense, Stroke, StrokeKind, Ui, Vec2, pos2, vec2};

use crate::theme::{ThemeKind, Tokens};
use crate::{PhotocraftApp, brand, icons};

/// The HarmonyOS closed-source build always prefers this screen over the upstream one.
pub(crate) fn use_custom_home(_kind: ThemeKind) -> bool {
    true
}

const HOME_RECENT_LIMIT: usize = 6;
const HOME_RECENT_ROW: f32 = 34.0;
/// Two card sizes instead of a share of the screen: the large split card keeps a fixed width,
/// and the compact card is square (1:1) at the same height. Only a window physically smaller
/// than the card clamps it, so the card never overflows.
const HOME_PANEL_WIDTH: f32 = 900.0;
const HOME_PANEL_MAX_HEIGHT: f32 = 560.0;
const HOME_PANEL_COMPACT_ASPECT: f32 = 1.0;
const HOME_PANEL_COMPACT_WIDTH: f32 = HOME_PANEL_COMPACT_ASPECT * HOME_PANEL_MAX_HEIGHT;
/// Space kept between the card and the canvas edge before the fixed size is clamped.
const HOME_PANEL_MARGIN: f32 = 24.0;
/// Both side margins together keep at least this share of the canvas width, so a card never
/// crowds the workspace: a card that would break it steps down to the compact one.
const HOME_PANEL_MIN_SIDE_SHARE: f32 = 0.20;

#[derive(Clone, Copy, Debug)]
struct HomeLayout {
    panel: Rect,
    content: Rect,
    art: Rect,
    split: bool,
}

#[derive(Clone, Copy)]
struct Palette {
    dark: bool,
    ink: Color32,
    muted: Color32,
    faint: Color32,
    panel_top: Color32,
    border: Color32,
    blue: Color32,
    violet: Color32,
    primary_text: Color32,
    glass: Color32,
    glass_hover: Color32,
    shadow: Color32,
}

impl Palette {
    fn new(t: &Tokens) -> Self {
        Self {
            dark: t.dark(),
            ink: t.text,
            muted: t.text_dim,
            faint: t.text_faint,
            panel_top: with_alpha(t.card, 238),
            border: t.card_border,
            blue: t.primary_bg,
            violet: mix(t.accent, t.text, 0.34),
            primary_text: t.primary_text,
            glass: with_alpha(t.field, if t.dark() { 218 } else { 236 }),
            glass_hover: with_alpha(t.hover, if t.dark() { 240 } else { 248 }),
            shadow: t.shadow,
        }
    }
}

/// With no document open, double-clicking the blank workspace around the Home panel shows
/// File › Open (as in Photoshop); the panel's own widgets keep their clicks.
pub(crate) fn open_on_blank_double_click(app: &mut PhotocraftApp, ui: &Ui, area: Rect) {
    if !app.session.documents().is_empty() {
        return;
    }
    let panel = layout(area).panel;
    let mut opened = false;
    for (i, blank) in blank_around(area, panel).into_iter().enumerate() {
        if blank.is_positive() && !opened {
            opened = ui.interact(blank, ui.id().with(("artforge-home-blank", i)), Sense::click()).double_clicked();
        }
    }
    if opened {
        let _ = app.open_dialog_file();
    }
}

fn blank_around(area: Rect, panel: Rect) -> [Rect; 4] {
    [
        Rect::from_min_max(area.min, pos2(area.max.x, panel.min.y)),
        Rect::from_min_max(pos2(area.min.x, panel.max.y), area.max),
        Rect::from_min_max(pos2(area.min.x, panel.min.y), pos2(panel.min.x, panel.max.y)),
        Rect::from_min_max(pos2(panel.max.x, panel.min.y), pos2(area.max.x, panel.max.y)),
    ]
}

pub(crate) fn start_screen(app: &mut PhotocraftApp, ui: &mut Ui) {
    let area = ui.available_rect_before_wrap();
    if !area.is_finite() || !area.is_positive() {
        return;
    }
    let t = Tokens::get(ui.ctx());
    let p = Palette::new(&t);
    let layout = layout(area);

    paint_panel(ui, layout.panel, &p);
    if layout.split {
        paint_artwork(ui, layout.art, &p);
    } else {
        paint_compact_artwork(ui, layout.content, &p);
    }
    paint_content(app, ui, layout, &p);
}

fn layout(area: Rect) -> HomeLayout {
    // Keep at least 20% of the width as side space (and the usual edge margin): a card that
    // would take more steps down to the compact one, then shrinks only if it still cannot fit.
    let max_width = (area.width() * (1.0 - HOME_PANEL_MIN_SIDE_SHARE))
        .min((area.width() - 2.0 * HOME_PANEL_MARGIN).max(0.0))
        .max(0.0);
    let max_height = positive_clamp(area.height() - 2.0 * HOME_PANEL_MARGIN, 0.0, HOME_PANEL_MAX_HEIGHT);
    let large = area.width().is_finite() && HOME_PANEL_WIDTH <= max_width;
    let (panel_width, panel_height) = if large {
        (HOME_PANEL_WIDTH, max_height)
    } else {
        // The compact card stays square, so a narrow canvas shrinks its height with it.
        let side = HOME_PANEL_COMPACT_WIDTH.min(max_width).min(max_height);
        (side, side)
    };
    let panel = Rect::from_center_size(area.center(), vec2(panel_width, panel_height));
    let split = large && panel.height() >= 460.0;
    if !split {
        return HomeLayout { panel, content: panel.shrink2(vec2(26.0, 24.0)), art: panel, split };
    }
    let padding = vec2(30.0, 26.0);
    let content_width = positive_clamp(panel.width() * 0.51, 430.0, 600.0).min((panel.width() - 160.0).max(0.0));
    let content = Rect::from_min_max(panel.min + padding, pos2(panel.left() + content_width, panel.bottom() - padding.y));
    let art = Rect::from_min_max(pos2(content.right() + 16.0, panel.top() + 14.0), panel.max - vec2(18.0, 18.0));
    HomeLayout { panel, content, art, split }
}

fn positive_clamp(value: f32, min: f32, max: f32) -> f32 {
    let min = min.max(0.0);
    let max = max.max(min);
    if value.is_finite() { value.max(min).min(max) } else { min }
}

fn paint_panel(ui: &Ui, rect: Rect, p: &Palette) {
    if !rect.is_positive() {
        return;
    }
    let painter = ui.painter_at(rect.expand(28.0));
    let shadow = egui::epaint::Shadow { offset: [0, 6], blur: 18, spread: 0, color: p.shadow };
    painter.add(shadow.as_shape(rect, 24.0));
    painter.rect_filled(rect, 24.0, p.panel_top);
    painter.rect_stroke(rect, 24.0, Stroke::new(1.0, p.border), StrokeKind::Inside);
    painter.line_segment(
        [rect.left_top() + vec2(26.0, 1.0), rect.right_top() + vec2(-26.0, 1.0)],
        Stroke::new(1.0, if p.dark { Color32::from_white_alpha(32) } else { Color32::from_white_alpha(230) }),
    );
}

fn paint_content(app: &mut PhotocraftApp, ui: &mut Ui, layout: HomeLayout, p: &Palette) {
    let show_recent = app.session.prefs().interface.show_home_recent_files;
    let recent: Vec<String> =
        if cfg!(target_arch = "wasm32") || !show_recent { Vec::new() } else { app.ui.recent_files.iter().take(HOME_RECENT_LIMIT).cloned().collect() };
    ui.scope_builder(egui::UiBuilder::new().max_rect(layout.content).layout(egui::Layout::top_down(Align::Min)), |ui| {
        ui.set_width(layout.content.width());
        ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
        egui::ScrollArea::vertical()
            .id_salt("artforge-home-scroll")
            .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::VisibleWhenNeeded)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                paint_hero(ui, p);
                ui.add_space(18.0);
                paint_actions(app, ui, p);
                ui.add_space(12.0);
                paint_drop_hint(app, ui, p);
                if !recent.is_empty() {
                    ui.add_space(18.0);
                    paint_divider(ui, p);
                    ui.add_space(12.0);
                    paint_recent(app, ui, p, &recent);
                }
                ui.add_space(10.0);
            });
    });
}

fn paint_hero(ui: &mut Ui, p: &Palette) {
    let title_size = if ui.available_width() < 430.0 {
        38.0
    } else if ui.available_width() < 620.0 {
        46.0
    } else {
        52.0
    };
    // The brand title is painted, not labelled, like the upstream start screen: the About dialog
    // owns the accessible product name, so the Home title must not duplicate it in the widget tree.
    let galley = ui.painter().layout_no_wrap(brand::app_name().into(), crate::theme::semibold(title_size), p.ink);
    let (rect, _) = ui.allocate_exact_size(galley.size(), Sense::hover());
    ui.painter().galley(rect.min, galley, p.ink);
    ui.add_space(12.0);
    ui.label(egui::RichText::new(tl!("Create a new document or open an existing file.")).size(14.0).color(p.muted));
}

fn paint_actions(app: &mut PhotocraftApp, ui: &mut Ui, p: &Palette) {
    let new_label = crate::shortcuts::command_label(app, "New document…", "file.new");
    let open_label = crate::shortcuts::command_label(app, "Open…", "file.open");
    if ui.available_width() < 390.0 {
        if action_card(ui, vec2(ui.available_width(), 58.0), "plus", &new_label, "NEW CANVAS", true, p).clicked() {
            let fields = app.new_document_fields();
            app.ui.open_dialog(crate::state::DialogKind::NewDocument, fields);
        }
        ui.add_space(8.0);
        if action_card(ui, vec2(ui.available_width(), 58.0), "folder-open", &open_label, "FROM DEVICE", false, p).clicked() {
            let _ = app.open_dialog_file();
        }
        return;
    }
    let gap = 10.0;
    let width = ((ui.available_width() - gap) * 0.5).max(0.0);
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = gap;
        if action_card(ui, vec2(width, 66.0), "plus", &new_label, "NEW CANVAS", true, p).clicked() {
            let fields = app.new_document_fields();
            app.ui.open_dialog(crate::state::DialogKind::NewDocument, fields);
        }
        if action_card(ui, vec2(width, 66.0), "folder-open", &open_label, "FROM DEVICE", false, p).clicked() {
            let _ = app.open_dialog_file();
        }
    });
}

fn action_card(ui: &mut Ui, size: Vec2, icon: &str, label: &str, eyebrow: &str, primary: bool, p: &Palette) -> Response {
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    let hovered = response.hovered();
    let pressed = response.is_pointer_button_down_on();
    let painter = ui.painter_at(rect.expand(6.0));
    if primary {
        let a = if hovered { mix(p.blue, p.ink, 0.08) } else { p.blue };
        painter.rect_filled(rect, 12.0, a);
        painter.rect_stroke(rect, 12.0, Stroke::new(1.0, with_alpha(p.primary_text, 56)), StrokeKind::Inside);
    } else {
        painter.rect_filled(rect, 12.0, if hovered { p.glass_hover } else { p.glass });
        painter.rect_stroke(rect, 12.0, Stroke::new(1.0, if hovered { p.violet } else { p.border }), StrokeKind::Inside);
    }
    if pressed {
        painter.rect_filled(rect, 12.0, Color32::from_black_alpha(24));
    }
    let icon_rect = Rect::from_center_size(pos2(rect.left() + 25.0, rect.center().y), vec2(30.0, 30.0));
    painter.rect_filled(icon_rect, 9.0, if primary { with_alpha(p.primary_text, 35) } else { p.glass_hover });
    icons::paint(ui, icon_rect, icon, 16.0, if primary { p.primary_text } else { p.ink });
    let text_x = rect.left() + 47.0;
    painter.text(
        pos2(text_x, rect.center().y - 8.0),
        Align2::LEFT_CENTER,
        eyebrow,
        FontId::monospace(8.5),
        if primary { with_alpha(p.primary_text, 190) } else { p.faint },
    );
    let label_color = if primary { p.primary_text } else { p.ink };
    let available = (rect.right() - text_x - 10.0).max(0.0);
    let galley = ui.painter().layout(label.to_owned(), FontId::proportional(13.5), label_color, available);
    painter.galley(pos2(text_x, rect.center().y + 3.0), galley, label_color);
    response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, label));
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

fn paint_drop_hint(app: &PhotocraftApp, ui: &mut Ui, p: &Palette) {
    let msg = if app.services.is_wayland {
        crate::i18n::fmt(tl!("Use File › Open, or paste a copied image with {paste}."), &[("paste", &crate::notices::paste_hint(app))])
    } else {
        tl!("Drop an image or PSD anywhere to open it.").to_owned()
    };
    let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width().max(0.0), 30.0), Sense::hover());
    ui.painter().rect_filled(rect, 9.0, with_alpha(p.glass, if p.dark { 12 } else { 80 }));
    let icon = Rect::from_center_size(pos2(rect.left() + 17.0, rect.center().y), vec2(16.0, 16.0));
    icons::paint(ui, icon, "image", 13.0, p.faint);
    let galley = crate::tab_strip::elided(ui, &msg, FontId::proportional(11.5), p.faint, (rect.width() - 40.0).max(0.0));
    ui.painter().galley(pos2(rect.left() + 31.0, rect.center().y - galley.size().y * 0.5), galley, p.faint);
    response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Label, true, tl!("Drop an image or PSD anywhere to open it.")));
}

fn paint_divider(ui: &Ui, p: &Palette) {
    let rect = Rect::from_min_size(ui.cursor().min, vec2(ui.available_width().max(0.0), 1.0));
    ui.painter().line_segment([rect.left_center(), rect.right_center()], Stroke::new(1.0, p.border));
}

fn paint_recent(app: &mut PhotocraftApp, ui: &mut Ui, p: &Palette, recent: &[String]) {
    let width = ui.available_width().max(0.0);
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(tl!("Recent")).font(crate::theme::semibold(12.0)).color(p.ink));
        ui.add_space(8.0);
        ui.label(egui::RichText::new(format!("{:02}", recent.len())).font(FontId::monospace(9.5)).color(p.faint));
    });
    ui.add_space(6.0);
    let mut open = None;
    let mut remove = None;
    let mut clear = false;
    for path in recent {
        let name = crate::file_open::display_name(path);
        let folder = std::path::Path::new(path).parent().map(|d| d.to_string_lossy().into_owned()).unwrap_or_default();
        let (row, response) = ui.allocate_exact_size(vec2(width, HOME_RECENT_ROW), Sense::click());
        let hovered = ui.rect_contains_pointer(row);
        if hovered {
            ui.painter().rect_filled(row, 9.0, p.glass_hover);
        }
        let accent = Rect::from_min_max(row.left_top() + vec2(0.0, 7.0), pos2(row.left() + 2.0, row.bottom() - 7.0));
        ui.painter().rect_filled(accent, 1.0, p.blue);
        let icon_rect = Rect::from_center_size(pos2(row.left() + 18.0, row.center().y), vec2(20.0, 20.0));
        ui.painter().circle_filled(icon_rect.center(), 9.5, with_alpha(p.blue, if p.dark { 30 } else { 20 }));
        icons::paint(ui, icon_rect, "file", 13.0, p.muted);
        let close = Rect::from_center_size(pos2(row.right() - 17.0, row.center().y), vec2(22.0, 22.0));
        let right = close.left() - 5.0;
        let text_x = row.left() + 34.0;
        let name_g = crate::tab_strip::elided(ui, &name, FontId::proportional(12.5), p.ink, (width * 0.52).max(48.0));
        ui.painter().galley(pos2(text_x, row.center().y - name_g.size().y * 0.5), name_g.clone(), p.ink);
        let folder_max = (right - (text_x + name_g.size().x + 12.0)).max(0.0);
        if folder_max > 24.0 && !folder.is_empty() {
            let folder_g = crate::tab_strip::elided(ui, &folder, FontId::proportional(11.0), p.faint, folder_max);
            ui.painter().galley(pos2(right - folder_g.size().x, row.center().y - folder_g.size().y * 0.5), folder_g, p.faint);
        }
        response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, &name));
        let response = response.on_hover_text(path).on_hover_cursor(egui::CursorIcon::PointingHand);
        // Touch input has no hover: keep both the icon and its hit target available.
        let x = ui.interact(close, response.id.with("remove"), Sense::click());
        let label = crate::i18n::fmt(tl!("Remove {name} from Recent"), &[("name", &name)]);
        x.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, &label));
        let color = crate::icons::button_chrome(ui, close, false, x.hovered());
        crate::icons::paint(ui, close, "x", 12.0, color);
        if x.on_hover_text(tl!("Remove from Recent")).clicked() {
            remove = Some(path.clone());
        }
        if response.clicked() && remove.is_none() {
            open = Some(path.clone());
        }
        response.context_menu(|ui| {
            if ui.button(tl!("Remove from Recent")).clicked() {
                remove = Some(path.clone());
                ui.close();
            }
            if ui.button(tl!("Clear Recent Files")).clicked() {
                clear = true;
                ui.close();
            }
        });
    }
    if clear {
        app.clear_recent();
    } else if let Some(path) = remove {
        app.remove_recent(&path);
    }
    if let Some(path) = open
        && let Err(error) = app.open_path(&path)
    {
        app.open_failed(&crate::file_open::display_name(&path), &error);
    }
}

fn paint_artwork(ui: &Ui, rect: Rect, p: &Palette) {
    if !rect.is_positive() {
        return;
    }
    let painter = ui.painter_at(rect);
    let center = rect.center() + vec2(10.0, 8.0);
    let radius = rect.width().min(rect.height()) * 0.29;
    paint_glow(&painter, center, radius * 1.8, p.blue);

    for layer in 0..2 {
        let r = radius * (1.0 - layer as f32 * 0.13);
        let start = layer as f32 * 0.8 + 0.45;
        paint_arc(&painter, center, r, start, start + 2.15, Stroke::new(1.2, with_alpha(if layer == 0 { p.blue } else { p.violet }, 150 - layer as u8 * 28)));
    }

    let inner = radius * 0.57;
    painter.circle_filled(center, inner + 12.0, with_alpha(p.glass, if p.dark { 72 } else { 166 }));
    painter.circle_stroke(center, inner + 12.0, Stroke::new(1.0, with_alpha(p.blue, 92)));
    paint_glow(&painter, center, inner * 1.3, p.violet);
    painter.circle_filled(center, inner, p.panel_top);
    let mark = Rect::from_center_size(center, vec2(inner * 1.16, inner * 1.16));
    brand::paint_mark(ui, mark);

    paint_swatch(&painter, Rect::from_center_size(center + vec2(-radius * 1.12, -radius * 0.72), vec2(66.0, 88.0)), p.blue, p, -0.08);
    paint_swatch(&painter, Rect::from_center_size(center + vec2(radius * 1.08, radius * 0.46), vec2(78.0, 106.0)), p.violet, p, 0.06);

    let label_pos = pos2(rect.left() + 12.0, rect.bottom() - 18.0);
    painter.text(label_pos, Align2::LEFT_CENTER, "ARTFORGE / 01", FontId::monospace(9.5), with_alpha(p.faint, 210));
    painter.line_segment([label_pos + vec2(108.0, 0.0), pos2(rect.right() - 18.0, rect.bottom() - 18.0)], Stroke::new(1.0, p.border));
}

fn paint_compact_artwork(ui: &Ui, rect: Rect, p: &Palette) {
    if !rect.is_positive() {
        return;
    }
    let painter = ui.painter_at(rect);
    let radius = 46.0;
    let inset = 14.0;
    let center = pos2(rect.right() - radius - inset, rect.top() + radius + inset);
    paint_glow(&painter, center, radius * 1.55, p.blue);
    for layer in 0..2 {
        let start = layer as f32 * 0.7 + 0.4;
        paint_arc(
            &painter,
            center,
            radius - layer as f32 * 11.0,
            start,
            start + 2.4,
            Stroke::new(1.0, with_alpha(if layer == 0 { p.blue } else { p.violet }, 58)),
        );
    }
    let watermark = with_alpha(p.blue, if p.dark { 12 } else { 14 });
    painter.text(pos2(rect.left() + 12.0, rect.bottom() - 44.0), Align2::LEFT_BOTTOM, "01", FontId::monospace(92.0), watermark);
    painter.line_segment([pos2(rect.left() + 18.0, rect.bottom() - 32.0), pos2(rect.right() - 16.0, rect.bottom() - 32.0)], Stroke::new(1.0, p.border));
}

fn paint_swatch(painter: &egui::Painter, rect: Rect, color: Color32, p: &Palette, angle: f32) {
    let points = rotated_rect(rect, angle);
    painter.add(egui::Shape::convex_polygon(points.clone(), with_alpha(color, if p.dark { 52 } else { 42 }), Stroke::new(1.0, with_alpha(color, 130))));
    let inset = rect.shrink(9.0);
    painter.add(egui::Shape::convex_polygon(
        rotated_rect(inset, angle),
        with_alpha(if p.dark { Color32::WHITE } else { p.ink }, if p.dark { 13 } else { 26 }),
        Stroke::NONE,
    ));
    let dot = rect.center() + vec2(rect.width() * 0.27, -rect.height() * 0.31);
    painter.circle_filled(dot, 3.2, with_alpha(color, 230));
}

fn paint_arc(painter: &egui::Painter, center: Pos2, radius: f32, start: f32, end: f32, stroke: Stroke) {
    if radius <= 0.0 || !radius.is_finite() {
        return;
    }
    let mut points = Vec::with_capacity(42);
    for i in 0..=40 {
        let angle = start + (end - start) * (i as f32 / 40.0);
        points.push(center + vec2(angle.cos(), angle.sin()) * radius);
    }
    painter.add(egui::Shape::line(points, stroke));
}

fn paint_glow(painter: &egui::Painter, center: Pos2, radius: f32, color: Color32) {
    if radius <= 0.0 || !radius.is_finite() {
        return;
    }
    for i in (1..=10).rev() {
        let u = i as f32 / 10.0;
        painter.circle_filled(center, radius * u, with_alpha(color, (2.0 + (1.0 - u) * 10.0) as u8));
    }
}

fn rotated_rect(rect: Rect, angle: f32) -> Vec<Pos2> {
    let (sin, cos) = angle.sin_cos();
    let half = rect.size() * 0.5;
    [vec2(-half.x, -half.y), vec2(half.x, -half.y), vec2(half.x, half.y), vec2(-half.x, half.y)]
        .into_iter()
        .map(|v| rect.center() + vec2(v.x * cos - v.y * sin, v.x * sin + v.y * cos))
        .collect()
}

fn with_alpha(color: Color32, alpha: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), alpha)
}

fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = if t.is_finite() { t.clamp(0.0, 1.0) } else { 0.0 };
    let channel = |a: u8, b: u8| (f32::from(a) + (f32::from(b) - f32::from(a)) * t).round().clamp(0.0, 255.0) as u8;
    Color32::from_rgba_unmultiplied(channel(a.r(), b.r()), channel(a.g(), b.g()), channel(a.b(), b.b()), channel(a.a(), b.a()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_theme_keeps_its_own_tokens() {
        for kind in ThemeKind::ALL {
            assert!(use_custom_home(kind), "{kind:?} must use the ArtForge start screen");
            let t = Tokens::for_kind(kind);
            let p = Palette::new(&t);
            assert_eq!(p.ink, t.text, "{kind:?}");
            assert_eq!(p.muted, t.text_dim, "{kind:?}");
            assert_eq!(p.faint, t.text_faint, "{kind:?}");
            assert_eq!(p.blue, t.primary_bg, "{kind:?}");
            assert_eq!(p.border, t.card_border, "{kind:?}");
            assert_eq!(p.shadow, t.shadow, "{kind:?}");
        }
    }

    #[test]
    fn custom_home_keeps_two_fixed_card_sizes() {
        // The large card is fixed at 900 × 560 while it still leaves 20% of the width at the sides.
        for width in [1200.0, 1280.0, 1920.0] {
            let home = layout(Rect::from_min_size(Pos2::ZERO, vec2(width, 800.0)));
            assert_eq!(home.panel.size(), vec2(HOME_PANEL_WIDTH, HOME_PANEL_MAX_HEIGHT), "{width}");
            assert!(home.split, "{width}");
        }
        // Otherwise the compact card is a fixed square at the same height.
        for width in [800.0, 900.0, 1100.0] {
            let home = layout(Rect::from_min_size(Pos2::ZERO, vec2(width, 800.0)));
            assert_eq!(home.panel.size(), vec2(HOME_PANEL_COMPACT_WIDTH, HOME_PANEL_MAX_HEIGHT), "{width}");
            assert_eq!(home.panel.width(), home.panel.height() * HOME_PANEL_COMPACT_ASPECT, "{width}");
            assert!(!home.split, "{width}");
        }
        // A canvas narrower than the square shrinks it, still square.
        for (width, side) in [(650.0, 520.0), (520.0, 416.0)] {
            let home = layout(Rect::from_min_size(Pos2::ZERO, vec2(width, 620.0)));
            assert!((home.panel.width() - side).abs() < 0.01, "{width}");
            assert!((home.panel.height() - side).abs() < 0.01, "{width}");
        }
    }

    #[test]
    fn custom_home_layout_is_contained_and_switches_to_compact_cleanly() {
        for (size, should_split) in [(vec2(1920.0, 800.0), true), (vec2(1280.0, 800.0), true), (vec2(1100.0, 800.0), false), (vec2(620.0, 760.0), false), (vec2(420.0, 620.0), false)] {
            let area = Rect::from_min_size(Pos2::ZERO, size);
            let home = layout(area);
            assert!(area.contains_rect(home.panel));
            assert!(home.panel.contains_rect(home.content));
            if should_split {
                assert!(home.split);
                assert!(home.panel.contains_rect(home.art));
                assert!(home.art.left() >= home.content.right());
            } else {
                assert!(!home.split);
            }
        }
    }

    #[test]
    fn custom_artwork_helpers_stay_inside_valid_ranges() {
        assert_eq!(with_alpha(Color32::RED, 0).a(), 0);
        assert_eq!(mix(Color32::BLACK, Color32::WHITE, 0.5), Color32::from_gray(128));
        let points = rotated_rect(Rect::from_center_size(Pos2::ZERO, vec2(20.0, 10.0)), 0.5);
        assert_eq!(points.len(), 4);
        assert!(points.iter().all(|point| point.x.is_finite() && point.y.is_finite()));
    }

    #[test]
    fn recent_remove_is_available_without_hover_and_does_not_open_file() {
        use egui_kittest::kittest::Queryable;

        let mut app = PhotocraftApp::new(photocraft_engine::Session::new(), crate::Services::default());
        app.ui.recent_files = vec!["/recent/first.png".into(), "/recent/second.png".into()];
        let mut h = egui_kittest::Harness::builder().with_size(vec2(640.0, 240.0)).build_ui_state(
            |ui, (app, ready): &mut (PhotocraftApp, bool)| {
                if *ready {
                    let recent = app.ui.recent_files.clone();
                    paint_recent(app, ui, &Palette::new(&Tokens::for_kind(ThemeKind::ArkuiProDark)), &recent);
                }
            },
            (app, false),
        );
        PhotocraftApp::setup_context(&h.ctx, ThemeKind::ArkuiProDark);
        h.state_mut().1 = true;
        h.run_steps(3);

        // No pointer has entered a row, so a hover-only button would be absent.
        assert!(h.query_by_label("Remove first.png from Recent").is_some());
        assert!(h.query_by_label("Remove second.png from Recent").is_some());
        h.get_by_label("Remove first.png from Recent").click();
        h.run_steps(3);
        assert_eq!(h.state().0.ui.recent_files, vec!["/recent/second.png"]);
        assert!(h.state().0.ui.dialogs.is_empty(), "removing must not try to open the missing file");
        assert!(h.query_by_label("Remove first.png from Recent").is_none());
        assert!(h.query_by_label("Remove second.png from Recent").is_some());
    }

    #[test]
    fn custom_home_keeps_document_entry_points_live() {
        use egui_kittest::kittest::Queryable;

        let mut app = PhotocraftApp::new(photocraft_engine::Session::new(), crate::Services::default());
        app.ui.theme = ThemeKind::ArkuiProDark;
        let mut h = egui_kittest::Harness::builder().with_size(vec2(1280.0, 800.0)).build_ui_state(
            |ui, (app, ready): &mut (PhotocraftApp, bool)| {
                if *ready {
                    egui::CentralPanel::default().frame(egui::Frame::NONE).show(ui, |ui| crate::canvas::document_area(app, ui));
                }
            },
            (app, false),
        );
        PhotocraftApp::setup_context(&h.ctx, ThemeKind::ArkuiProDark);
        h.state_mut().1 = true;
        h.run_steps(3);

        assert!(h.query_by_label_contains("New document…").is_some());
        assert!(h.query_by_label_contains("Open…").is_some());
        h.get_by_label_contains("New document…").click();
        h.run_steps(2);
        assert!(h.state().0.ui.dialogs.iter().any(|dialog| dialog.kind == crate::state::DialogKind::NewDocument));
    }
}
