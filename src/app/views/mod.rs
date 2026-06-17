//! โค้ดวาดแต่ละแท็บ + ตัวช่วยร่วมเกี่ยวกับวันที่

pub mod alerts;
pub mod checkinout;
pub mod history;
pub mod items;
pub mod reports;

use chrono::NaiveDate;

/// แปลงสตริงวันที่รูปแบบ `YYYY-MM-DD`
pub fn parse_date(s: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(s.trim(), "%Y-%m-%d").ok()
}

/// จำนวนวันนับจากวันนี้ถึงวันหมดอายุ (ค่าลบ = หมดอายุไปแล้ว)
/// คืน None ถ้าไม่มีวันหมดอายุหรือรูปแบบไม่ถูกต้อง
pub fn days_until(expiry: &Option<String>) -> Option<i64> {
    let s = expiry.as_ref()?;
    if s.trim().is_empty() {
        return None;
    }
    let d = parse_date(s)?;
    let today = chrono::Local::now().date_naive();
    Some((d - today).num_days())
}
