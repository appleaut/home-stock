; Inno Setup script — ตัวติดตั้งระบบจัดการสต๊อกห้องเก็บของ
; คอมไพล์ด้วย: iscc installer\home-stock.iss  (รันจากโฟลเดอร์รากของโปรเจกต์)

#define MyAppName "ระบบจัดการสต๊อกห้องเก็บของ"
#define MyAppNameEn "Home Stock Manager"
#define MyAppVersion "0.1.0"
#define MyAppPublisher "Napawan"
#define MyAppExeName "home-stock.exe"

[Setup]
; AppId เป็น GUID คงที่ ใช้ระบุโปรแกรมสำหรับการอัปเดต/ถอนการติดตั้ง — อย่าเปลี่ยน
AppId={{8F3A1C7E-2B4D-4E9A-9C1F-6D5E7A8B0C12}
AppName={#MyAppNameEn}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
DefaultDirName={autopf}\HomeStock
DefaultGroupName={#MyAppNameEn}
DisableProgramGroupPage=yes
UninstallDisplayName={#MyAppNameEn}
UninstallDisplayIcon={app}\{#MyAppExeName}
; ติดตั้งระดับผู้ใช้ (ไม่ต้องใช้สิทธิ์ admin) แต่อนุญาตให้ผู้ใช้เลือกติดตั้งทั้งเครื่องได้
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
OutputDir=..\dist
OutputBaseFilename=HomeStock-Setup-{#MyAppVersion}
Compression=lzma2
SolidCompression=yes
WizardStyle=modern

[Languages]
Name: "thai"; MessagesFile: "compiler:Languages\Thai.isl"
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
Source: "..\target\release\{#MyAppExeName}"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\README.md"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\{#MyAppNameEn}"; Filename: "{app}\{#MyAppExeName}"
Name: "{autodesktop}\{#MyAppNameEn}"; Filename: "{app}\{#MyAppExeName}"; Tasks: desktopicon

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "{cm:LaunchProgram,{#MyAppNameEn}}"; Flags: nowait postinstall skipifsilent
