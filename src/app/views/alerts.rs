//! แท็บ "แจ้งเตือน" — ของใกล้หมด และของใกล้/หมดอายุ

use eframe::egui;
use egui::{Align, Color32, CornerRadius, Layout, Margin, RichText};

use super::days_until;
use crate::app::App;

const ORANGE: Color32 = Color32::from_rgb(220, 120, 0);
const RED: Color32 = Color32::from_rgb(200, 40, 40);
const DARK_RED: Color32 = Color32::from_rgb(170, 20, 20);
const GREEN: Color32 = Color32::from_rgb(16, 124, 16);

/// หนึ่งแถวแจ้งเตือนที่เตรียมไว้แสดง (เลี่ยงการยืม App ใน closure)
struct AlertRow {
    title: String,
    subtitle: String,
    badge: String,
    color: Color32,
}

pub fn show(app: &mut App, ui: &mut egui::Ui) {
    ui.add_space(8.0);

    // ── เตรียมข้อมูลล่วงหน้า ──
    let low: Vec<AlertRow> = app
        .items
        .iter()
        .filter(|i| i.quantity <= i.min_quantity)
        .map(|i| {
            let loc = app.location_name(i.location_id);
            let where_ = if loc.is_empty() { String::new() } else { format!("{} • ", loc) };
            AlertRow {
                title: i.name.clone(),
                subtitle: format!("{}ขั้นต่ำ {} {}", where_, i.min_quantity, i.unit),
                badge: format!("เหลือ {} {}", i.quantity, i.unit),
                color: ORANGE,
            }
        })
        .collect();

    let days = app.alert_days;
    let mut expiring: Vec<(AlertRow, i64)> = app
        .items
        .iter()
        .filter_map(|i| days_until(&i.expiry_date).map(|d| (i, d)))
        .filter(|(_, d)| *d <= days)
        .map(|(i, d)| {
            let (color, status) = if d < 0 {
                (DARK_RED, format!("หมดอายุแล้ว {} วัน", -d))
            } else if d == 0 {
                (RED, "หมดอายุวันนี้".to_string())
            } else {
                (RED, format!("อีก {} วัน", d))
            };
            let row = AlertRow {
                title: i.name.clone(),
                subtitle: format!("หมดอายุ {}", i.expiry_date.clone().unwrap_or_default()),
                badge: status,
                color,
            };
            (row, d)
        })
        .collect();
    expiring.sort_by_key(|(_, d)| *d);
    let expiring: Vec<AlertRow> = expiring.into_iter().map(|(r, _)| r).collect();

    // ── เนื้อหา จัดกึ่งกลาง กว้างคงที่ ──
    ui.vertical_centered(|ui| {
        ui.set_max_width(660.0);

        // หัวเรื่อง + ตัวตั้งจำนวนวันเตือนล่วงหน้า
        ui.horizontal(|ui| {
            ui.heading("การแจ้งเตือน");
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label("วัน");
                ui.add(egui::DragValue::new(&mut app.alert_days).range(0..=365).speed(0.3));
                ui.label("เตือนหมดอายุล่วงหน้า:");
            });
        });
        ui.add_space(8.0);

        // แถบสรุปจำนวน
        ui.horizontal(|ui| {
            count_chip(ui, "ของใกล้หมด", low.len(), ORANGE);
            ui.add_space(8.0);
            count_chip(ui, "ใกล้/หมดอายุ", expiring.len(), RED);
        });
        ui.add_space(12.0);

        egui::ScrollArea::vertical().show(ui, |ui| {
            // ── ของใกล้หมด ──
            section(ui, "ของใกล้หมด", low.len(), ORANGE, |ui| {
                if low.is_empty() {
                    empty_ok(ui, "ไม่มีของใกล้หมด");
                } else {
                    rows(ui, &low);
                }
            });

            ui.add_space(14.0);

            // ── ของใกล้/หมดอายุ ──
            section(ui, "ของใกล้/หมดอายุ", expiring.len(), RED, |ui| {
                if expiring.is_empty() {
                    empty_ok(ui, "ไม่มีของใกล้หมดอายุ");
                } else {
                    rows(ui, &expiring);
                }
            });
        });
    });
}

/// วาดสี่เหลี่ยมสีเล็ก ๆ (ไม่พึ่งฟอนต์ — เลี่ยงปัญหา emoji ไม่มี glyph)
fn swatch(ui: &mut egui::Ui, color: Color32, size: f32) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::hover());
    ui.painter()
        .rect_filled(rect, CornerRadius::same((size / 2.0) as u8), color);
}

/// หัวข้อ section + การ์ดเนื้อหา
fn section(
    ui: &mut egui::Ui,
    title: &str,
    count: usize,
    color: Color32,
    body: impl FnOnce(&mut egui::Ui),
) {
    ui.horizontal(|ui| {
        swatch(ui, color, 14.0);
        ui.add_space(2.0);
        ui.label(RichText::new(title).size(16.0).strong());
        pill(ui, &count.to_string(), color);
    });
    ui.add_space(4.0);
    egui::Frame::group(ui.style())
        .inner_margin(Margin::same(12))
        .corner_radius(CornerRadius::same(10))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            body(ui);
        });
}

/// แสดงหลายแถว โดยมีเส้นคั่นบาง ๆ ระหว่างแถว
fn rows(ui: &mut egui::Ui, items: &[AlertRow]) {
    for (idx, r) in items.iter().enumerate() {
        if idx > 0 {
            ui.add_space(6.0);
            ui.separator();
            ui.add_space(6.0);
        }
        alert_row(ui, r);
    }
}

/// แถวเดียว: จุดสี + ชื่อ/รายละเอียด + badge สถานะชิดขวา
fn alert_row(ui: &mut egui::Ui, r: &AlertRow) {
    ui.horizontal(|ui| {
        swatch(ui, r.color, 10.0);
        ui.add_space(4.0);
        ui.vertical(|ui| {
            ui.label(RichText::new(&r.title).strong());
            ui.label(RichText::new(&r.subtitle).weak().size(12.0));
        });
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            pill(ui, &r.badge, r.color);
        });
    });
}

/// ป้ายตัวเลขสรุปแบบกล่อง
fn count_chip(ui: &mut egui::Ui, label: &str, count: usize, color: Color32) {
    egui::Frame::new()
        .fill(color.gamma_multiply(0.12))
        .stroke(egui::Stroke::new(1.0, color.gamma_multiply(0.45)))
        .corner_radius(CornerRadius::same(8))
        .inner_margin(Margin::symmetric(14, 8))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(count.to_string()).size(20.0).strong().color(color));
                ui.label(RichText::new(label).color(color));
            });
        });
}

/// ป้ายสถานะเล็ก ๆ (pill)
fn pill(ui: &mut egui::Ui, text: &str, color: Color32) {
    egui::Frame::new()
        .fill(color.gamma_multiply(0.16))
        .corner_radius(CornerRadius::same(6))
        .inner_margin(Margin::symmetric(8, 2))
        .show(ui, |ui| {
            ui.label(RichText::new(text).color(color).strong());
        });
}

/// สถานะ "ไม่มี" แบบเป็นมิตร (เขียว)
fn empty_ok(ui: &mut egui::Ui, text: &str) {
    ui.horizontal(|ui| {
        swatch(ui, GREEN, 10.0);
        ui.add_space(4.0);
        ui.label(RichText::new(text).color(GREEN));
    });
}
