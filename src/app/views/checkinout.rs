//! แท็บ "รับเข้า/เบิกออก" — เลือกของ (หรือสแกนบาร์โค้ด) แล้ว check-in/out

use eframe::egui;

use crate::app::App;
use crate::db::models::TxType;
use crate::db::queries;

pub fn show(app: &mut App, ui: &mut egui::Ui) {
    ui.add_space(8.0);

    let mut action: Option<TxType> = None;

    // จัดฟอร์มเป็น "การ์ด" กว้างคงที่ จัดกึ่งกลางแนวนอน
    ui.vertical_centered(|ui| {
        ui.set_max_width(560.0);

        ui.heading("รับเข้า / เบิกออก");
        ui.add_space(8.0);

        if app.current_user.is_none() {
            warning_banner(
                ui,
                "⚠ ยังไม่ได้เลือกผู้ทำรายการ (มุมบนขวา) — ระบบจะบันทึกโดยไม่ระบุชื่อ",
            );
            ui.add_space(8.0);
        }

        // ── การ์ดฟอร์ม ──
        egui::Frame::group(ui.style())
            .inner_margin(egui::Margin::same(16))
            .corner_radius(egui::CornerRadius::same(10))
            .show(ui, |ui| {
                egui::Grid::new("cio_grid")
                    .num_columns(2)
                    .spacing([14.0, 12.0])
                    .min_col_width(96.0)
                    .show(ui, |ui| {
                        // สแกน/ค้นหาบาร์โค้ด
                        ui.label("🔎 สแกนบาร์โค้ด");
                        ui.horizontal(|ui| {
                            let resp = ui.add(
                                egui::TextEdit::singleline(&mut app.cio.barcode_input)
                                    .desired_width(260.0)
                                    .hint_text("สแกนหรือพิมพ์รหัสแล้วกด Enter"),
                            );
                            let enter = resp.lost_focus()
                                && ui.input(|i| i.key_pressed(egui::Key::Enter));
                            if (ui.button("ค้นหา").clicked() || enter)
                                && !app.cio.barcode_input.trim().is_empty()
                            {
                                let code = app.cio.barcode_input.trim().to_string();
                                match queries::find_by_barcode(&app.db.conn, &code) {
                                    Ok(Some(item)) => {
                                        app.cio.selected_item = Some(item.id);
                                        app.cio.barcode_input.clear();
                                        app.set_ok(format!("เลือก \"{}\" แล้ว", item.name));
                                    }
                                    Ok(None) => app
                                        .set_err(format!("ไม่พบของที่มีบาร์โค้ด {}", code)),
                                    Err(e) => app.set_err(format!("ค้นหาไม่สำเร็จ: {}", e)),
                                }
                            }
                        });
                        ui.end_row();

                        // เลือกของจากรายการ
                        ui.label("📦 เลือกของ");
                        let label = match app.cio.selected_item {
                            Some(id) => app.item_name(id),
                            None => "— เลือกรายการ —".to_string(),
                        };
                        egui::ComboBox::from_id_salt("cio_item")
                            .width(320.0)
                            .selected_text(label)
                            .show_ui(ui, |ui| {
                                for it in &app.items {
                                    ui.selectable_value(
                                        &mut app.cio.selected_item,
                                        Some(it.id),
                                        &it.name,
                                    );
                                }
                            });
                        ui.end_row();

                        // คงเหลือปัจจุบัน (badge)
                        let current = app
                            .cio
                            .selected_item
                            .and_then(|id| app.items.iter().find(|i| i.id == id))
                            .map(|i| (i.quantity, i.unit.clone(), i.min_quantity));
                        ui.label("คงเหลือ");
                        match &current {
                            Some((qty, unit, min)) => stock_badge(ui, *qty, unit, *min),
                            None => {
                                ui.weak("—");
                            }
                        }
                        ui.end_row();

                        // จำนวน
                        ui.label("จำนวน");
                        ui.add(
                            egui::DragValue::new(&mut app.cio.qty)
                                .range(1..=1_000_000)
                                .speed(0.2),
                        );
                        ui.end_row();

                        // หมายเหตุ
                        ui.label("หมายเหตุ");
                        ui.add(
                            egui::TextEdit::singleline(&mut app.cio.note)
                                .desired_width(320.0)
                                .hint_text("เช่น ซื้อเพิ่ม / หุงข้าว (ไม่บังคับ)"),
                        );
                        ui.end_row();
                    });

                ui.add_space(14.0);
                ui.separator();
                ui.add_space(10.0);

                // ── ปุ่มทำรายการ: เต็มความกว้าง สองปุ่มเท่ากัน มีสีแยกชัด ──
                let has_item = app.cio.selected_item.is_some();
                let spacing = 10.0;
                let btn_w = (ui.available_width() - spacing) / 2.0;
                let btn_size = egui::vec2(btn_w, 44.0);

                ui.horizontal(|ui| {
                    let in_btn = egui::Button::new(
                        egui::RichText::new("⬇  รับเข้า")
                            .size(16.0)
                            .color(egui::Color32::WHITE),
                    )
                    .fill(egui::Color32::from_rgb(16, 124, 16)) // เขียว Fluent
                    .corner_radius(egui::CornerRadius::same(8))
                    .min_size(btn_size);
                    if ui.add_enabled(has_item, in_btn).clicked() {
                        action = Some(TxType::In);
                    }

                    ui.add_space(spacing);

                    let out_btn = egui::Button::new(
                        egui::RichText::new("⬆  เบิกออก")
                            .size(16.0)
                            .color(egui::Color32::WHITE),
                    )
                    .fill(egui::Color32::from_rgb(196, 43, 28)) // แดง Fluent
                    .corner_radius(egui::CornerRadius::same(8))
                    .min_size(btn_size);
                    if ui.add_enabled(has_item, out_btn).clicked() {
                        action = Some(TxType::Out);
                    }
                });

                if !has_item {
                    ui.add_space(6.0);
                    ui.weak("เลือกของหรือสแกนบาร์โค้ดก่อนทำรายการ");
                }
            });
    });

    if let Some(kind) = action {
        do_transaction(app, kind);
    }
}

