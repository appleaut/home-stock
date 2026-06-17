//! ฟังก์ชัน query ทั้งหมด: CRUD, check-in/out (atomic), และ query สำหรับแจ้งเตือน/รายงาน

use anyhow::{bail, Result};
use rusqlite::{params, Connection, Row};

use super::models::{Category, Item, Location, Transaction, TxType, User};

/// เวลาปัจจุบันรูปแบบ ISO `YYYY-MM-DD HH:MM:SS`
fn now_iso() -> String {
    chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

// ───────────────────────────────── settings (key/value) ─────────────────────────────────

pub fn get_setting(conn: &Connection, key: &str) -> Result<Option<String>> {
    let mut stmt = conn.prepare("SELECT value FROM settings WHERE key = ?1")?;
    let mut rows = stmt.query_map(params![key], |r| r.get::<_, String>(0))?;
    match rows.next() {
        Some(v) => Ok(Some(v?)),
        None => Ok(None),
    }
}

pub fn set_setting(conn: &Connection, key: &str, value: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2) \
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )?;
    Ok(())
}

// ───────────────────────── users / categories / locations ─────────────────────────

pub fn list_users(conn: &Connection) -> Result<Vec<User>> {
    let mut stmt = conn.prepare("SELECT id, name FROM users ORDER BY name")?;
    let rows = stmt
        .query_map([], |r| {
            Ok(User {
                id: r.get(0)?,
                name: r.get(1)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

pub fn add_user(conn: &Connection, name: &str) -> Result<i64> {
    conn.execute("INSERT INTO users (name) VALUES (?1)", params![name])?;
    Ok(conn.last_insert_rowid())
}

pub fn list_categories(conn: &Connection) -> Result<Vec<Category>> {
    let mut stmt = conn.prepare("SELECT id, name FROM categories ORDER BY name")?;
    let rows = stmt
        .query_map([], |r| {
            Ok(Category {
                id: r.get(0)?,
                name: r.get(1)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

pub fn add_category(conn: &Connection, name: &str) -> Result<i64> {
    conn.execute("INSERT INTO categories (name) VALUES (?1)", params![name])?;
    Ok(conn.last_insert_rowid())
}

pub fn list_locations(conn: &Connection) -> Result<Vec<Location>> {
    let mut stmt = conn.prepare("SELECT id, name FROM locations ORDER BY name")?;
    let rows = stmt
        .query_map([], |r| {
            Ok(Location {
                id: r.get(0)?,
                name: r.get(1)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

pub fn add_location(conn: &Connection, name: &str) -> Result<i64> {
    conn.execute("INSERT INTO locations (name) VALUES (?1)", params![name])?;
    Ok(conn.last_insert_rowid())
}

// ───────────────────────────────── items ─────────────────────────────────

fn row_to_item(r: &Row) -> rusqlite::Result<Item> {
    Ok(Item {
        id: r.get("id")?,
        name: r.get("name")?,
        category_id: r.get("category_id")?,
        location_id: r.get("location_id")?,
        unit: r.get("unit")?,
        quantity: r.get("quantity")?,
        min_quantity: r.get("min_quantity")?,
        barcode: r.get("barcode")?,
        expiry_date: r.get("expiry_date")?,
        created_at: r.get("created_at")?,
        updated_at: r.get("updated_at")?,
    })
}

pub fn list_items(conn: &Connection) -> Result<Vec<Item>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, category_id, location_id, unit, quantity, min_quantity, \
         barcode, expiry_date, created_at, updated_at FROM items ORDER BY name",
    )?;
    let rows = stmt
        .query_map([], row_to_item)?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

/// ค้นหาของจากบาร์โค้ด (ใช้ตอนสแกนเพื่อ check-in/out)
pub fn find_by_barcode(conn: &Connection, barcode: &str) -> Result<Option<Item>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, category_id, location_id, unit, quantity, min_quantity, \
         barcode, expiry_date, created_at, updated_at FROM items WHERE barcode = ?1 LIMIT 1",
    )?;
    let mut rows = stmt.query_map(params![barcode], row_to_item)?;
    match rows.next() {
        Some(item) => Ok(Some(item?)),
        None => Ok(None),
    }
}

/// อาร์กิวเมนต์สำหรับเพิ่ม/แก้ไขรายการของ
pub struct ItemInput {
    pub name: String,
    pub category_id: Option<i64>,
    pub location_id: Option<i64>,
    pub unit: String,
    pub quantity: i64,
    pub min_quantity: i64,
    pub barcode: Option<String>,
    pub expiry_date: Option<String>,
}

pub fn add_item(conn: &Connection, input: &ItemInput) -> Result<i64> {
    if input.name.trim().is_empty() {
        bail!("ต้องระบุชื่อของ");
    }
    let now = now_iso();
    conn.execute(
        "INSERT INTO items \
         (name, category_id, location_id, unit, quantity, min_quantity, barcode, expiry_date, created_at, updated_at) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?9)",
        params![
            input.name.trim(),
            input.category_id,
            input.location_id,
            input.unit,
            input.quantity,
            input.min_quantity,
            input.barcode,
            input.expiry_date,
            now,
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn update_item(conn: &Connection, id: i64, input: &ItemInput) -> Result<()> {
    if input.name.trim().is_empty() {
        bail!("ต้องระบุชื่อของ");
    }
    conn.execute(
        "UPDATE items SET name=?1, category_id=?2, location_id=?3, unit=?4, \
         quantity=?5, min_quantity=?6, barcode=?7, expiry_date=?8, updated_at=?9 WHERE id=?10",
        params![
            input.name.trim(),
            input.category_id,
            input.location_id,
            input.unit,
            input.quantity,
            input.min_quantity,
            input.barcode,
            input.expiry_date,
            now_iso(),
            id,
        ],
    )?;
    Ok(())
}

pub fn delete_item(conn: &Connection, id: i64) -> Result<()> {
    // ลบประวัติของรายการนี้ก่อน (เคารพ foreign key) แล้วจึงลบตัวรายการ
    conn.execute("DELETE FROM transactions WHERE item_id = ?1", params![id])?;
    conn.execute("DELETE FROM items WHERE id = ?1", params![id])?;
    Ok(())
}

// ──────────────────────────── check-in / check-out ────────────────────────────

/// รับของเข้า: เพิ่มจำนวน + บันทึกประวัติ (atomic)
pub fn check_in(
    conn: &Connection,
    item_id: i64,
    user_id: Option<i64>,
    qty: i64,
    note: &str,
) -> Result<()> {
    if qty <= 0 {
        bail!("จำนวนต้องมากกว่า 0");
    }
    let tx = conn.unchecked_transaction()?;
    let now = now_iso();
    tx.execute(
        "UPDATE items SET quantity = quantity + ?1, updated_at = ?2 WHERE id = ?3",
        params![qty, now, item_id],
    )?;
    tx.execute(
        "INSERT INTO transactions (item_id, user_id, type, quantity, note, timestamp) \
         VALUES (?1, ?2, 'IN', ?3, ?4, ?5)",
        params![item_id, user_id, qty, note, now],
    )?;
    tx.commit()?;
    Ok(())
}

/// เบิกของออก: ตรวจจำนวนพอ → ลดจำนวน + บันทึกประวัติ (atomic, กันติดลบ)
pub fn check_out(
    conn: &Connection,
    item_id: i64,
    user_id: Option<i64>,
    qty: i64,
    note: &str,
) -> Result<()> {
    if qty <= 0 {
        bail!("จำนวนต้องมากกว่า 0");
    }
    let tx = conn.unchecked_transaction()?;
    let current: i64 = tx.query_row(
        "SELECT quantity FROM items WHERE id = ?1",
        params![item_id],
        |r| r.get(0),
    )?;
    if current < qty {
        bail!("จำนวนคงเหลือไม่พอ (เหลือ {})", current);
    }
    let now = now_iso();
    tx.execute(
        "UPDATE items SET quantity = quantity - ?1, updated_at = ?2 WHERE id = ?3",
        params![qty, now, item_id],
    )?;
    tx.execute(
        "INSERT INTO transactions (item_id, user_id, type, quantity, note, timestamp) \
         VALUES (?1, ?2, 'OUT', ?3, ?4, ?5)",
        params![item_id, user_id, qty, note, now],
    )?;
    tx.commit()?;
    Ok(())
}

// ───────────────────────────────── transactions ─────────────────────────────────

fn row_to_tx(r: &Row) -> rusqlite::Result<Transaction> {
    let type_str: String = r.get("type")?;
    Ok(Transaction {
        id: r.get("id")?,
        item_id: r.get("item_id")?,
        item_name: r.get("item_name")?,
        user_name: r.get::<_, Option<String>>("user_name")?.unwrap_or_default(),
        tx_type: TxType::from_db(&type_str),
        quantity: r.get("quantity")?,
        note: r.get("note")?,
        timestamp: r.get("timestamp")?,
    })
}

/// ดึงประวัติทั้งหมด (join ชื่อของและผู้ใช้) เรียงเวลาล่าสุดก่อน
pub fn list_transactions(conn: &Connection) -> Result<Vec<Transaction>> {
    let mut stmt = conn.prepare(
        "SELECT t.id, t.item_id, i.name AS item_name, u.name AS user_name, \
         t.type, t.quantity, t.note, t.timestamp \
         FROM transactions t \
         JOIN items i ON i.id = t.item_id \
         LEFT JOIN users u ON u.id = t.user_id \
         ORDER BY t.timestamp DESC, t.id DESC",
    )?;
    let rows = stmt
        .query_map([], row_to_tx)?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

// ───────────────────────────────── alerts / reports ─────────────────────────────────

/// ของใกล้หมด: จำนวนคงเหลือ ≤ จำนวนขั้นต่ำ
/// (UI คำนวณจากแคชในหน่วยความจำ ฟังก์ชันนี้เป็น API ระดับ DB ที่ครอบด้วยเทสต์)
#[allow(dead_code)]
pub fn low_stock_items(conn: &Connection) -> Result<Vec<Item>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, category_id, location_id, unit, quantity, min_quantity, \
         barcode, expiry_date, created_at, updated_at FROM items \
         WHERE quantity <= min_quantity ORDER BY quantity ASC, name",
    )?;
    let rows = stmt
        .query_map([], row_to_item)?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

/// ของใกล้/หมดอายุ: มีวันหมดอายุ และวันหมดอายุ ≤ วันนี้ + days
/// (UI คำนวณจากแคชในหน่วยความจำ ฟังก์ชันนี้เป็น API ระดับ DB ที่ครอบด้วยเทสต์)
#[allow(dead_code)]
pub fn expiring_items(conn: &Connection, days: i64) -> Result<Vec<Item>> {
    let cutoff = (chrono::Local::now().date_naive() + chrono::Duration::days(days))
        .format("%Y-%m-%d")
        .to_string();
    let mut stmt = conn.prepare(
        "SELECT id, name, category_id, location_id, unit, quantity, min_quantity, \
         barcode, expiry_date, created_at, updated_at FROM items \
         WHERE expiry_date IS NOT NULL AND expiry_date <> '' AND expiry_date <= ?1 \
         ORDER BY expiry_date ASC, name",
    )?;
    let rows = stmt
        .query_map(params![cutoff], row_to_item)?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Db;

    fn sample_item(name: &str, qty: i64, min: i64) -> ItemInput {
        ItemInput {
            name: name.to_string(),
            category_id: None,
            location_id: None,
            unit: "ชิ้น".to_string(),
            quantity: qty,
            min_quantity: min,
            barcode: None,
            expiry_date: None,
        }
    }

    #[test]
    fn check_in_increases_quantity_and_logs() {
        let db = Db::open_in_memory().unwrap();
        let id = add_item(&db.conn, &sample_item("น้ำปลา", 5, 2)).unwrap();

        check_in(&db.conn, id, None, 3, "ซื้อเพิ่ม").unwrap();

        let items = list_items(&db.conn).unwrap();
        assert_eq!(items[0].quantity, 8);

        let txs = list_transactions(&db.conn).unwrap();
        assert_eq!(txs.len(), 1);
        assert_eq!(txs[0].tx_type, TxType::In);
        assert_eq!(txs[0].quantity, 3);
    }

    #[test]
    fn check_out_decreases_quantity_and_logs() {
        let db = Db::open_in_memory().unwrap();
        let id = add_item(&db.conn, &sample_item("ข้าวสาร", 10, 1)).unwrap();

        check_out(&db.conn, id, None, 4, "หุงข้าว").unwrap();

        let items = list_items(&db.conn).unwrap();
        assert_eq!(items[0].quantity, 6);
        let txs = list_transactions(&db.conn).unwrap();
        assert_eq!(txs[0].tx_type, TxType::Out);
    }

    #[test]
    fn check_out_rejects_when_insufficient_and_leaves_data_unchanged() {
        let db = Db::open_in_memory().unwrap();
        let id = add_item(&db.conn, &sample_item("ไข่", 3, 1)).unwrap();

        let result = check_out(&db.conn, id, None, 5, "");
        assert!(result.is_err(), "ต้องปฏิเสธเมื่อจำนวนไม่พอ");

        // จำนวนต้องไม่เปลี่ยน และต้องไม่มีประวัติถูกบันทึก (rollback)
        let items = list_items(&db.conn).unwrap();
        assert_eq!(items[0].quantity, 3);
        assert_eq!(list_transactions(&db.conn).unwrap().len(), 0);
    }

    #[test]
    fn check_out_rejects_non_positive_quantity() {
        let db = Db::open_in_memory().unwrap();
        let id = add_item(&db.conn, &sample_item("เกลือ", 5, 1)).unwrap();
        assert!(check_out(&db.conn, id, None, 0, "").is_err());
        assert!(check_in(&db.conn, id, None, -2, "").is_err());
    }

    #[test]
    fn low_stock_returns_only_items_at_or_below_min() {
        let db = Db::open_in_memory().unwrap();
        add_item(&db.conn, &sample_item("ของพอใช้", 10, 2)).unwrap();
        add_item(&db.conn, &sample_item("ของใกล้หมด", 1, 3)).unwrap();
        add_item(&db.conn, &sample_item("ของหมด", 0, 0)).unwrap();

        let low = low_stock_items(&db.conn).unwrap();
        let names: Vec<_> = low.iter().map(|i| i.name.as_str()).collect();
        assert!(names.contains(&"ของใกล้หมด"));
        assert!(names.contains(&"ของหมด"));
        assert!(!names.contains(&"ของพอใช้"));
    }

    #[test]
    fn expiring_returns_only_items_within_window() {
        let db = Db::open_in_memory().unwrap();
        let today = chrono::Local::now().date_naive();
        let soon = (today + chrono::Duration::days(5)).format("%Y-%m-%d").to_string();
        let far = (today + chrono::Duration::days(90)).format("%Y-%m-%d").to_string();

        let mut a = sample_item("นมใกล้หมดอายุ", 2, 0);
        a.expiry_date = Some(soon);
        add_item(&db.conn, &a).unwrap();

        let mut b = sample_item("ปลากระป๋อง", 5, 0);
        b.expiry_date = Some(far);
        add_item(&db.conn, &b).unwrap();

        let exp = expiring_items(&db.conn, 30).unwrap();
        let names: Vec<_> = exp.iter().map(|i| i.name.as_str()).collect();
        assert_eq!(names, vec!["นมใกล้หมดอายุ"]);
    }
}
