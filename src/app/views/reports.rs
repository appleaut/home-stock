//! แท็บ "รายงาน" — สรุปยอด + export CSV (UTF-8 + BOM ให้ Excel อ่านภาษาไทยได้)

use std::collections::BTreeMap;
use std::path::PathBuf;

use eframe::egui;
use egui::{Color32, CornerRadius, Margin, RichText, Stroke};

use super::days_until;
use crate::app::App;

const ORANGE: Color32 = Color32::from_rgb(220, 120, 0);
const RED: Color32 = Color32::from_rgb(200, 40, 40);
const GREEN: Color32 = Color32::from_rgb(16, 124, 16);

pub fn show(app: &mut App, ui: &mut egui::Ui) {
    ui.add_space(8.0);

    // ── คำนวณตัวเลขสรุป ──
    let total = app.items.len();
    let total_qty: i64 = app.items.iter().map(|i| i.quantity).sum();
    let low = app.items.iter().filter(|i| i.quantity <= i.min_quantity).count();
    let days = app.alert_days;
    let expiring = app
        .items
        .iter()
        .filter_map(|i| days_until(&i.expiry_date))
        .filter(|d| *d <= days)
        .count();

    // สรุปตามหมวดหมู่ (เรียงชื่อ)
    let mut by_cat: BTreeMap<String, (usize, i64)> = BTreeMap::new();
    for i in &app.items {
        let name = app.category_name(i.category_id);
        let name = if name.is_empty() { "— ไม่ระบุ —".to_string() } else { name };
        let e = by_cat.entry(name).or_default();
        e.0 += 1;
        e.1 += i.quantity;
    }

    let mut export: Option<ExportKind> = None;

    // คอลัมน์กว้างคงที่ จัดกึ่งกลางด้วยระยะขอบซ้าย (เนื้อหาชิดซ้าย)
    let avail = ui.available_width();
    let col_w = 720.0_f32.min(avail - 16.0);
    let left = ((avail - col_w) / 2.0).max(0.0);

    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.add_space(left);
            ui.vertical(|ui| {
                ui.set_width(col_w);
                let accent = ui.visuals().hyperlink_color;

                ui.heading("รายงานสรุป");
                ui.add_space(12.0);

                // ── การ์ดตัวเลข (KPI) เรียง 4 ช่อง ──
                ui.horizontal(|ui| {
                    let gap = 10.0;
                    let w = (col_w - gap * 3.0) / 4.0 - 4.0;
                    stat_card(ui, &total.to_string(), "รายการทั้งหมด", accent, w);
                    ui.add_space(gap);
                    stat_card(ui, &total_qty.to_string(), "ชิ้นรวมทุกหน่วย", GREEN, w);
                    ui.add_space(gap);
                    stat_card(ui, &low.to_string(), "ของใกล้หมด", ORANGE, w);
                    ui.add_space(gap);
                    stat_card(
                        ui,
                        &expiring.to_string(),
                        &format!("ใกล้/หมดอายุ ({} วัน)", days),
                        RED,
                        w,
                    );
                });

                ui.add_space(16.0);

                // ── สรุปตามหมวดหมู่ ──
                ui.label(RichText::new("สรุปตามหมวดหมู่").size(16.0).strong());
                ui.add_space(4.0);
                egui::Frame::group(ui.style())
                    .inner_margin(Margin::same(12))
                    .corner_radius(CornerRadius::same(10))
                    .show(ui, |ui| {
                        ui.set_width(col_w - 24.0);
                        if by_cat.is_empty() {
                            ui.weak("— ยังไม่มีรายการ —");
                        } else {
                            egui::Grid::new("report_by_cat")
                                .num_columns(3)
                                .spacing([24.0, 8.0])
                                .striped(true)
                                .min_col_width((col_w - 72.0) / 3.0)
                                .show(ui, |ui| {
                                    ui.strong("หมวดหมู่");
                                    ui.strong("จำนวนรายการ");
                                    ui.strong("ชิ้นรวม");
                                    ui.end_row();
                                    for (name, (count, qty)) in &by_cat {
                                        ui.label(name);
                                        ui.label(count.to_string());
                                        ui.label(qty.to_string());
                                        ui.end_row();
                                    }
                                });
                        }
                    });

                ui.add_space(16.0);

                // ── Export CSV ──
                ui.label(RichText::new("ส่งออกข้อมูล (CSV)").size(16.0).strong());
                ui.add_space(4.0);
                egui::Frame::group(ui.style())
                    .inner_margin(Margin::same(12))
                    .corner_radius(CornerRadius::same(10))
                    .show(ui, |ui| {
                        ui.set_width(col_w - 24.0);
                        ui.weak("ไฟล์เข้ารหัส UTF-8 (มี BOM) เปิดใน Excel แล้วอ่านภาษาไทยได้");
                        ui.add_space(10.0);
                        ui.horizontal(|ui| {
                            let h = 38.0;
                            let in_btn = egui::Button::new(
                                RichText::new("บันทึกรายการของ").color(Color32::WHITE),
                            )
                            .fill(accent)
                            .corner_radius(CornerRadius::same(8))
                            .min_size(egui::vec2(180.0, h));
                            if ui.add(in_btn).clicked() {
                                export = Some(ExportKind::Items);
                            }
                            ui.add_space(10.0);
                            let tx_btn = egui::Button::new(
                                RichText::new("บันทึกประวัติ").color(Color32::WHITE),
                            )
                            .fill(accent)
                            .corner_radius(CornerRadius::same(8))
                            .min_size(egui::vec2(180.0, h));
                            if ui.add(tx_btn).clicked() {
                                export = Some(ExportKind::Transactions);
                            }
                        });
                    });
            });
        });
    });

    // จัดการ export หลังวาด UI เสร็จ
    match export {
        Some(ExportKind::Items) => {
            if let Some(path) = save_dialog("items.csv") {
                match export_items(app, &path) {
                    Ok(_) => app.set_ok(format!("บันทึกรายการของไปที่ {}", path.display())),
                    Err(e) => app.set_err(format!("export ไม่สำเร็จ: {}", e)),
                }
            }
        }
        Some(ExportKind::Transactions) => {
            if let Some(path) = save_dialog("transactions.csv") {
                match export_transactions(app, &path) {
                    Ok(_) => app.set_ok(format!("บันทึกประวัติไปที่ {}", path.display())),
                    Err(e) => app.set_err(format!("export ไม่สำเร็จ: {}", e)),
                }
            }
        }
        None => {}
    }
}

