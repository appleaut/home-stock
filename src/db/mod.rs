//! ชั้นฐานข้อมูล (SQLite ผ่าน rusqlite) — เปิด connection, สร้าง schema และรวม query
//!
//! ชั้นนี้ไม่พึ่งพา egui เลย จึงทดสอบแยกได้ด้วย SQLite in-memory

pub mod models;
pub mod queries;

use anyhow::Result;
use rusqlite::Connection;

/// คำสั่งสร้างตารางทั้งหมด (idempotent — เรียกซ้ำได้)
const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS users (
    id   INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS categories (
    id   INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS locations (
    id   INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS items (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    name         TEXT NOT NULL,
    category_id  INTEGER REFERENCES categories(id),
    location_id  INTEGER REFERENCES locations(id),
    unit         TEXT NOT NULL DEFAULT 'ชิ้น',
    quantity     INTEGER NOT NULL DEFAULT 0,
    min_quantity INTEGER NOT NULL DEFAULT 0,
    barcode      TEXT,
    expiry_date  TEXT,
    created_at   TEXT NOT NULL,
    updated_at   TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_items_barcode ON items(barcode);

CREATE TABLE IF NOT EXISTS transactions (
    id        INTEGER PRIMARY KEY AUTOINCREMENT,
    item_id   INTEGER NOT NULL REFERENCES items(id),
    user_id   INTEGER REFERENCES users(id),
    type      TEXT NOT NULL CHECK(type IN ('IN','OUT')),
    quantity  INTEGER NOT NULL,
    note      TEXT NOT NULL DEFAULT '',
    timestamp TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_tx_item ON transactions(item_id);
CREATE INDEX IF NOT EXISTS idx_tx_time ON transactions(timestamp);

CREATE TABLE IF NOT EXISTS settings (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
"#;

/// ตัวห่อ connection ฐานข้อมูล
pub struct Db {
    pub conn: Connection,
}

impl Db {
    /// เปิดฐานข้อมูลจากไฟล์ (สร้างใหม่ถ้ายังไม่มี) พร้อมสร้าง schema
    pub fn open(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;
        Self::from_conn(conn)
    }

    /// เปิดฐานข้อมูลในหน่วยความจำ (สำหรับการทดสอบ)
    #[cfg(test)]
    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        Self::from_conn(conn)
    }

    fn from_conn(conn: Connection) -> Result<Self> {
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;
        conn.execute_batch(SCHEMA)?;
        Ok(Db { conn })
    }
}
