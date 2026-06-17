//! แท็บ "รับเข้า/เบิกออก" — เลือกของ (หรือสแกนบาร์โค้ด) แล้ว check-in/out

use eframe::egui;

use crate::app::App;
use crate::db::models::TxType;
use crate::db::queries;

pub fn show(app: &mut App, ui: &mut egui::Ui) {
    ui.add_space(6.0);
    ui.heading("รับเข้า / เบิกออก");
    ui.add_space(4.0);

    if app.current_user.is_none() {
        ui.colored_label(
            egui::Color32::from_rgb(200, 120, 0),
            "⚠ ยังไม่ได้เลือกผู้ทำรายการ (เลือกที่มุมบนขวา) — ระบบจะบันทึกโดยไม่ระบุชื่อ",
        );
        ui.add_space(4.0);
    }

    // ── สแกน/ค้นหาด้วยบาร์โค้ด ──
    ui.horizontal(|ui| {
        ui.label("สแกนบาร์โค้ด:");
        let resp = ui.add(
            egui::TextEdit::singleline(&mut app.cio.barcode_input)
                .desired_width(220.0)
                .hint_text("สแกนหรือพิมพ์รหัสแล้วกด Enter"),
        );
        let enter = resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
        if (ui.button("ค้นหา").clicked() || enter) && !app.cio.barcode_input.trim().is_empty() {
            let code = app.cio.barcode_input.trim().to_string();
            match queries::find_by_barcode(&app.db.conn, &code) {
                Ok(Some(item)) => {
                    app.cio.selected_item = Some(item.id);
                    app.cio.barcode_input.clear();
                    app.set_ok(format!("เลือก \"{}\" แล้ว", item.name));
                }
                Ok(None) => app.set_err(format!("ไม่พบของที่มีบาร์โค้ด {}", code)),
                Err(e) => app.set_err(format!("ค้นหาไม่สำเร็จ: {}", e)),
            }
        }
    });

    ui.add_space(4.0);

    // ── เลือกของจากรายการ ──
    ui.horizontal(|ui| {
        ui.label("เลือกของ:");
        let label = match app.cio.selected_item {
            Some(id) => app.item_name(id),
            None => "— เลือกรายการ —".to_string(),
        };
        egui::ComboBox::from_id_salt("cio_item")
            .width(280.0)
            .selected_text(label)
            .show_ui(ui, |ui| {
                for it in &app.items {
                    ui.selectable_value(&mut app.cio.selected_item, Some(it.id), &it.name);
                }
            });
    });

    // ── แสดงจำนวนคงเหลือปัจจุบัน ──
    let current = app
        .cio
        .selected_item
        .and_then(|id| app.items.iter().find(|i| i.id == id))
        .map(|i| (i.quantity, i.unit.clone()));
    if let Some((qty, unit)) = &current {
        ui.label(format!("คงเหลือปัจจุบัน: {} {}", qty, unit));
    }

    ui.add_space(6.0);
    ui.horizontal(|ui| {
        ui.label("จำนวน:");
        ui.add(egui::DragValue::new(&mut app.cio.qty).range(1..=1_000_000));
        ui.label("หมายเหตุ:");
        ui.add(
            egui::TextEdit::singleline(&mut app.cio.note)
                .desired_width(240.0)
                .hint_text("เช่น ซื้อเพิ่ม / หุงข้าว"),
        );
    });

    ui.add_space(8.0);

    let mut action: Option<TxType> = None;
    ui.horizontal(|ui| {
        if ui
            .add_sized([130.0, 32.0], egui::Button::new("⬇ รับเข้า (Check-in)"))
            .clicked()
        {
            action = Some(TxType::In);
        }
        if ui
            .add_sized([130.0, 32.0], egui::Button::new("⬆ เบิกออก (Check-out)"))
            .clicked()
        {
            action = Some(TxType::Out);
        }
    });

    if let Some(kind) = action {
        do_transaction(app, kind);
    }
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
