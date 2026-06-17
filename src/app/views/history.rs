//! แท็บ "ประวัติ" — ตารางการเคลื่อนไหว พร้อมตัวกรองตามของ/ผู้ใช้

use eframe::egui;
use egui_extras::{Column, TableBuilder};

use crate::app::App;
use crate::db::models::TxType;

pub fn show(app: &mut App, ui: &mut egui::Ui) {
    ui.add_space(4.0);

    ui.horizontal(|ui| {
        ui.label("กรองตามของ:");
        let item_label = match app.history_filter.item_id {
            Some(id) => app.item_name(id),
            None => "ทั้งหมด".to_string(),
        };
        egui::ComboBox::from_id_salt("hist_item")
            .width(200.0)
            .selected_text(item_label)
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut app.history_filter.item_id, None, "ทั้งหมด");
                for it in &app.items {
                    ui.selectable_value(&mut app.history_filter.item_id, Some(it.id), &it.name);
                }
            });

        ui.label("ผู้ทำรายการ:");
        let user_label = match app.history_filter.user_id {
            Some(id) => app.user_name(Some(id)),
            None => "ทั้งหมด".to_string(),
        };
        egui::ComboBox::from_id_salt("hist_user")
            .width(140.0)
            .selected_text(user_label)
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut app.history_filter.user_id, None, "ทั้งหมด");
                for u in &app.users {
                    ui.selectable_value(&mut app.history_filter.user_id, Some(u.id), &u.name);
                }
            });
    });

    ui.separator();

    let filter_user_name = app
        .history_filter
        .user_id
        .map(|id| app.user_name(Some(id)));

    let txs: Vec<&crate::db::models::Transaction> = app
        .transactions
        .iter()
        .filter(|t| {
            if let Some(id) = app.history_filter.item_id {
                if t.item_id != id {
                    return false;
                }
            }
            if let Some(name) = &filter_user_name {
                if &t.user_name != name {
                    return false;
                }
            }
            true
        })
        .collect();

    ui.label(format!("ทั้งหมด {} รายการ", txs.len()));

    egui::ScrollArea::both().show(ui, |ui| {
        TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            .column(Column::auto().at_least(150.0)) // เวลา
            .column(Column::auto().at_least(150.0)) // ของ
            .column(Column::auto().at_least(70.0)) // ประเภท
            .column(Column::auto().at_least(60.0)) // จำนวน
            .column(Column::auto().at_least(90.0)) // ผู้ทำรายการ
            .column(Column::remainder().at_least(150.0)) // หมายเหตุ
            .header(22.0, |mut header| {
                for title in ["เวลา", "ของ", "ประเภท", "จำนวน", "ผู้ทำรายการ", "หมายเหตุ"] {
                    header.col(|ui| {
                        ui.strong(title);
                    });
                }
            })
            .body(|mut body| {
                for t in &txs {
                    body.row(22.0, |mut row| {
                        row.col(|ui| {
                            ui.label(&t.timestamp);
                        });
                        row.col(|ui| {
                            ui.label(&t.item_name);
                        });
                        row.col(|ui| {
                            let (color, text) = match t.tx_type {
                                TxType::In => {
                                    (egui::Color32::from_rgb(30, 140, 60), t.tx_type.label_th())
                                }
                                TxType::Out => {
                                    (egui::Color32::from_rgb(190, 90, 0), t.tx_type.label_th())
                                }
                            };
                            ui.colored_label(color, text);
                        });
                        row.col(|ui| {
                            ui.label(t.quantity.to_string());
                        });
                        row.col(|ui| {
                            ui.label(&t.user_name);
                        });
                        row.col(|ui| {
                            ui.label(&t.note);
                        });
                    });
                }
            });
    });
}
