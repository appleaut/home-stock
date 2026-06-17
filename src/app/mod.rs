//! ชั้น UI: โครงแอป (eframe::App), แถบบน, เมนูแท็บ และตัวช่วยร่วม

pub mod state;
pub mod theme;
pub mod views;

use eframe::egui;
use std::sync::Arc;

use crate::db::models::{Category, Item, Location, Transaction, User};
use crate::db::{queries, Db};
use state::{CheckInOutForm, HistoryFilter, ItemFilter, ItemForm, QuickAdd, Status, Tab};
use theme::Theme;

const SETTING_THEME: &str = "theme";

/// โหลดฟอนต์ไทย (Sarabun) ฝังในไบนารี แล้วตั้งเป็นฟอนต์หลัก
pub fn setup_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "sarabun".to_owned(),
        Arc::new(egui::FontData::from_static(include_bytes!(
            "../../assets/fonts/Sarabun-Regular.ttf"
        ))),
    );
    // ใช้ Sarabun เป็นอันดับแรกทั้งตัวอักษรปกติและ monospace
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "sarabun".to_owned());
    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .insert(0, "sarabun".to_owned());
    ctx.set_fonts(fonts);
}

/// สถานะแอปทั้งหมด
pub struct App {
    pub db: Db,
    pub tab: Tab,
    pub theme: Theme,
    pub current_user: Option<i64>,

    // แคชข้อมูลจากฐานข้อมูล (รีเฟรชหลังการกระทำที่เปลี่ยนข้อมูล)
    pub items: Vec<Item>,
    pub categories: Vec<Category>,
    pub locations: Vec<Location>,
    pub users: Vec<User>,
    pub transactions: Vec<Transaction>,

    // สถานะฟอร์ม/ตัวกรอง
    pub item_form: ItemForm,
    pub item_filter: ItemFilter,
    pub cio: CheckInOutForm,
    pub history_filter: HistoryFilter,
    pub alert_days: i64,
    pub quick: QuickAdd,

    pub status: Option<Status>,
    /// id ของรายการที่รอการยืนยันก่อนลบ
    pub confirm_delete: Option<i64>,
}

impl App {
    pub fn new(db: Db) -> Self {
        // โหลดธีมที่บันทึกไว้ (ค่าเริ่มต้น = สว่าง)
        let theme = queries::get_setting(&db.conn, SETTING_THEME)
            .ok()
            .flatten()
            .map(|s| Theme::from_str(&s))
            .unwrap_or(Theme::Light);

        let mut app = App {
            db,
            tab: Tab::default(),
            theme,
            current_user: None,
            items: Vec::new(),
            categories: Vec::new(),
            locations: Vec::new(),
            users: Vec::new(),
            transactions: Vec::new(),
            item_form: ItemForm::default(),
            item_filter: ItemFilter::default(),
            cio: CheckInOutForm::default(),
            history_filter: HistoryFilter::default(),
            alert_days: 30,
            quick: QuickAdd::default(),
            status: None,
            confirm_delete: None,
        };
        app.refresh_all();
        if app.current_user.is_none() {
            app.current_user = app.users.first().map(|u| u.id);
        }
        app
    }

    /// โหลดข้อมูลทั้งหมดจากฐานข้อมูลใหม่
    pub fn refresh_all(&mut self) {
        match (
            queries::list_items(&self.db.conn),
            queries::list_categories(&self.db.conn),
            queries::list_locations(&self.db.conn),
            queries::list_users(&self.db.conn),
            queries::list_transactions(&self.db.conn),
        ) {
            (Ok(i), Ok(c), Ok(l), Ok(u), Ok(t)) => {
                self.items = i;
                self.categories = c;
                self.locations = l;
                self.users = u;
                self.transactions = t;
            }
            _ => self.set_err("โหลดข้อมูลจากฐานข้อมูลไม่สำเร็จ"),
        }
    }

    pub fn set_ok(&mut self, text: impl Into<String>) {
        self.status = Some(Status {
            text: text.into(),
            error: false,
        });
    }

    pub fn set_err(&mut self, text: impl Into<String>) {
        self.status = Some(Status {
            text: text.into(),
            error: true,
        });
    }

    // ─── ตัวช่วยค้นชื่อจาก id ───
    pub fn category_name(&self, id: Option<i64>) -> String {
        id.and_then(|id| self.categories.iter().find(|c| c.id == id))
            .map(|c| c.name.clone())
            .unwrap_or_default()
    }

    pub fn location_name(&self, id: Option<i64>) -> String {
        id.and_then(|id| self.locations.iter().find(|l| l.id == id))
            .map(|l| l.name.clone())
            .unwrap_or_default()
    }

