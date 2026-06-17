# สร้างตัวติดตั้ง (setup.exe) แบบครบขั้นตอน
# รันจากโฟลเดอร์รากของโปรเจกต์:  .\installer\build-installer.ps1

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot

# ให้ใช้ MinGW gcc สำหรับ build (กรณีเครื่องไม่มี MSVC)
$env:Path = "$env:USERPROFILE\scoop\apps\mingw\current\bin;$env:USERPROFILE\scoop\shims;$env:Path"

Write-Host "==> build release exe" -ForegroundColor Cyan
Push-Location $root
cargo build --release
Pop-Location

Write-Host "==> compile installer ด้วย Inno Setup" -ForegroundColor Cyan
iscc "$root\installer\home-stock.iss"

Write-Host "==> เสร็จ — ดูไฟล์ติดตั้งที่ $root\dist\" -ForegroundColor Green
Get-ChildItem "$root\dist\*.exe" | Select-Object Name, @{N='MB';E={[math]::Round($_.Length/1MB,1)}}
