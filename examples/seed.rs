//! ตัวช่วย seed ข้อมูลตัวอย่างลง home_stock.db (สำหรับ dev/ทดสอบภาพหน้าจอ)
//! รัน: cargo run --example seed -- [light|dark]

use rusqlite::{params, Connection};

const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS users (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT NOT NULL UNIQUE);
CREATE TABLE IF NOT EXISTS categories (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT NOT NULL UNIQUE);
CREATE TABLE IF NOT EXISTS locations (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT NOT NULL UNIQUE);
CREATE TABLE IF NOT EXISTS items (
    id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT NOT NULL,
    category_id INTEGER, location_id INTEGER, unit TEXT NOT NULL DEFAULT 'ชิ้น',
    quantity INTEGER NOT NULL DEFAULT 0, min_quantity INTEGER NOT NULL DEFAULT 0,
    barcode TEXT, expiry_date TEXT, created_at TEXT NOT NULL, updated_at TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS transactions (
    id INTEGER PRIMARY KEY AUTOINCREMENT, item_id INTEGER NOT NULL, user_id INTEGER,
    type TEXT NOT NULL, quantity INTEGER NOT NULL, note TEXT NOT NULL DEFAULT '', timestamp TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS settings (key TEXT PRIMARY KEY, value TEXT NOT NULL);
"#;

fn main() {
    let theme = std::env::args().nth(1).unwrap_or_else(|| "light".to_string());
    let conn = Connection::open("home_stock.db").unwrap();
    conn.execute_batch(SCHEMA).unwrap();

    // ล้างข้อมูลเดิมให้ seed ซ้ำได้ + รีเซ็ตลำดับ AUTOINCREMENT ให้ id เริ่มที่ 1 เหมือนเดิม
    for t in ["transactions", "items", "categories", "locations", "users", "settings"] {
        conn.execute(&format!("DELETE FROM {t}"), []).unwrap();
    }
    let _ = conn.execute("DELETE FROM sqlite_sequence", []);

    let now = "2026-06-17 10:00:00";
    for u in ["แม่", "พ่อ", "ลูก"] {
        conn.execute("INSERT INTO users(name) VALUES(?1)", params![u]).unwrap();
    }
    for c in ["ของกิน", "ของใช้", "ยา"] {
        conn.execute("INSERT INTO categories(name) VALUES(?1)", params![c]).unwrap();
    }
    for l in ["ชั้นบนซ้าย", "ลิ้นชัก A", "ตู้เย็น"] {
        conn.execute("INSERT INTO locations(name) VALUES(?1)", params![l]).unwrap();
    }

    // (name, cat, loc, unit, qty, min, barcode, expiry)
    let items: &[(&str, i64, i64, &str, i64, i64, &str, &str)] = &[
        ("ข้าวสารหอมมะลิ", 1, 1, "ถุง", 8, 3, "8850001", ""),
        ("น้ำปลา", 1, 2, "ขวด", 2, 3, "8850002", "2026-12-31"),
        ("นมกล่อง UHT", 1, 3, "กล่อง", 12, 6, "8850003", "2026-06-25"),
        ("ผงซักฟอก", 2, 1, "ถุง", 1, 2, "8850004", ""),
        ("พาราเซตามอล", 3, 2, "แผง", 5, 2, "8850005", "2026-07-10"),
        ("ปลากระป๋อง", 1, 1, "กระป๋อง", 20, 8, "8850006", "2027-03-01"),
    ];
    for (name, cat, loc, unit, qty, min, bc, exp) in items {
        let expiry: Option<&str> = if exp.is_empty() { None } else { Some(*exp) };
        conn.execute(
            "INSERT INTO items(name,category_id,location_id,unit,quantity,min_quantity,barcode,expiry_date,created_at,updated_at) \
             VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?9)",
            params![name, cat, loc, unit, qty, min, bc, expiry, now],
        ).unwrap();
    }

    // ประวัติตัวอย่าง
    conn.execute("INSERT INTO transactions(item_id,user_id,type,quantity,note,timestamp) VALUES(1,1,'IN',10,'ซื้อเข้าบ้าน','2026-06-15 09:00:00')", []).unwrap();
    conn.execute("INSERT INTO transactions(item_id,user_id,type,quantity,note,timestamp) VALUES(1,2,'OUT',2,'หุงข้าว','2026-06-16 18:30:00')", []).unwrap();
    conn.execute("INSERT INTO transactions(item_id,user_id,type,quantity,note,timestamp) VALUES(3,3,'IN',12,'ซื้อโปรโมชัน','2026-06-17 08:00:00')", []).unwrap();

    conn.execute(
        "INSERT INTO settings(key,value) VALUES('theme',?1) ON CONFLICT(key) DO UPDATE SET value=excluded.value",
        params![theme],
    ).unwrap();

    println!("seeded sample data (theme={theme})");
}
