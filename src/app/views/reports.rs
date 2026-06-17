//! แท็บ "รายงาน" — สรุปยอด + export CSV (UTF-8 + BOM ให้ Excel อ่านภาษาไทยได้)

use std::path::PathBuf;

use eframe::egui;

use super::days_until;
use crate::app::App;

pub fn show(app: &mut App, ui: &mut egui::Ui) {
    ui.add_space(6.0);
    ui.heading("รายงานสรุป");
    ui.add_space(6.0);

    let total = app.items.len();
    let total_qty: i64 = app.items.iter().map(|i| i.quantity).sum();
    let low = app
        .items
        .iter()
        .filter(|i| i.quantity <= i.min_quantity)
        .count();
    let days = app.alert_days;
    let expiring = app
        .items
        .iter()
        .filter_map(|i| days_until(&i.expiry_date))
        .filter(|d| *d <= days)
        .count();

    egui::Grid::new("report_summary")
        .num_columns(2)
        .spacing([20.0, 6.0])
        .show(ui, |ui| {
            ui.label("จำนวนรายการทั้งหมด:");
            ui.strong(total.to_string());
            ui.end_row();
            ui.label("จำนวนชิ้นรวม (ทุกหน่วย):");
            ui.strong(total_qty.to_string());
            ui.end_row();
            ui.label("ของใกล้หมด:");
            ui.colored_label(egui::Color32::from_rgb(220, 120, 0), low.to_string());
            ui.end_row();
            ui.label(format!("ของใกล้/หมดอายุ (ใน {} วัน):", days));
            ui.colored_label(egui::Color32::from_rgb(200, 40, 40), expiring.to_string());
            ui.end_row();
        });

    ui.add_space(12.0);
    ui.separator();
    ui.heading("Export CSV");
    ui.add_space(4.0);
    ui.label("ไฟล์ CSV เข้ารหัส UTF-8 (มี BOM) เปิดใน Excel แล้วอ่านภาษาไทยได้");
    ui.add_space(6.0);

    ui.horizontal(|ui| {
        if ui.button("📤 Export รายการของ").clicked() {
            if let Some(path) = save_dialog("items.csv") {
                match export_items(app, &path) {
                    Ok(_) => app.set_ok(format!("บันทึกรายการของไปที่ {}", path.display())),
                    Err(e) => app.set_err(format!("export ไม่สำเร็จ: {}", e)),
                }
            }
        }
        if ui.button("📤 Export ประวัติ").clicked() {
            if let Some(path) = save_dialog("transactions.csv") {
                match export_transactions(app, &path) {
                    Ok(_) => app.set_ok(format!("บันทึกประวัติไปที่ {}", path.display())),
                    Err(e) => app.set_err(format!("export ไม่สำเร็จ: {}", e)),
                }
            }
        }
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
