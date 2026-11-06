<p align="center">
  <img src="assets/autotext.png" width="96" alt="AutoText">
</p>

<h1 align="center">AutoText</h1>

<p align="center">핫키 한 번으로 자주 쓰는 문구를 입력하는 Windows 트레이 프로그램<br>v1.0.0</p>

---

## 주요 기능

- **트레이 상주**: 실행하면 작업표시줄 알림 영역에 아이콘이 나타나고 백그라운드에서 동작함
- **상용구 등록**: 이름, 내용(여러 줄 가능), 핫키, 입력 방식, 사용 여부를 상용구별로 지정
- **핫키 조합**: `Ctrl`, `Alt`, `Shift`, `Win` 과 트리거 키(`A-Z`, `0-9`, `F1-F12`) 조합
- **두 가지 입력 방식**
  - 직접 입력: 한 글자씩 키 입력으로 전송하며 클립보드를 건드리지 않음
  - 클립보드 붙여넣기: `Ctrl+V` 로 붙여넣은 뒤 기존 클립보드 내용을 복원함
- **Windows 시작 시 자동 실행**: 설정 창에서 켜고 끌 수 있음
- **자동 업데이트**: GitHub 릴리스에 새 버전이 올라오면 알리고, 버튼 한 번으로 설치 후 다시 실행함
- **중복 실행 방지**: 다시 실행하면 이미 실행 중인 프로그램의 설정 창이 열림

## 설치

