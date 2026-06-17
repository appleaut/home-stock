# 📦 ระบบจัดการสต๊อกห้องเก็บของบ้าน (Home Stock Manager)

แอป Windows desktop เขียนด้วย **Rust + egui** สำหรับจัดการของในห้องเก็บของบ้าน
รองรับการรับเข้า/เบิกออก, ประวัติ, แจ้งเตือนของใกล้หมด/ใกล้หมดอายุ, บาร์โค้ด และ export CSV

> เอกสารวิเคราะห์ระบบอยู่ในโฟลเดอร์ [`analysis/`](analysis/)

## ฟีเจอร์
- **รายการของ** — เพิ่ม/แก้ไข/ลบ พร้อมหมวดหมู่ ตำแหน่งเก็บ หน่วยนับ จำนวนขั้นต่ำ บาร์โค้ด วันหมดอายุ
- **รับเข้า/เบิกออก** — เพิ่ม/ลดจำนวน (กันเบิกเกินคงเหลือ) รองรับสแกนบาร์โค้ดด้วยเครื่อง USB
- **ประวัติ** — บันทึกทุกการเคลื่อนไหวว่าใคร/เมื่อไหร่/เข้า-ออกเท่าไหร่ กรองตามของ/ผู้ใช้
- **แจ้งเตือน** — ของใกล้หมด (เหลือ ≤ ขั้นต่ำ) และของใกล้/หมดอายุ (ปรับจำนวนวันล่วงหน้าได้)
- **รายงาน** — สรุปยอด + export รายการของ/ประวัติเป็น CSV (UTF-8 + BOM อ่านภาษาไทยใน Excel ได้)
- เลือกผู้ทำรายการจาก dropdown (ไม่มี login) ทั้งหมดเป็นภาษาไทย
- **ธีม Dark / Light** สลับได้จากปุ่มมุมบนขวา (จำค่าไว้ในฐานข้อมูล) ดีไซน์แนว Microsoft Fluent
  พร้อม navigation rail ด้านซ้ายสไตล์ Windows 11

ข้อมูลเก็บใน SQLite ไฟล์เดียว `home_stock.db` (สร้างอัตโนมัติข้างไฟล์ .exe)

## ความต้องการในการ build

ต้องมี C compiler + linker เพราะ `rusqlite` (feature `bundled`) คอมไพล์ SQLite จากซอร์ส C

### ทางเลือก A — MSVC (มาตรฐานบน Windows)
ติดตั้ง **Visual Studio Build Tools** พร้อม workload "Desktop development with C++"
แล้ว build ด้วย toolchain `stable-x86_64-pc-windows-msvc` (ค่าเริ่มต้นของ rustup)

### ทางเลือก B — MinGW + GNU toolchain (ไม่ต้องใช้สิทธิ์ admin)
เป็นวิธีที่ใช้ตั้งค่าโปรเจกต์นี้ (เครื่องไม่มี MSVC build tools):

```powershell
scoop install mingw                                  # ได้ gcc + ld
rustup toolchain install stable-x86_64-pc-windows-gnu
rustup override set stable-x86_64-pc-windows-gnu     # ผูก toolchain กับโฟลเดอร์นี้
```
ตรวจให้ `...\scoop\apps\mingw\current\bin` อยู่ใน PATH (มี `gcc`)

## Build & Run

```powershell
cargo run                      # รันแบบ debug
cargo test                     # รัน unit test ของชั้น DB
cargo build --release          # ได้ target\release\home-stock.exe (รันได้เดี่ยว ๆ)
cargo run --example seed -- dark   # (ตัวช่วย dev) ใส่ข้อมูลตัวอย่าง + ตั้งธีม light/dark
```

## สร้างตัวติดตั้ง (Installer)

ตัวติดตั้งใช้ **Inno Setup** สร้างไฟล์ `setup.exe` เดียวที่ติดตั้งแอป + สร้าง shortcut เมนู Start
(และ desktop ถ้าเลือก) + ตัวถอนการติดตั้ง โดยติดตั้งระดับผู้ใช้ (ไม่ต้องใช้สิทธิ์ admin)

```powershell
scoop bucket add extras       # ครั้งเดียว
scoop install inno-setup      # ครั้งเดียว
.\installer\build-installer.ps1
```
ได้ไฟล์ติดตั้งที่ `dist\HomeStock-Setup-0.1.0.exe`

> ฐานข้อมูลถูกเก็บที่ `%APPDATA%\HomeStock\home_stock.db` (แยกจากที่ติดตั้ง) จึงเขียนได้แม้ติดตั้งใน Program Files
> และข้อมูลไม่หายเมื่อถอน/ติดตั้งใหม่

## โครงสร้างโปรเจกต์
```
analysis/        เอกสารวิเคราะห์ (requirements, architecture, data model, UI flows)
assets/fonts/    ฟอนต์ไทย Sarabun (OFL) ฝังในไบนารี
src/
  main.rs        entry point + โหลดฟอนต์ไทย + เปิด DB
  db/            ชั้นฐานข้อมูล (ไม่พึ่ง UI, ทดสอบแยกได้): mod / models / queries
  app/           ชั้น UI (egui): mod / state / views/{items,checkinout,history,alerts,reports}
```
