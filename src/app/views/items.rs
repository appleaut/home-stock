//! แท็บ "รายการของ" — ตาราง + ฟอร์มเพิ่ม/แก้ไข/ลบ

use eframe::egui;
use egui_extras::{Column, TableBuilder};

use super::days_until;
use crate::app::state::ItemForm;
use crate::app::App;
use crate::db::queries::{self, ItemInput};

/// ข้อมูลหนึ่งแถวที่เตรียมไว้แสดงผล (เพื่อเลี่ยงการยืม App ภายใน closure ของตาราง)
struct RowData {
    id: i64,
    name: String,
    category: String,
    location: String,
    quantity: i64,
    unit: String,
    min_quantity: i64,
    barcode: String,
    expiry: String,
    days_until: Option<i64>,
    low: bool,
}

pub fn show(app: &mut App, ui: &mut egui::Ui) {
    ui.add_space(4.0);

    // ── แถบควบคุม: ค้นหา + กรอง + ปุ่มเพิ่ม ──
    ui.horizontal(|ui| {
        ui.label("ค้นหา:");
        ui.add(
            egui::TextEdit::singleline(&mut app.item_filter.search)
                .desired_width(180.0)
                .hint_text("ชื่อ หรือ บาร์โค้ด"),
        );

        ui.label("หมวด:");
        let cat_label = if app.item_filter.category_id.is_none() {
            "ทั้งหมด".to_string()
        } else {
            app.category_name(app.item_filter.category_id)
        };
        egui::ComboBox::from_id_salt("filter_cat")
            .selected_text(cat_label)
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut app.item_filter.category_id, None, "ทั้งหมด");
                for c in &app.categories {
                    ui.selectable_value(&mut app.item_filter.category_id, Some(c.id), &c.name);
                }
            });

        ui.label("ตำแหน่ง:");
        let loc_label = if app.item_filter.location_id.is_none() {
            "ทั้งหมด".to_string()
        } else {
            app.location_name(app.item_filter.location_id)
        };
        egui::ComboBox::from_id_salt("filter_loc")
            .selected_text(loc_label)
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut app.item_filter.location_id, None, "ทั้งหมด");
                for l in &app.locations {
                    ui.selectable_value(&mut app.item_filter.location_id, Some(l.id), &l.name);
                }
            });

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("➕ เพิ่มของ").clicked() {
                let mut form = ItemForm::for_new();
                // ออกบาร์โค้ดตัวเลข 5 หลักให้อัตโนมัติ (แก้ไขเองได้ภายหลัง)
                if let Ok(code) = queries::next_barcode(&app.db.conn) {
                    form.barcode = code;
                }
                app.item_form = form;
            }
        });
    });

    ui.separator();

    // ── เตรียมแถวข้อมูลตามตัวกรอง ──
    let search = app.item_filter.search.trim().to_lowercase();
    let rows: Vec<RowData> = app
        .items
        .iter()
        .filter(|it| {
            if let Some(cid) = app.item_filter.category_id {
                if it.category_id != Some(cid) {
                    return false;
                }
            }
            if let Some(lid) = app.item_filter.location_id {
                if it.location_id != Some(lid) {
                    return false;
                }
            }
            if !search.is_empty() {
                let in_name = it.name.to_lowercase().contains(&search);
                let in_barcode = it
                    .barcode
                    .as_deref()
                    .map(|b| b.to_lowercase().contains(&search))
                    .unwrap_or(false);
                if !in_name && !in_barcode {
                    return false;
                }
            }
            true
        })
        .map(|it| RowData {
            id: it.id,
            name: it.name.clone(),
            category: app.category_name(it.category_id),
            location: app.location_name(it.location_id),
            quantity: it.quantity,
            unit: it.unit.clone(),
            min_quantity: it.min_quantity,
            barcode: it.barcode.clone().unwrap_or_default(),
            expiry: it.expiry_date.clone().unwrap_or_default(),
            days_until: days_until(&it.expiry_date),
            low: it.quantity <= it.min_quantity,
        })
        .collect();

    ui.label(format!("ทั้งหมด {} รายการ", rows.len()));

    let mut want_edit: Option<i64> = None;
    let mut want_delete: Option<i64> = None;

    // ── ตาราง ──
    egui::ScrollArea::horizontal().show(ui, |ui| {
        TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            .column(Column::auto().at_least(140.0)) // ชื่อ
            .column(Column::auto().at_least(80.0)) // หมวด
            .column(Column::auto().at_least(90.0)) // ตำแหน่ง
            .column(Column::auto().at_least(70.0)) // คงเหลือ
            .column(Column::auto().at_least(50.0)) // หน่วย
            .column(Column::auto().at_least(60.0)) // ขั้นต่ำ
            .column(Column::auto().at_least(90.0)) // บาร์โค้ด
            .column(Column::auto().at_least(100.0)) // หมดอายุ
            .column(Column::remainder().at_least(160.0)) // จัดการ (ปุ่มแก้ไข+ลบ เรียงแนวนอน)
            .header(22.0, |mut header| {
                for title in [
                    "ชื่อ", "หมวด", "ตำแหน่ง", "คงเหลือ", "หน่วย", "ขั้นต่ำ", "บาร์โค้ด",
                    "วันหมดอายุ", "จัดการ",
                ] {
                    header.col(|ui| {
                        ui.strong(title);
                    });
                }
            })
            .body(|mut body| {
                for r in &rows {
                    body.row(28.0, |mut row| {
                        row.col(|ui| {
                            ui.label(&r.name);
                        });
                        row.col(|ui| {
                            ui.label(&r.category);
                        });
                        row.col(|ui| {
                            ui.label(&r.location);
                        });
                        row.col(|ui| {
                            // ของใกล้หมด = สีส้ม
                            if r.low {
                                ui.colored_label(
                                    egui::Color32::from_rgb(220, 120, 0),
                                    r.quantity.to_string(),
                                );
                            } else {
                                ui.label(r.quantity.to_string());
                            }
                        });
                        row.col(|ui| {
                            ui.label(&r.unit);
                        });
                        row.col(|ui| {
                            ui.label(r.min_quantity.to_string());
                        });
                        row.col(|ui| {
                            ui.label(&r.barcode);
                        });
                        row.col(|ui| {
                            // ใกล้/หมดอายุ = สีแดง
                            match r.days_until {
                                Some(d) if d < 0 => ui.colored_label(
                                    egui::Color32::from_rgb(200, 40, 40),
                                    format!("{} (หมดอายุแล้ว)", r.expiry),
                                ),
                                Some(d) if d <= 30 => ui.colored_label(
                                    egui::Color32::from_rgb(200, 40, 40),
                                    format!("{} (อีก {} วัน)", r.expiry, d),
                                ),
                                _ => ui.label(&r.expiry),
                            };
                        });
                        row.col(|ui| {
                            // ปุ่มแก้ไข/ลบ อยู่แนวนอนเดียวกัน
                            ui.horizontal(|ui| {
                                if ui.button("✏ แก้ไข").clicked() {
                                    want_edit = Some(r.id);
                                }
                                if ui.button("🗑 ลบ").clicked() {
                                    want_delete = Some(r.id);
                                }
                            });
                        });
                    });
                }
            });
    });

    // ── จัดการ action จากปุ่มในตาราง (หลังตารางเพื่อเลี่ยงการยืมซ้อน) ──
    if let Some(id) = want_edit {
        if let Some(it) = app.items.iter().find(|i| i.id == id) {
            app.item_form = ItemForm {
                open: true,
                editing_id: Some(it.id),
                name: it.name.clone(),
                category_id: it.category_id,
                location_id: it.location_id,
                unit: it.unit.clone(),
                quantity: it.quantity,
                min_quantity: it.min_quantity,
                barcode: it.barcode.clone().unwrap_or_default(),
                expiry_date: it.expiry_date.clone().unwrap_or_default(),
            };
        }
    }
    if let Some(id) = want_delete {
        app.confirm_delete = Some(id);
    }

    item_form_window(app, ui.ctx());
    delete_confirm_window(app, ui.ctx());
}

