//! แท็บ "แจ้งเตือน" — ของใกล้หมด และของใกล้/หมดอายุ

use eframe::egui;

use super::days_until;
use crate::app::App;

pub fn show(app: &mut App, ui: &mut egui::Ui) {
    ui.add_space(4.0);

    ui.horizontal(|ui| {
        ui.label("เตือนของหมดอายุล่วงหน้า (วัน):");
        ui.add(egui::DragValue::new(&mut app.alert_days).range(0..=365));
    });
    ui.separator();

    // ── ของใกล้หมด ──
    let low: Vec<_> = app
        .items
        .iter()
        .filter(|i| i.quantity <= i.min_quantity)
        .collect();

    ui.heading(format!("🟠 ของใกล้หมด ({})", low.len()));
    if low.is_empty() {
        ui.label("— ไม่มี —");
    } else {
        egui::Frame::group(ui.style()).show(ui, |ui| {
            for i in &low {
                ui.horizontal(|ui| {
                    ui.colored_label(egui::Color32::from_rgb(220, 120, 0), "●");
                    ui.label(format!(
                        "{} — เหลือ {} {} (ขั้นต่ำ {})",
                        i.name, i.quantity, i.unit, i.min_quantity
                    ));
                });
            }
        });
    }

    ui.add_space(10.0);

    // ── ของใกล้/หมดอายุ ──
    let days = app.alert_days;
    let mut expiring: Vec<(&crate::db::models::Item, i64)> = app
        .items
        .iter()
        .filter_map(|i| days_until(&i.expiry_date).map(|d| (i, d)))
        .filter(|(_, d)| *d <= days)
        .collect();
    expiring.sort_by_key(|(_, d)| *d);

    ui.heading(format!("🔴 ของใกล้/หมดอายุ ({})", expiring.len()));
    if expiring.is_empty() {
        ui.label("— ไม่มี —");
    } else {
        egui::Frame::group(ui.style()).show(ui, |ui| {
            for (i, d) in &expiring {
                let (color, status) = if *d < 0 {
                    (
                        egui::Color32::from_rgb(170, 20, 20),
                        format!("หมดอายุแล้ว {} วัน", -d),
                    )
                } else if *d == 0 {
                    (egui::Color32::from_rgb(200, 40, 40), "หมดอายุวันนี้".to_string())
                } else {
                    (egui::Color32::from_rgb(200, 40, 40), format!("อีก {} วัน", d))
                };
                ui.horizontal(|ui| {
                    ui.colored_label(color, "●");
                    ui.label(format!(
                        "{} — หมดอายุ {} ({})",
                        i.name,
                        i.expiry_date.clone().unwrap_or_default(),
                        status
                    ));
                });
            }
        });
    }
}
