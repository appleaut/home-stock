# 02 — สถาปัตยกรรมระบบ (Architecture)

## เทคโนโลยีที่เลือกและเหตุผล

| ส่วน | เลือกใช้ | เหตุผล |
|------|---------|--------|
| ภาษา | Rust | ประสิทธิภาพสูง, ปลอดภัยเรื่อง memory, build เป็น native .exe |
| GUI | egui / eframe | Pure Rust ล้วน, build เป็น .exe ไฟล์เดียว, เหมาะกับฟอร์ม+ตาราง |
| ตาราง UI | egui_extras (TableBuilder) | แสดงข้อมูลแบบตารางที่จัดคอลัมน์ได้ |
| ฐานข้อมูล | SQLite ผ่าน rusqlite (feature `bundled`) | embedded ไฟล์เดียว, ไม่ต้องลง server/DLL, รองรับ transaction |
| วันที่/เวลา | chrono | คำนวณ timestamp และช่วงวันหมดอายุ |
| รายงาน | csv | เขียนไฟล์ CSV รองรับ UTF-8 |
| Dialog เซฟไฟล์ | rfd | เลือกตำแหน่งบันทึก CSV |
| Error | anyhow | จัดการ error แบบกระชับ |

### ทำไม egui แทน Tauri/Slint
- **Tauri** ต้องเขียน frontend แยกด้วย HTML/CSS/JS — ซับซ้อนเกินจำเป็นสำหรับแอปภายในบ้าน
- **Slint** ต้องเรียน markup language เฉพาะตัว
- **egui** เป็น immediate-mode GUI ใน Rust ล้วน เขียน UI กับ logic ในภาษาเดียว, build ง่าย

### หมายเหตุเรื่องฟอนต์ไทย
egui ไม่มีฟอนต์ที่ render ภาษาไทยมาให้ default จึงต้อง **ฝังฟอนต์ไทย** (Sarabun, OFL license)
ผ่าน `include_bytes!` แล้วลงทะเบียนใน `FontDefinitions` ตอนเริ่มแอป เพื่อให้ได้ .exe ที่มีฟอนต์ในตัว

## โครงสร้างโมดูล (แยก Logic ออกจาก UI)

```
src/
├─ main.rs          → entry point: ตั้งค่า eframe, โหลดฟอนต์ไทย, เปิด DB
├─ db/              → ★ ชั้น logic ล้วน ไม่พึ่ง egui ทดสอบได้อิสระ
│  ├─ mod.rs        → เปิด connection + สร้าง schema (migration)
│  ├─ models.rs     → struct ข้อมูล: Item, Transaction, User, Category, Location
│  └─ queries.rs    → CRUD, check_in/check_out (atomic), query สำหรับ alert/report
└─ app/             → ★ ชั้น UI ล้วน เรียกใช้ db
   ├─ mod.rs        → struct App (impl eframe::App), routing 5 แท็บ
   ├─ state.rs      → สถานะ UI: แท็บปัจจุบัน, ค่าในฟอร์ม, ผู้ใช้ที่เลือก, ตัวกรอง
   └─ views/        → โค้ดวาดแต่ละแท็บ
      ├─ items.rs, checkinout.rs, history.rs, alerts.rs, reports.rs
```

**หลักการแยกส่วน:** ชั้น `db` ไม่รู้จัก egui เลย รับ-คืนเป็น struct ธรรมดา
→ ทดสอบ logic ด้วย SQLite in-memory ได้โดยไม่ต้องเปิด UI
ชั้น `app` ทำหน้าที่วาดหน้าจอและเรียก `db` เท่านั้น

## Data Flow

```
ผู้ใช้คลิก/พิมพ์บน egui (app/views/*)
        │
        ▼
เรียกฟังก์ชันใน db/queries.rs  ──►  SQLite (transaction ถ้าจำเป็น)
        │                                  │
        ▼                                  ▼
ได้ struct กลับมา (models)        ข้อมูลถูกบันทึกถาวร
        │
        ▼
egui วาดผลลัพธ์ใหม่ในเฟรมถัดไป
```

แอปโหลดข้อมูลจาก DB ใส่ cache ใน state แล้วรีเฟรช cache หลังการกระทำที่เปลี่ยนข้อมูล
(เพิ่ม/แก้/ลบ/check-in/out) เพื่อไม่ query ทุกเฟรม

## การจัดการ Error
- ชั้น `db` คืน `anyhow::Result<T>`
- ชั้น `app` แสดงข้อความ error ให้ผู้ใช้เห็น (แถบข้อความ/สีแดง) แทนที่จะ panic
- check-out ที่จำนวนไม่พอ คืน error และไม่แก้ไขข้อมูล

## การทดสอบ
- **Unit test** ใน `db/queries.rs` ใช้ SQLite `:memory:`:
  check-in เพิ่มจำนวน, check-out ลดจำนวน+กันติดลบ, low-stock query, expiry query
- **Manual test** end-to-end ผ่าน UI (ดูเอกสาร 04 และแผน)