/// หน้าต่างฟอร์มเพิ่ม/แก้ไขรายการของ
fn item_form_window(app: &mut App, ctx: &egui::Context) {
    if !app.item_form.open {
        return;
    }
    let title = if app.item_form.editing_id.is_some() {
        "แก้ไขรายการของ"
    } else {
        "เพิ่มรายการของ"
    };

    let mut open = true;
    let mut do_save = false;
    let mut do_cancel = false;

    egui::Window::new(title)
        .open(&mut open)
        .collapsible(false)
        .resizable(false)
        .show(ctx, |ui| {
            egui::Grid::new("item_form_grid")
                .num_columns(2)
                .spacing([12.0, 8.0])
                .show(ui, |ui| {
                    ui.label("ชื่อ *");
                    ui.text_edit_singleline(&mut app.item_form.name);
                    ui.end_row();

                    ui.label("หมวดหมู่");
                    ui.horizontal(|ui| {
                        let label = if app.item_form.category_id.is_none() {
                            "— ไม่ระบุ —".to_string()
                        } else {
                            app.category_name(app.item_form.category_id)
                        };
                        egui::ComboBox::from_id_salt("form_cat")
                            .selected_text(label)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut app.item_form.category_id, None, "— ไม่ระบุ —");
                                for c in &app.categories {
                                    ui.selectable_value(
                                        &mut app.item_form.category_id,
                                        Some(c.id),
                                        &c.name,
                                    );
                                }
                            });
                        ui.add(
                            egui::TextEdit::singleline(&mut app.quick.new_category)
                                .desired_width(90.0)
                                .hint_text("เพิ่มหมวด"),
                        );
                        if ui.button("+").clicked() && !app.quick.new_category.trim().is_empty() {
                            let name = app.quick.new_category.trim().to_string();
                            if let Ok(id) = queries::add_category(&app.db.conn, &name) {
                                app.quick.new_category.clear();
                                app.refresh_all();
                                app.item_form.category_id = Some(id);
                            }
                        }
                    });
                    ui.end_row();

                    ui.label("ตำแหน่งเก็บ");
                    ui.horizontal(|ui| {
                        let label = if app.item_form.location_id.is_none() {
                            "— ไม่ระบุ —".to_string()
                        } else {
                            app.location_name(app.item_form.location_id)
                        };
                        egui::ComboBox::from_id_salt("form_loc")
                            .selected_text(label)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut app.item_form.location_id, None, "— ไม่ระบุ —");
                                for l in &app.locations {
                                    ui.selectable_value(
                                        &mut app.item_form.location_id,
                                        Some(l.id),
                                        &l.name,
                                    );
                                }
                            });
                        ui.add(
                            egui::TextEdit::singleline(&mut app.quick.new_location)
                                .desired_width(90.0)
                                .hint_text("เพิ่มตำแหน่ง"),
                        );
                        if ui.button("+").clicked() && !app.quick.new_location.trim().is_empty() {
                            let name = app.quick.new_location.trim().to_string();
                            if let Ok(id) = queries::add_location(&app.db.conn, &name) {
                                app.quick.new_location.clear();
                                app.refresh_all();
                                app.item_form.location_id = Some(id);
                            }
                        }
                    });
                    ui.end_row();

                    ui.label("หน่วยนับ");
                    ui.text_edit_singleline(&mut app.item_form.unit);
                    ui.end_row();

                    ui.label("จำนวนคงเหลือ");
                    ui.add(egui::DragValue::new(&mut app.item_form.quantity).range(0..=1_000_000));
                    ui.end_row();

                    ui.label("จำนวนขั้นต่ำ (เตือน)");
                    ui.add(egui::DragValue::new(&mut app.item_form.min_quantity).range(0..=1_000_000));
                    ui.end_row();

                    ui.label("บาร์โค้ด");
                    ui.horizontal(|ui| {
                        ui.text_edit_singleline(&mut app.item_form.barcode);
                        // ปุ่มออกรหัสใหม่ (เฉพาะตอนเพิ่มของใหม่)
                        if app.item_form.editing_id.is_none()
                            && ui
                                .button("🔄")
                                .on_hover_text("ออกบาร์โค้ด 5 หลักใหม่")
                                .clicked()
                        {
                            if let Ok(code) = queries::next_barcode(&app.db.conn) {
                                app.item_form.barcode = code;
                            }
                        }
                    });
                    ui.end_row();

                    ui.label("วันหมดอายุ");
                    ui.add(
                        egui::TextEdit::singleline(&mut app.item_form.expiry_date)
                            .hint_text("YYYY-MM-DD (เว้นว่างได้)"),
                    );
                    ui.end_row();
                });

            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("💾 บันทึก").clicked() {
                    do_save = true;
                }
                if ui.button("ยกเลิก").clicked() {
                    do_cancel = true;
                }
            });
        });

    if do_save {
        save_item_form(app);
    }
    if do_cancel || !open {
        app.item_form.open = false;
    }
}

