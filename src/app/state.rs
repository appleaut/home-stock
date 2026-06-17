//! สถานะของ UI (แท็บ, ฟอร์ม, ตัวกรอง) แยกจาก logic ฐานข้อมูล

/// แท็บหลักของแอป
#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Tab {
    Items,
    CheckInOut,
    History,
    Alerts,
    Reports,
}

impl Default for Tab {
    fn default() -> Self {
        Tab::Items
    }
}

/// ข้อความสถานะ (สำเร็จ = เขียว, ผิดพลาด = แดง)
pub struct Status {
    pub text: String,
    pub error: bool,
}

/// ฟอร์มเพิ่ม/แก้ไขรายการของ (เปิดเป็นหน้าต่างซ้อน)
#[derive(Default)]
pub struct ItemForm {
    pub open: bool,
    /// None = เพิ่มรายการใหม่, Some(id) = กำลังแก้ไขรายการนั้น
    pub editing_id: Option<i64>,
    pub name: String,
    pub category_id: Option<i64>,
    pub location_id: Option<i64>,
    pub unit: String,
    pub quantity: i64,
    pub min_quantity: i64,
    pub barcode: String,
    pub expiry_date: String,
}

impl ItemForm {
    /// เตรียมฟอร์มสำหรับเพิ่มรายการใหม่
    pub fn for_new() -> Self {
        ItemForm {
            open: true,
            unit: "ชิ้น".to_string(),
            ..Default::default()
        }
    }
}

/// ตัวกรองในแท็บรายการของ
#[derive(Default)]
pub struct ItemFilter {
    pub search: String,
    pub category_id: Option<i64>,
    pub location_id: Option<i64>,
}

/// ฟอร์มรับเข้า/เบิกออก
pub struct CheckInOutForm {
    pub barcode_input: String,
    pub selected_item: Option<i64>,
    pub qty: i64,
    pub note: String,
}

impl Default for CheckInOutForm {
    fn default() -> Self {
        CheckInOutForm {
            barcode_input: String::new(),
            selected_item: None,
            qty: 1,
            note: String::new(),
        }
    }
}

/// ตัวกรองในแท็บประวัติ
#[derive(Default)]
pub struct HistoryFilter {
    pub item_id: Option<i64>,
    pub user_id: Option<i64>,
}

/// ช่องเพิ่มข้อมูลพื้นฐานแบบเร็ว (ผู้ใช้/หมวด/ตำแหน่ง)
#[derive(Default)]
pub struct QuickAdd {
    pub new_user: String,
    pub new_category: String,
    pub new_location: String,
}
