//! โครงสร้างข้อมูล (struct) ที่ใช้แทนแถวในตารางต่าง ๆ ของฐานข้อมูล

/// ผู้ทำรายการ (สมาชิกในบ้าน)
#[derive(Clone, Debug)]
pub struct User {
    pub id: i64,
    pub name: String,
}

/// หมวดหมู่ของของ เช่น ของกิน, ของใช้, ยา
#[derive(Clone, Debug)]
pub struct Category {
    pub id: i64,
    pub name: String,
}

/// ตำแหน่งที่เก็บของในห้อง
#[derive(Clone, Debug)]
pub struct Location {
    pub id: i64,
    pub name: String,
}

/// รายการของในสต๊อก
#[derive(Clone, Debug)]
pub struct Item {
    pub id: i64,
    pub name: String,
    pub category_id: Option<i64>,
    pub location_id: Option<i64>,
    pub unit: String,
    pub quantity: i64,
    pub min_quantity: i64,
    pub barcode: Option<String>,
    /// วันหมดอายุรูปแบบ ISO `YYYY-MM-DD` (None = ไม่มีวันหมดอายุ)
    pub expiry_date: Option<String>,
    // เก็บไว้บันทึก/ตรวจสอบ แต่ยังไม่แสดงบน UI
    #[allow(dead_code)]
    pub created_at: String,
    #[allow(dead_code)]
    pub updated_at: String,
}

/// ประเภทการเคลื่อนไหวสต๊อก
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TxType {
    /// รับของเข้า
    In,
    /// เบิกของออก
    Out,
}

impl TxType {
    pub fn from_db(s: &str) -> TxType {
        match s {
            "OUT" => TxType::Out,
            _ => TxType::In,
        }
    }

    /// ป้ายภาษาไทยสำหรับแสดงผล
    pub fn label_th(self) -> &'static str {
        match self {
            TxType::In => "รับเข้า",
            TxType::Out => "เบิกออก",
        }
    }
}

/// หนึ่งแถวประวัติการเคลื่อนไหว (join ชื่อของและชื่อผู้ใช้มาแล้วเพื่อแสดงผล)
#[derive(Clone, Debug)]
pub struct Transaction {
    #[allow(dead_code)]
    pub id: i64,
    pub item_id: i64,
    pub item_name: String,
    pub user_name: String,
    pub tx_type: TxType,
    pub quantity: i64,
    pub note: String,
    pub timestamp: String,
}