/// ตรวจและบันทึกฟอร์ม
fn save_item_form(app: &mut App) {
    let f = &app.item_form;
    if f.name.trim().is_empty() {
        app.set_err("ต้องระบุชื่อของ");
        return;
    }
    // ตรวจรูปแบบวันหมดอายุ
    let expiry = f.expiry_date.trim();
    if !expiry.is_empty() && super::parse_date(expiry).is_none() {
        app.set_err("รูปแบบวันหมดอายุไม่ถูกต้อง (ต้องเป็น YYYY-MM-DD)");
        return;
    }

    let input = ItemInput {
        name: f.name.trim().to_string(),
        category_id: f.category_id,
        location_id: f.location_id,
        unit: if f.unit.trim().is_empty() {
            "ชิ้น".to_string()
        } else {
            f.unit.trim().to_string()
        },
        quantity: f.quantity,
        min_quantity: f.min_quantity,
        barcode: {
            let b = f.barcode.trim();
            if b.is_empty() {
                None
            } else {
                Some(b.to_string())
            }
        },
        expiry_date: if expiry.is_empty() {
            None
        } else {
            Some(expiry.to_string())
        },
    };

    let editing_id = f.editing_id;
    let result = match editing_id {
        Some(id) => queries::update_item(&app.db.conn, id, &input).map(|_| "แก้ไขรายการแล้ว"),
        None => queries::add_item(&app.db.conn, &input).map(|_| "เพิ่มรายการแล้ว"),
    };
    match result {
        Ok(msg) => {
            app.item_form.open = false;
            app.refresh_all();
            app.set_ok(msg);
        }
        Err(e) => app.set_err(format!("บันทึกไม่สำเร็จ: {}", e)),
    }
}

/// หน้าต่างยืนยันการลบ
fn delete_confirm_window(app: &mut App, ctx: &egui::Context) {
    let Some(id) = app.confirm_delete else {
        return;
    };
    let name = app.item_name(id);
    let mut do_delete = false;
    let mut do_cancel = false;

    egui::Window::new("ยืนยันการลบ")
        .collapsible(false)
        .resizable(false)
        .show(ctx, |ui| {
            ui.label(format!(
                "ต้องการลบ \"{}\" และประวัติทั้งหมดของรายการนี้หรือไม่?",
                name
            ));
            ui.horizontal(|ui| {
                if ui.button("ลบ").clicked() {
                    do_delete = true;
                }
                if ui.button("ยกเลิก").clicked() {
                    do_cancel = true;
                }
            });
        });

    if do_delete {
        match queries::delete_item(&app.db.conn, id) {
            Ok(_) => {
                app.refresh_all();
                app.set_ok(format!("ลบ \"{}\" แล้ว", name));
            }
            Err(e) => app.set_err(format!("ลบไม่สำเร็จ: {}", e)),
        }
        app.confirm_delete = None;
    }
    if do_cancel {
        app.confirm_delete = None;
    }
}
