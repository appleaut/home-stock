// ซ่อนหน้าต่าง console สีดำตอนรันแบบ release บน Windows
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod db;

use eframe::egui;

/// คืน path ไฟล์ฐานข้อมูลในโฟลเดอร์ข้อมูลผู้ใช้ (`%APPDATA%\HomeStock\home_stock.db`)
/// เพื่อให้เขียนได้แม้ติดตั้งแอปไว้ใน Program Files (อ่าน/เขียนข้าง .exe ไม่ได้)
fn database_path() -> std::path::PathBuf {
    let base = std::env::var_os("APPDATA")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    let dir = base.join("HomeStock");
    let _ = std::fs::create_dir_all(&dir);
    dir.join("home_stock.db")
}

fn main() -> eframe::Result<()> {
    // เปิด/สร้างฐานข้อมูลในโฟลเดอร์ข้อมูลผู้ใช้
    let db_path = database_path();
    let db = match db::Db::open(&db_path.to_string_lossy()) {
        Ok(db) => db,
        Err(e) => {
            eprintln!("เปิดฐานข้อมูลไม่สำเร็จ: {e}");
            std::process::exit(1);
        }
    };

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1100.0, 720.0])
            .with_min_inner_size([800.0, 500.0])
            .with_title("ระบบจัดการสต๊อกห้องเก็บของ"),
        ..Default::default()
    };

    eframe::run_native(
        "home-stock",
        native_options,
        Box::new(move |cc| {
            app::setup_fonts(&cc.egui_ctx);
            let app = app::App::new(db);
            app::theme::apply(&cc.egui_ctx, app.theme);
            Ok(Box::new(app))
        }),
    )
}
