빠른 명령은 프로젝트의 빌드/테스트/배포 명령을 터미널의 마우스 오른쪽 클릭 메뉴에 고정합니다 — 한 번 클릭으로 실행, 어디서든 사용 가능. 명령은 리포지토리별로 리포지토리 루트의 `.vterm/commands.json`에서 관리됩니다.

## commands.json 만들기

터미널 안에서 마우스 오른쪽 클릭 → `Commands`:

- 세션의 리포지토리에 commands.json이 아직 없으면 메뉴에는 단일 항목 `Generate commands.json`이 표시됩니다. 클릭하면 서버가 프로젝트 유형(npm / Cargo / .NET / Xcode / Gradle-Android / 일반)을 감지해 보수적인 시작 템플릿을 씁니다. 이미 있는 비어 있지 않은 파일은 **결코 덮어쓰지 않습니다**;
- 파일 위치: Git 리포지토리 루트의 `.vterm/commands.json`(비 Git 폴더에서는 현재 작업 디렉터리).

## 다듬기

생성된 템플릿에는 감지된 프로젝트 유형의 흔한 명령만 들어 있습니다. 코드 에이전트(예: 이 서버의 MCP에 연결된 편집기 에이전트)에게 프로젝트를 읽고 파일을 다듬게 하세요:

```json
{
  "commands": [
    { "name": "Build", "command": "cargo build --release" },
    { "name": "Test verbose", "command": "cargo test -- --nocapture",
      "env": { "RUST_LOG": "debug" } },
    { "name": "Package (Windows)", "command": "./packaging/windows/package.bat",
      "os": "win" }
  ]
}
```

최상위 배열도 받아들여집니다. `env`는 선택 사항이며 명령 실행 전 변수를 설정합니다. 구형 `value` 키도 계속 읽힙니다. 선택적 `os` 필드는 항목을 `vterm-server`가 실행되는 운영 체제로 제한합니다. Windows는 `"win"`, Linux는 `"linux"`, macOS는 `"osx"`를 사용합니다. `os`를 생략하거나 `"all"` 또는 인식할 수 없는 값으로 설정하면 항목이 모든 플랫폼에서 사용 가능합니다. 서버는 CWD와 commands.json 변경을 지켜보고 브로드캐스트합니다 — 메뉴를 다시 열면 새 명령이 있습니다.

## 실행

마우스 오른쪽 클릭 → `Commands` → 항목을 클릭하면 그 터미널에서 실행됩니다(선택적 env이 먼저 적용됨). 세션에서 에이전트 앱이 실행 중인 동안에는 메뉴를 사용할 수 없어 명령 텍스트가 앱 UI 안에 떨어질 수 없습니다.

## 예: Wi-Fi로 Android 빌드 & 설치

Gradle-Android 프로젝트에서의 전체 안내: 무선 ADB 설정과 빌드-설치 흐름을 마우스 오른쪽 클릭 메뉴에 고정하고, 한 번 클릭으로 앱을 다시 빌드합니다.

**1. 휴대폰을 무선 ADB로 전환.** 이 두 명령은 기기마다 한 번 입력합니다 — 휴대폰이 USB에 연결된 동안 `adb tcpip 5555`, 이후 휴대폰 주소로 `adb connect`:

![터미널 — adb tcpip 5555와 adb connect로 Android 휴대폰을 무선 ADB로 전환](images/terminal-commands-adb-setup.gif)

**2. 코드 에이전트에게 명령 작성을 맡기기.** 프로젝트의 코드 에이전트(여기서는 Claude Code)를 터미널에서 열고 빌드 & 설치 명령 추가를 요청합니다 — 프로젝트 구조를 읽고 `.vterm/commands.json`에 항목을 씁니다:

![터미널 — Claude Code가 commands.json에 빌드 & 설치 APK 명령 편집](images/terminal-commands-agent-edit.mp4)

**3. 메뉴에서 빌드하고 설치.** 터미널을 분할해 빌드 출력을 전용 창에 두고, 마우스 오른쪽 클릭 → `Commands` → 항목 — Gradle이 APK를 조립하고 Wi-Fi로 휴대폰에 설치합니다:

![터미널 — 창 분할 후 마우스 오른쪽 클릭 메뉴에서 Android 빌드 & 설치 명령 실행](images/terminal-commands-split-run.mp4)
