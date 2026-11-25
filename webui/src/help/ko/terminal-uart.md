UART 시리얼은 기기의 시리얼 콘솔(디버그 포트, 임베디드 보드, SBC)을 다른 터미널처럼 보고 상호 작용할 수 있는 터미널 탭으로 바꿉니다.

## 포트 설정

Web UI의 `UART` 섹션 → `Add UART Port`:

- `Name` — 터미널 제목으로 표시됩니다(예: "Debug Console");
- `Serial Port` — 감지된 포트에서 고릅니다(개수는 페이지 상단에 표시);
- `Baud Rate` / `Data Bits` / `Parity` / `Stop Bits` / `Flow Control` — 기기 펌웨어에 맞춥니다;
- `Terminal Type`과 `Encoding` — 인코딩은 기본 Auto(UTF-8)이며 펌웨어를 따를 수 있습니다;
- `Show Advanced Options`로 추가 시리얼 매개변수가 드러납니다.

저장된 포트는 `Configured UART Ports` 목록에 나타나며 언제든 편집·제거할 수 있습니다.

![UART 설정 페이지 — 포트 폼](images/terminal-uart-config.jpg)

## 클라이언트에서 사용

설정된 포트는 일반 서버 세션처럼 터미널 탭으로 열립니다:

- 시리얼 데이터는 도착하는 대로 표시되고 키보드 입력은 바로 포트로 갑니다;
- 마우스 오른쪽 클릭 메뉴에는 `Clear`(화면 비우기)와 `Encoding` 하위 메뉴(인코딩 실시간 전환, 현재 것에 체크 표시)가 있습니다;
- UART 세션에는 셸 기능이 없습니다(빠른 명령, Apps, Envs 메뉴 항목 없음).

전형적 용도: 라우터/스위치 콘솔, 임베디드 보드 로그, 플래시 지그 모니터링.