enum ExportKind {
    Items,
    Transactions,
}

/// การ์ดตัวเลขสรุปหนึ่งช่อง: ตัวเลขใหญ่ + ป้ายกำกับ
fn stat_card(ui: &mut egui::Ui, value: &str, label: &str, color: Color32, width: f32) {
    egui::Frame::new()
        .fill(color.gamma_multiply(0.10))
        .stroke(Stroke::new(1.0, color.gamma_multiply(0.40)))
        .corner_radius(CornerRadius::same(10))
        .inner_margin(Margin::same(14))
        .show(ui, |ui| {
            ui.set_width(width);
            ui.vertical(|ui| {
                ui.label(RichText::new(value).size(28.0).strong().color(color));
                ui.label(RichText::new(label).size(12.5).weak());
            });
        });
}

fn save_dialog(default_name: &str) -> Option<PathBuf> {
    rfd::FileDialog::new()
        .add_filter("CSV", &["csv"])
        .set_file_name(default_name)
        .save_file()
}

/// เขียน Vec<u8> ที่มี BOM นำหน้าลงไฟล์
fn write_with_bom(path: &PathBuf, body: Vec<u8>) -> anyhow::Result<()> {
    let mut out = Vec::with_capacity(body.len() + 3);
    out.extend_from_slice(&[0xEF, 0xBB, 0xBF]); // UTF-8 BOM
    out.extend_from_slice(&body);
    std::fs::write(path, out)?;
    Ok(())
}

fn export_items(app: &App, path: &PathBuf) -> anyhow::Result<()> {
    let mut wtr = csv::Writer::from_writer(vec![]);
    wtr.write_record(["ชื่อ", "หมวด", "ตำแหน่ง", "หน่วย", "คงเหลือ", "ขั้นต่ำ", "บาร์โค้ด", "วันหมดอายุ"])?;
    for i in &app.items {
        wtr.write_record([
            i.name.as_str(),
            &app.category_name(i.category_id),
            &app.location_name(i.location_id),
            i.unit.as_str(),
            &i.quantity.to_string(),
            &i.min_quantity.to_string(),
            i.barcode.as_deref().unwrap_or(""),
            i.expiry_date.as_deref().unwrap_or(""),
        ])?;
    }
    let body = wtr.into_inner()?;
    write_with_bom(path, body)
}

fn export_transactions(app: &App, path: &PathBuf) -> anyhow::Result<()> {
    let mut wtr = csv::Writer::from_writer(vec![]);
    wtr.write_record(["เวลา", "ของ", "ประเภท", "จำนวน", "ผู้ทำรายการ", "หมายเหตุ"])?;
    for t in &app.transactions {
        wtr.write_record([
            t.timestamp.as_str(),
            t.item_name.as_str(),
            t.tx_type.label_th(),
            &t.quantity.to_string(),
            t.user_name.as_str(),
            t.note.as_str(),
        ])?;
    }
    let body = wtr.into_inner()?;
    write_with_bom(path, body)
}
