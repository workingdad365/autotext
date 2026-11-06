; AutoText 설치 스크립트 (Inno Setup 6)
; build-installer.ps1이 릴리스 빌드 결과(target\release\AutoText.exe)를 넣고 이 스크립트를 컴파일함
; 사용자별 설치 (관리자 권한 불필요): %LocalAppData%\Programs\AutoText

#ifndef AppVersion
  #define AppVersion "1.0.0"
#endif
#ifndef BuildDir
  #define BuildDir "..\target\release"
#endif

#define AppName "AutoText"
#define AppExe "AutoText.exe"
#define AppPublisher "drasys"
#define AppUrl "https://github.com/workingdad365/autotext"

[Setup]
AppId={{3D065477-9D26-4577-966E-39D2C159AA3C}
AppName={#AppName}
AppVersion={#AppVersion}
AppVerName={#AppName} {#AppVersion}
AppPublisher={#AppPublisher}
AppPublisherURL={#AppUrl}
AppSupportURL={#AppUrl}
VersionInfoVersion={#AppVersion}
DefaultDirName={localappdata}\Programs\AutoText
DisableDirPage=yes
DefaultGroupName={#AppName}
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
OutputDir=..\dist
OutputBaseFilename=AutoText-Setup-{#AppVersion}
SetupIconFile=..\assets\autotext.ico
UninstallDisplayIcon={app}\{#AppExe}
UninstallDisplayName={#AppName}
Compression=lzma2/ultra64
SolidCompression=yes
WizardStyle=modern
; 트레이 앱은 최상위 창이 없어 정상 종료 요청을 받지 못하므로 강제 종료 (PrepareToInstall에서 먼저 종료함)
CloseApplications=force
RestartApplications=no
ShowLanguageDialog=no

[Languages]
Name: "korean"; MessagesFile: "compiler:Languages\Korean.isl"
Name: "english"; MessagesFile: "compiler:Default.isl"

[CustomMessages]
korean.LaunchAfterInstall=설치 후 {#AppName} 실행
korean.CreateDesktopIcon=바탕 화면 바로 가기 만들기
korean.AutoStart=Windows 로그인 시 자동 실행
english.LaunchAfterInstall=Launch {#AppName} after install
english.CreateDesktopIcon=Create a desktop shortcut
english.AutoStart=Start automatically at Windows sign-in

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; Flags: unchecked
Name: "autostart"; Description: "{cm:AutoStart}"

[Files]
Source: "{#BuildDir}\{#AppExe}"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\{#AppName}"; Filename: "{app}\{#AppExe}"
Name: "{autodesktop}\{#AppName}"; Filename: "{app}\{#AppExe}"; Tasks: desktopicon

[Registry]
; 앱 설정 창의 자동 실행과 같은 키/값/형식(--autostart 인자 포함)을 사용하므로 서로 호환됨
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\Run"; ValueType: string; ValueName: "AutoText"; ValueData: """{app}\{#AppExe}"" --autostart"; Tasks: autostart; Flags: uninsdeletevalue

[Run]
Filename: "{app}\{#AppExe}"; Description: "{cm:LaunchAfterInstall}"; Flags: nowait postinstall skipifsilent

[UninstallRun]
; 제거 전 실행 중인 앱 종료
Filename: "{sys}\taskkill.exe"; Parameters: "/IM {#AppExe} /F"; Flags: runhidden; RunOnceId: "KillApp"

[Code]
// 트레이 앱은 Restart Manager의 정상 종료 요청에 응답하지 않아 강제 종료까지 약 30초 대기하므로 미리 종료
function PrepareToInstall(var NeedsRestart: Boolean): String;
var
  ResultCode: Integer;
begin
  Exec(ExpandConstant('{sys}\taskkill.exe'), '/IM {#AppExe} /F', '', SW_HIDE, ewWaitUntilTerminated, ResultCode);
  Result := '';
end;

// 제거 시 앱이 직접 등록한 자동 실행 값도 정리 (설치 작업으로 만든 값은 uninsdeletevalue가 처리)
procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
begin
  if CurUninstallStep = usPostUninstall then
    RegDeleteValue(HKEY_CURRENT_USER, 'Software\Microsoft\Windows\CurrentVersion\Run', 'AutoText');
end;
