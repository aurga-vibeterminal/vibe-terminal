모든 터미널은 MCP(Model Context Protocol)의 일급 시민입니다. 외부 에이전트는 서버의 MCP 도구로 터미널을 구동합니다 — 터미널 나열, 화면 읽기, 명령 실행, 완료 대기 — 여러분이 이미 쓰는 코딩 에이전트가 실제 대화형 셸을 조작하게 합니다.

## 서버의 터미널 MCP 도구

터미널 세션을 대상으로 MCP 클라이언트는 다음을 할 수 있습니다:

- `list_terminals` — 열려 있는 모든 터미널. `get_terminal` — 한 터미널의 제목, cwd, 앱 상태;
- `get_screen` — 현재 보이는 화면 텍스트. `get_scrollback` — 최근 기록 줄;
- `send_command` — 명령줄 실행(Enter 추가). `send_input` — Enter 없는 원시 텍스트(`y` 확인, 폼 채우기). `send_key` — 이름 있는 키(Enter, Ctrl+C, 화살표);
- `clear_terminal` — 버퍼 비우기. `resize_terminal` — PTY 크기 조정. `wait_terminal` — 터미널이 유휴가 될 때까지 차단.

## 터미널 지정: Terminal ID

도구 호출은 Terminal ID로 터미널을 지정합니다. 터미널 안에서 마우스 오른쪽 클릭 → `MCP` → `Copy Terminal ID`로 복사한 뒤 그 ID를 에이전트에게 붙여 넣으면 정확히 그 터미널을 구동시킵니다.

![터미널 마우스 오른쪽 클릭 — MCP 하위 메뉴](images/terminal-mcp-menu.jpg)

## 원클릭 에이전트 앱 등록

`MCP` 하위 메뉴는 서버에서 감지된 에이전트 앱(편집기 CLI 등)도 나열합니다. 앱을 클릭하면 즉석의 `mcp add` 명령이 터미널에 입력됩니다 — **끝에 줄 바꿈 없이**; 검토한 뒤 직접 Enter를 눌러 이 서버의 MCP 서비스를 그 에이전트에 등록하세요. 앱이 이미 세션에서 실행 중인 동안에는 항목이 숨겨져 텍스트가 앱 UI 안에 떨어질 수 없습니다.

> 전형적 루프: Terminal ID를 에이전트에게 복사 → `send_command`로 빌드 시작 → `wait_terminal`로 완료 대기 → `get_screen`으로 결과 읽기. 자동 확인(터미널 장 참조)과 결합하면 긴 대화형 작업이 거의 무인으로 실행됩니다.
