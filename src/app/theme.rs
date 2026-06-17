//! ธีมแบบ Microsoft Fluent (Windows 11) — รองรับ Dark / Light

use eframe::egui::{
    self, Color32, CornerRadius, FontFamily, FontId, Stroke, TextStyle,
};

/// โหมดธีม
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    Light,
    Dark,
}

impl Theme {
    pub fn as_str(self) -> &'static str {
        match self {
            Theme::Light => "light",
            Theme::Dark => "dark",
        }
    }

    pub fn from_str(s: &str) -> Theme {
        match s {
            "dark" => Theme::Dark,
            _ => Theme::Light,
        }
    }

    pub fn toggled(self) -> Theme {
        match self {
            Theme::Light => Theme::Dark,
            Theme::Dark => Theme::Light,
        }
    }

    /// ไอคอน + ป้ายของปุ่มสลับ (แสดงสิ่งที่จะเปลี่ยนไป)
    pub fn toggle_label(self) -> &'static str {
        match self {
            Theme::Light => "🌙 โหมดมืด",
            Theme::Dark => "☀ โหมดสว่าง",
        }
    }
}

// สีเน้น (accent) สไตล์ Windows
const ACCENT_LIGHT: Color32 = Color32::from_rgb(0x00, 0x67, 0xC0); // #0067C0
const ACCENT_DARK: Color32 = Color32::from_rgb(0x4C, 0xC2, 0xFF); // #4CC2FF

/// ปรับ visuals + spacing + typography ให้เป็นสไตล์ Fluent แล้วนำไปใช้กับ context
pub fn apply(ctx: &egui::Context, theme: Theme) {
    let mut visuals = match theme {
        Theme::Dark => egui::Visuals::dark(),
        Theme::Light => egui::Visuals::light(),
    };
    let accent = match theme {
        Theme::Dark => ACCENT_DARK,
        Theme::Light => ACCENT_LIGHT,
    };

    // ── สีพื้นแบบ Fluent ──
    match theme {
        Theme::Light => {
            visuals.panel_fill = Color32::from_rgb(0xF3, 0xF3, 0xF3);
            visuals.window_fill = Color32::from_rgb(0xFB, 0xFB, 0xFB);
            visuals.extreme_bg_color = Color32::from_rgb(0xFF, 0xFF, 0xFF);
            visuals.faint_bg_color = Color32::from_rgb(0xEA, 0xEA, 0xEA);
            visuals.window_stroke = Stroke::new(1.0, Color32::from_rgb(0xDD, 0xDD, 0xDD));
        }
        Theme::Dark => {
            visuals.panel_fill = Color32::from_rgb(0x20, 0x20, 0x20);
            visuals.window_fill = Color32::from_rgb(0x2B, 0x2B, 0x2B);
            visuals.extreme_bg_color = Color32::from_rgb(0x1A, 0x1A, 0x1A);
            visuals.faint_bg_color = Color32::from_rgb(0x2D, 0x2D, 0x2D);
            visuals.window_stroke = Stroke::new(1.0, Color32::from_rgb(0x3A, 0x3A, 0x3A));
        }
    }

    visuals.hyperlink_color = accent;

    // ไฮไลต์การเลือก: ใช้ accent แบบโปร่งแสง เพื่อให้ตัวอักษรยังอ่านง่าย
    visuals.selection.bg_fill = match theme {
        Theme::Light => Color32::from_rgba_unmultiplied(0x00, 0x67, 0xC0, 60),
        Theme::Dark => Color32::from_rgba_unmultiplied(0x4C, 0xC2, 0xFF, 55),
    };
    visuals.selection.stroke = Stroke::new(1.0, accent);

    // ── มุมโค้งสไตล์ Fluent ──
    let r = CornerRadius::same(6);
    visuals.window_corner_radius = CornerRadius::same(8);
    visuals.widgets.noninteractive.corner_radius = r;
    visuals.widgets.inactive.corner_radius = r;
    visuals.widgets.hovered.corner_radius = r;
    visuals.widgets.active.corner_radius = r;
    visuals.widgets.open.corner_radius = r;

    // ปุ่มที่กำลังกด/เปิดอยู่ใช้สี accent
    visuals.widgets.active.bg_fill = accent;
    visuals.widgets.active.weak_bg_fill = accent;

    // ── รวมเข้ากับ style + ปรับ spacing/typography ──
    let mut style = (*ctx.global_style()).clone();
    style.visuals = visuals;

    style.spacing.item_spacing = egui::vec2(8.0, 8.0);
    style.spacing.button_padding = egui::vec2(12.0, 6.0);
    style.spacing.menu_margin = egui::Margin::same(6);
    style.spacing.interact_size.y = 28.0;

    style.text_styles = [
        (TextStyle::Heading, FontId::new(21.0, FontFamily::Proportional)),
        (TextStyle::Body, FontId::new(15.0, FontFamily::Proportional)),
        (TextStyle::Button, FontId::new(15.0, FontFamily::Proportional)),
        (TextStyle::Small, FontId::new(12.0, FontFamily::Proportional)),
        (TextStyle::Monospace, FontId::new(14.0, FontFamily::Monospace)),
    ]
    .into();

    ctx.set_global_style(style);
}
