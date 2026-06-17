# 03 — แบบจำลองข้อมูล (Data Model)

## ER Diagram (แบบข้อความ)

```
┌────────────┐        ┌──────────────┐        ┌──────────────┐
│ categories │        │    items     │        │  locations   │
│────────────│        │──────────────│        │──────────────│
│ id (PK)    │◄───────│ category_id  │   ┌───►│ id (PK)      │
│ name       │   1   ∞│ location_id  │───┘    │ name         │
└────────────┘        │ id (PK)      │ ∞    1 └──────────────┘
                      │ name         │
                      │ unit         │
                      │ quantity     │
                      │ min_quantity │
                      │ barcode      │
                      │ expiry_date  │
                      │ created_at   │
                      │ updated_at   │
                      └──────┬───────┘
                             │ 1
                             │
                             │ ∞
                      ┌──────▼────────┐        ┌──────────┐
                      │ transactions  │        │  users   │
                      │───────────────│        │──────────│
                      │ id (PK)       │∞     1 │ id (PK)  │
                      │ item_id (FK)  │   ┌───►│ name     │
                      │ user_id (FK)  │───┘    └──────────┘
                      │ type (IN/OUT) │
                      │ quantity      │
                      │ note          │
                      │ timestamp     │
                      └───────────────┘
```

## รายละเอียดตาราง (SQLite Schema)

### `users` — ผู้ทำรายการ
| คอลัมน์ | ชนิด | หมายเหตุ |
|---------|------|----------|
| id | INTEGER PK AUTOINCREMENT | |
| name | TEXT NOT NULL UNIQUE | ชื่อสมาชิกในบ้าน |

### `categories` — หมวดหมู่
| คอลัมน์ | ชนิด | หมายเหตุ |
|---------|------|----------|
| id | INTEGER PK AUTOINCREMENT | |
| name | TEXT NOT NULL UNIQUE | เช่น ของกิน, ของใช้, ยา |

### `locations` — ตำแหน่งเก็บ
| คอลัมน์ | ชนิด | หมายเหตุ |
|---------|------|----------|
| id | INTEGER PK AUTOINCREMENT | |
| name | TEXT NOT NULL UNIQUE | เช่น ชั้นบนซ้าย, ลิ้นชัก A |

### `items` — รายการของ
| คอลัมน์ | ชนิด | หมายเหตุ |
|---------|------|----------|
| id | INTEGER PK AUTOINCREMENT | |
| name | TEXT NOT NULL | ชื่อของ |
| category_id | INTEGER FK → categories(id) | NULL ได้ |
| location_id | INTEGER FK → locations(id) | NULL ได้ |
| unit | TEXT NOT NULL | หน่วยนับ เช่น ชิ้น, ขวด, กล่อง |
| quantity | INTEGER NOT NULL DEFAULT 0 | จำนวนคงเหลือ (≥ 0) |
| min_quantity | INTEGER NOT NULL DEFAULT 0 | จุดแจ้งเตือนของใกล้หมด |
| barcode | TEXT | รหัสบาร์โค้ด (NULL ได้), index เพื่อค้นเร็ว |
| expiry_date | TEXT | วันหมดอายุรูปแบบ ISO `YYYY-MM-DD` (NULL ได้) |
| created_at | TEXT NOT NULL | timestamp ISO |
| updated_at | TEXT NOT NULL | timestamp ISO |

### `transactions` — ประวัติการเคลื่อนไหว
| คอลัมน์ | ชนิด | หมายเหตุ |
|---------|------|----------|
| id | INTEGER PK AUTOINCREMENT | |
| item_id | INTEGER NOT NULL FK → items(id) | |
| user_id | INTEGER FK → users(id) | ใครทำรายการ |
| type | TEXT NOT NULL CHECK(type IN ('IN','OUT')) | ประเภท |
| quantity | INTEGER NOT NULL | จำนวนที่เข้า/ออก (> 0) |
| note | TEXT | หมายเหตุ |
| timestamp | TEXT NOT NULL | เวลาทำรายการ ISO |

## หลักการสำคัญด้านความถูกต้อง
- **Atomic check-in/out:** อัปเดต `items.quantity` และ insert `transactions`
  ภายใน SQL transaction เดียว — ถ้าขั้นใดล้มเหลว rollback ทั้งหมด
  เพื่อให้ยอดคงเหลือกับผลรวมประวัติตรงกันเสมอ
- **กันติดลบ:** check-out ตรวจ `quantity >= qty` ก่อน ถ้าไม่พอคืน error
- **วันที่เก็บเป็น TEXT ISO 8601** (`YYYY-MM-DD` / RFC3339) — เรียง/เทียบด้วย string ได้ตรงลำดับเวลา
- **Foreign key** เปิดใช้งาน (`PRAGMA foreign_keys = ON`)
- ตำแหน่งไฟล์ฐานข้อมูล: `home_stock.db` ข้างไฟล์ .exe (หรือโฟลเดอร์ข้อมูลผู้ใช้)