[Releases](https://github.com/workingdad365/autotext/releases) 에서 `AutoText-Setup-<버전>.exe` 를 내려받아 실행한다.

- 관리자 권한 없이 사용자별로 설치됨 (`%LocalAppData%\Programs\AutoText`)
- 설치 중 바탕 화면 바로 가기, Windows 로그인 시 자동 실행 여부를 선택할 수 있음
- 실행 중인 AutoText 는 설치 전에 자동으로 종료됨
- **설정 > 앱** 에서 제거할 수 있으며, 제거해도 상용구 설정 파일은 남음

### 업데이트

- 시작 10초 뒤와 이후 하루 한 번 GitHub 최신 릴리스를 확인함
- 새 버전이 있으면 설정 창이 열리고 상단에 안내가 표시됨 (자동 확인으로는 실행 중 한 번만 알림)
  - **지금 설치**: 설치 파일을 내려받아 설치한 뒤 프로그램을 다시 실행함
  - **이 버전 건너뛰기**: 해당 버전은 자동 확인에서 더 이상 알리지 않음
- 트레이 메뉴의 **업데이트 확인** 으로 언제든 직접 확인할 수 있음 (건너뛴 버전도 표시됨)
- 설정 창 하단의 **새 버전 자동 확인** 을 끄면 자동 확인을 하지 않음
- 업데이트 설치 시 자동 실행 설정은 그대로 유지됨

## 사용 방법

### 실행과 종료

| 동작 | 방법 |
|---|---|
| 설정 창 열기 | 트레이 아이콘 왼쪽 클릭, 또는 우클릭 메뉴의 **설정 열기** |
| 설정 창 닫기 | 창의 X 버튼 (프로그램은 트레이에서 계속 동작함) |
| 프로그램 종료 | 트레이 아이콘 우클릭 메뉴의 **종료** |

처음 실행할 때는 설정 창이 자동으로 열린다.

> 💡 Windows 11 은 새 트레이 아이콘을 `^` 숨김 영역에 넣는다. 아이콘을 작업표시줄로 끌어다 놓거나
> **설정 > 개인 설정 > 작업 표시줄 > 기타 시스템 트레이 아이콘** 에서 AutoText 를 켜면 항상 보인다.

### 상용구 등록

1. 설정 창에서 **+ 새 상용구** 클릭
2. 이름(선택), 핫키, 입력 방식, 상용구 내용 입력
3. **저장** 클릭 (또는 `Ctrl+S`)

저장 즉시 핫키가 적용되며, 원하는 프로그램에서 핫키를 누르면 내용이 입력된다.

### 핫키 규칙

- 문자/숫자 키는 `Win`, `Ctrl`, `Alt` 중 하나 이상을 반드시 조합해야 함 (일반 타이핑과 충돌 방지)
- `F1-F12` 는 단독 사용 가능
- 사용 중인 상용구끼리 같은 핫키는 저장되지 않음
- Windows 나 다른 프로그램이 이미 쓰는 핫키(예: `Win+E`, `Win+Shift+숫자`)는 등록에 실패함.
  이 경우 목록에 빨간색으로 표시되고, 마우스를 올리면 실패 사유가 보임

### 입력 방식 선택

| 방식 | 장점 | 적합한 경우 |
|---|---|---|
| 직접 입력 | 클립보드에 영향 없음 | 짧은 문구, 일반적인 경우 |
| 클립보드 붙여넣기 | 긴 글도 즉시 입력됨 | 긴 글, 직접 입력이 잘 안 되는 프로그램 |

클립보드 붙여넣기 방식의 세부 동작은 다음과 같다.

- 텍스트, 이미지, 파일 목록, HTML 등 기존 클립보드의 모든 형식을 백업한 뒤 붙여넣고 0.5초 후 복원함
- 그 사이 사용자가 새로 복사했다면 복원하지 않음
- 상용구와 복원한 내용은 Win+V 클립보드 기록과 클라우드 동기화에서 제외됨
- 클립보드를 사용할 수 없으면 직접 입력으로 대체함

## 설정 파일

상용구는 `%APPDATA%\AutoText\config.json` 에 저장된다. 백업하거나 다른 PC 로 옮길 때 이 파일을 복사하면 된다.

```json
{
  "snippets": [
    {
      "id": 1,
      "name": "인사말",
      "text": "안녕하세요.\n좋은 하루 되세요.",
      "hotkey": { "win": false, "ctrl": true, "alt": true, "shift": false, "key": "H" },
      "enabled": true,
      "mode": "paste"
    }
  ],
  "check_updates": true,
  "skipped_version": null
}
```

| 필드 | 설명 |
|---|---|
| `id` | 상용구 고유 번호 |
| `name` | 목록에 표시될 이름 (비어 있으면 내용 첫 줄 표시) |
| `text` | 입력될 내용 |
| `hotkey` | 수식키 사용 여부와 트리거 키 |
| `enabled` | 사용 여부 |
| `mode` | `type` (직접 입력, 기본값) 또는 `paste` (클립보드 붙여넣기) |
| `check_updates` | 새 버전 자동 확인 여부 (기본값 `true`) |
| `skipped_version` | 자동 확인에서 알리지 않을 버전 태그 (예: `"v1.0.1"`) |

자동 실행은 레지스트리 `HKCU\Software\Microsoft\Windows\CurrentVersion\Run` 의 `AutoText` 값으로 관리한다.
exe 를 다른 폴더로 옮긴 경우 한 번 직접 실행하면 등록 경로가 갱신된다.

## 제한 사항

- 관리자 권한으로 실행된 프로그램에는 입력되지 않음 (Windows UIPI 정책). 필요하면 AutoText 도 관리자 권한으로 실행해야 함
- Ditto 같은 클립보드 관리 프로그램이나 원격 데스크톱의 클립보드 공유 기능은 기록 제외 표시를 무시할 수 있음
- 클립보드를 읽는 데 0.5초 넘게 걸리는 무거운 프로그램에서는 복원된 원래 내용이 붙여넣어질 수 있음
- 설정 창을 한 번 연 뒤에는 그래픽 라이브러리(wgpu)가 메모리에 남아 사용량이 약 13MB 에서 약 76MB 로 늘어남

## 빌드

### 요구 사항

- Windows 10/11
- Rust 1.88 이상 (MSVC 툴체인, edition 2024)
- Visual Studio Build Tools 와 Windows SDK (exe 아이콘, 버전 정보 리소스 컴파일에 `rc.exe` 필요)
- 설치 파일 생성 시: [Inno Setup 6](https://jrsoftware.org/isinfo.php) (`winget install JRSoftware.InnoSetup`)
- 릴리스 게시 시: [GitHub CLI](https://cli.github.com) (`gh auth login` 완료 상태)

### 명령

```powershell
cargo build --release   # target\release\AutoText.exe 생성
cargo test              # 유닛 테스트
```

결과물은 단일 실행 파일이므로 설치 없이 바로 실행할 수도 있다.

### 설치 파일 생성과 릴리스

```powershell
.\build-installer.ps1                    # 릴리스 빌드 후 dist\AutoText-Setup-<버전>.exe 생성
.\build-installer.ps1 -Install           # 생성 직후 조용히 설치 (검증용)

.\release.ps1 "릴리스 설명"              # 버전 자동 증가 (1.0.0 -> 1.0.1) 후 GitHub Release 게시
.\release.ps1 "릴리스 설명" -Bump minor  # 1.0.1 -> 1.1.0 (major 는 2.0.0)
.\release.ps1 "릴리스 설명" -NoBump      # 현재 버전 그대로 게시 (게시 실패 후 재시도용)
.\release.ps1 "릴리스 설명" -DryRun      # 다음 버전만 출력
```

`release.ps1` 은 다음 순서로 동작한다. 커밋 전 단계에서 실패하면 버전 변경을 되돌린다.

1. 작업 트리가 깨끗한지, 원격보다 뒤처지지 않았는지, 같은 버전 릴리스가 없는지 확인
2. `Cargo.toml`, `README.md`, `installer\AutoText.iss` 의 버전 표기 변경
3. `cargo test` 실행 후 `build-installer.ps1` 로 설치 파일 생성 (`Cargo.lock` 도 갱신됨)
4. `release vX.Y.Z` 커밋 생성 후 푸시
5. 설치 파일을 첨부해 GitHub Release 게시. 실행 중인 AutoText 가 이 릴리스를 새 버전으로 감지함

네트워크가 필요한 업데이트 테스트(실제 릴리스 조회와 다운로드)는 기본 테스트에서 제외되어 있다.

```powershell
cargo test -- --ignored
```

### 아이콘 재생성

`assets/autotext.ico`, `assets/autotext.png` 는 스크립트로 생성한다. Python 과 Pillow 가 필요하다.

```powershell
python tools/gen_icon.py
```

## 프로젝트 구조

```
autotext/
+-- assets/
|   +-- autotext.ico    # exe, 트레이 아이콘 (16-256px)
|   +-- autotext.png    # 창 아이콘
+-- src/
|   +-- main.rs         # 진입점, 중복 실행 방지, 스레드 간 공유 상태
|   +-- config.rs       # 상용구 모델, 핫키 검증, 설정 파일 저장/로드
|   +-- service.rs      # 트레이 아이콘, 전역 핫키 등록 (백그라운드 Win32 스레드)
|   +-- input.rs        # SendInput 기반 직접 입력, 붙여넣기 입력
|   +-- clipboard.rs    # 클립보드 백업, 설정, 복원
|   +-- autostart.rs    # 자동 실행 레지스트리 관리
|   +-- update.rs       # GitHub 릴리스 확인, 설치 파일 다운로드 (WinHTTP)
|   +-- ui.rs           # 설정 창 (egui)
+-- installer/
|   +-- AutoText.iss    # Inno Setup 설치 스크립트
+-- tools/gen_icon.py   # 아이콘 생성 스크립트
+-- build.rs            # Windows 리소스(아이콘, 버전 정보) 포함
+-- build-installer.ps1 # 설치 파일 생성 스크립트
+-- release.ps1         # GitHub Release 게시 스크립트
+-- Cargo.toml
```

### 동작 구조

- **서비스 스레드**: 메시지 전용 창을 만들어 트레이 아이콘과 `RegisterHotKey` 전역 핫키를 처리함. 프로그램이 실행되는 동안 항상 동작함
- **메인 스레드**: 설정 창 열기 요청이 오면 egui 창을 만들고, 창이 닫히면 다음 요청을 기다림
- **입력 스레드**: 핫키가 눌릴 때마다 생성됨. 수식키가 모두 떼어질 때까지 기다린 뒤 입력함
- **업데이트 스레드**: 업데이트 확인, 다운로드 때만 생성됨. 결과는 서비스 스레드에 메시지로 전달함

### 사용 라이브러리

| 크레이트 | 용도 |
|---|---|
| [eframe / egui](https://github.com/emilk/egui) | 설정 창 UI |
| [tray-icon](https://github.com/tauri-apps/tray-icon) | 트레이 아이콘, 우클릭 메뉴 |
| [windows](https://github.com/microsoft/windows-rs) | 핫키, 키 입력, 클립보드, 레지스트리, WinHTTP Win32 API |
| [serde / serde_json](https://serde.rs) | 설정 파일 직렬화 |
| [winresource](https://github.com/BenjaminRi/winresource) | exe 아이콘, 버전 정보 리소스 |