/// แถบเตือนแบบมีพื้นหลังอ่อน
fn warning_banner(ui: &mut egui::Ui, text: &str) {
    let accent = egui::Color32::from_rgb(200, 120, 0);
    egui::Frame::new()
        .fill(accent.gamma_multiply(0.12))
        .stroke(egui::Stroke::new(1.0, accent.gamma_multiply(0.5)))
        .corner_radius(egui::CornerRadius::same(8))
        .inner_margin(egui::Margin::symmetric(12, 8))
        .show(ui, |ui| {
            ui.colored_label(accent, text);
        });
}

/// แสดงจำนวนคงเหลือเป็น badge (ส้มถ้าต่ำกว่า/เท่าขั้นต่ำ)
fn stock_badge(ui: &mut egui::Ui, qty: i64, unit: &str, min: i64) {
    let low = qty <= min;
    let color = if low {
        egui::Color32::from_rgb(220, 120, 0)
    } else {
        egui::Color32::from_rgb(16, 124, 16)
    };
    egui::Frame::new()
        .fill(color.gamma_multiply(0.14))
        .corner_radius(egui::CornerRadius::same(6))
        .inner_margin(egui::Margin::symmetric(10, 3))
        .show(ui, |ui| {
            let mut text = egui::RichText::new(format!("{} {}", qty, unit)).color(color).strong();
            if low {
                text = egui::RichText::new(format!("{} {} (ใกล้หมด)", qty, unit))
                    .color(color)
                    .strong();
            }
            ui.label(text);
        });
}

fn do_transaction(app: &mut App, kind: TxType) {
    let Some(item_id) = app.cio.selected_item else {
        app.set_err("กรุณาเลือกของก่อน");
        return;
    };
    let qty = app.cio.qty;
    let note = app.cio.note.trim().to_string();
    let user = app.current_user;
    let item_name = app.item_name(item_id);

    let result = match kind {
        TxType::In => queries::check_in(&app.db.conn, item_id, user, qty, &note),
        TxType::Out => queries::check_out(&app.db.conn, item_id, user, qty, &note),
    };

    match result {
        Ok(_) => {
            app.refresh_all();
            app.cio.note.clear();
            app.cio.qty = 1;
            app.set_ok(format!("{} \"{}\" จำนวน {} สำเร็จ", kind.label_th(), item_name, qty));
        }
        Err(e) => app.set_err(format!("{}ไม่สำเร็จ: {}", kind.label_th(), e)),
    }
}
