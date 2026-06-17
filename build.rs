// ฝังไอคอนแอปไว้ในไฟล์ .exe สำหรับ Windows
// ทำให้ shortcut / taskbar / Alt-Tab / Add-Remove Programs แสดงไอคอนนี้อัตโนมัติ
fn main() {
    #[cfg(windows)]
    {
        println!("cargo:rerun-if-changed=assets/icon.ico");
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/icon.ico");
        if let Err(e) = res.compile() {
            // อย่าทำให้ build ล้มถ้าฝังไอคอนไม่สำเร็จ (เช่น ไม่มี windres ใน PATH)
            println!("cargo:warning=ฝังไอคอนไม่สำเร็จ: {e}");
        }
    }
}