    pub fn user_name(&self, id: Option<i64>) -> String {
        id.and_then(|id| self.users.iter().find(|u| u.id == id))
            .map(|u| u.name.clone())
            .unwrap_or_default()
    }

    pub fn item_name(&self, id: i64) -> String {
        self.items
            .iter()
            .find(|i| i.id == id)
            .map(|i| i.name.clone())
            .unwrap_or_default()
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::top("top_bar").show_inside(ui, |ui| {
            self.top_bar(ui);
        });

        if let Some(status) = &self.status {
            let color = if status.error {
                egui::Color32::from_rgb(200, 40, 40)
            } else {
                egui::Color32::from_rgb(30, 140, 60)
            };
            let text = status.text.clone();
            egui::Panel::bottom("status_bar").show_inside(ui, |ui| {
                ui.colored_label(color, text);
            });
        }

        egui::Panel::left("nav")
            .resizable(false)
            .exact_size(196.0)
            .show_inside(ui, |ui| {
                self.nav_panel(ui);
            });

        egui::CentralPanel::default().show_inside(ui, |ui| match self.tab {
            Tab::Items => views::items::show(self, ui),
            Tab::CheckInOut => views::checkinout::show(self, ui),
            Tab::History => views::history::show(self, ui),
            Tab::Alerts => views::alerts::show(self, ui),
            Tab::Reports => views::reports::show(self, ui),
        });
    }
}

impl App {
    /// สลับธีมแล้วบันทึกลงฐานข้อมูล
    fn toggle_theme(&mut self, ctx: &egui::Context) {
        self.theme = self.theme.toggled();
        theme::apply(ctx, self.theme);
        let _ = queries::set_setting(&self.db.conn, SETTING_THEME, self.theme.as_str());
    }

    /// แถบบน: ชื่อแอป + ปุ่มสลับธีม + เลือกผู้ทำรายการ
    fn top_bar(&mut self, ui: &mut egui::Ui) {
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.heading("📦 ระบบจัดการสต๊อกห้องเก็บของ");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // ปุ่มสลับธีม Dark/Light
                if ui.button(self.theme.toggle_label()).clicked() {
                    let ctx = ui.ctx().clone();
                    self.toggle_theme(&ctx);
                }
                ui.separator();

                // เพิ่มผู้ใช้ใหม่อย่างเร็ว
                if ui.button("➕").on_hover_text("เพิ่มผู้ใช้").clicked()
                    && !self.quick.new_user.trim().is_empty()
                {
                    let name = self.quick.new_user.trim().to_string();
                    match queries::add_user(&self.db.conn, &name) {
                        Ok(id) => {
                            self.quick.new_user.clear();
                            self.refresh_all();
                            self.current_user = Some(id);
                            self.set_ok(format!("เพิ่มผู้ใช้ \"{}\" แล้ว", name));
                        }
                        Err(e) => self.set_err(format!("เพิ่มผู้ใช้ไม่สำเร็จ: {}", e)),
                    }
                }
                ui.add(
                    egui::TextEdit::singleline(&mut self.quick.new_user)
                        .desired_width(110.0)
                        .hint_text("ชื่อผู้ใช้ใหม่"),
                );

                let selected = self.user_name(self.current_user);
                let selected_label = if selected.is_empty() {
                    "— เลือกผู้ทำรายการ —".to_string()
                } else {
                    selected
                };
                egui::ComboBox::from_id_salt("user_select")
                    .selected_text(selected_label)
                    .show_ui(ui, |ui| {
                        for u in &self.users {
                            ui.selectable_value(
                                &mut self.current_user,
                                Some(u.id),
                                &u.name,
                            );
                        }
                    });
                ui.label("ผู้ทำรายการ:");
            });
        });
        ui.add_space(6.0);
    }

    /// แถบนำทางด้านซ้าย (navigation rail สไตล์ Windows 11)
    fn nav_panel(&mut self, ui: &mut egui::Ui) {
        ui.add_space(10.0);
        let items = [
            (Tab::Items, "📋  รายการของ"),
            (Tab::CheckInOut, "🔄  รับเข้า/เบิกออก"),
            (Tab::History, "🕘  ประวัติ"),
            (Tab::Alerts, "⚠  แจ้งเตือน"),
            (Tab::Reports, "📊  รายงาน"),
        ];
        for (tab, label) in items {
            let selected = self.tab == tab;
            let resp = ui.add_sized(
                [ui.available_width(), 38.0],
                egui::Button::selectable(selected, label),
            );
            if resp.clicked() {
                self.tab = tab;
            }
            ui.add_space(2.0);
        }
    }
}
