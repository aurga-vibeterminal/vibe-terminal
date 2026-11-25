서버는 Model Context Protocol 호스트를 내장하고 있으며 Web UI의 `MCP` 섹션으로 설정합니다. Code Agent(Claude Code, Codex, Kimi Code, Pi, OpenCode, Qwen Code, DeepSeek Harness 등)가 자체 채팅에서 연결해 터미널을 읽고 조작하며 EasyEDA Pro까지 직접 다룰 수 있습니다.

## 서버 설정

MCP 서버를 켜고 다음을 설정합니다:

- `Port`(기본 `6180`)와 `Bind Address` —— `127.0.0.1`은 로컬호스트 전용, `0.0.0.0`은 네트워크에 개방；
- `Access Token` —— localhost 이외의 요청에 필수인 Bearer 토큰입니다. localhost 요청은 항상 허용됩니다. 토큰은 비어 있지 않습니다 —— 신규 설치 시 무작위 토큰이 제공되며 주사위 버튼으로 재생성, 복사 버튼으로 꺼낼 수 있습니다；
- `Stream Throttle` —— 연결된 에이전트로 화면 스트림을 밀어주는 최소 간격.

변경 사항은 `Save` 후 적용되며, 서버는 재시작 없이 핫 리로드됩니다.

## 엔드포인트

각 기능 집합은 고유 URL `http://localhost:{port}/{id}/mcp`로 제공됩니다:

- **Terminal MCP**(`/terminal/mcp`) —— 내장 터미널 기능. 에이전트는 터미널 나열, 보이는 화면과 스크롤백 읽기, 원시 입력과 명명된 키 전송, 크기 조절, 프롬프트 대기를 할 수 있습니다. 도구 호출은 Terminal ID로 터미널을 지정합니다 —— 터미널 우클릭 → `MCP` → `Copy Terminal ID`로 ID를 복사해 에이전트 컨텍스트에 붙여넣으세요(Terminals 섹션의 Terminal MCP 참조)；
- **EasyEDA Pro MCP**(`/easyeda/mcp`) —— EasyEDA Pro를 에이전트에 연결하는 내장 MCP 애드온. 소수의 고정 도구 세트와 검색 가능한 `eda.*` API 카탈로그를 제공하며 WebSocket으로 EasyEDA 커넥터 확장에 전달합니다. 먼저 `Addons` 섹션에서 애드온을 활성화하세요 —— 설치하면 엔드포인트가 목록에 나타나고 배지가 애드온 데몬의 연결 여부를 보여줍니다. 애드온의 `Timeout` 필드는 브리지 요청마다 적용되는 상한입니다(느린 EDA 작업에는 늘리세요)；
- **기타 MCP 애드온** —— 설치된 MCP 유형 애드온은 같은 방식으로 엔드포인트를 추가하며, 연결/끊김 배지가 실시간 표시됩니다.

목록 위 드롭다운에서 에이전트 CLI를 고르면 엔드포인트 옆 복사 버튼이 해당 앱용 실행 가능한 설정 명령을 건네줍니다.

## 에이전트 연결

토큰을 직접 붙여넣을 필요가 없습니다. 서버는 `VTERM_MCP_TOKEN`을 내보내는 자동 생성된 `Vibe Terminal MCP Bearer Token` 환경 프로필(Environment Variables 섹션 참조)을 유지합니다. 에이전트 연결 전에 터미널에 적용한 뒤 복사한 명령을 사용하세요:

- **셸 명령 앱**(Claude Code, Codex) —— 일반 터미널에서 복사한 `mcp add` 명령을 실행합니다. Codex는 `--bearer-token-env-var`로 `VTERM_MCP_TOKEN`에서 토큰을 읽으므로 명령줄에 비밀이 노출되지 않습니다；
- **Kimi Code** —— `mcp add` 명령이 없습니다. 프로필을 적용한 상태로 Kimi를 시작한 뒤 복사한 `/mcp-config …` 프롬프트를 대화에 보내면 `bearerTokenEnvVar`로 서버를 등록합니다.
- **Qwen Code** —— `mcp add` 명령이 없습니다. 프로필을 적용한 상태로 Qwen Code를 시작한 뒤 복사한 프롬프트를 대화에 보내면 에이전트가 MCP 설정을 직접 업데이트합니다. 세션 안에서 `/mcp`로 연결을 점검할 수 있습니다.

Pi 안에서는 `/mcp`가 연결을 점검하고 `/reload`가 세션 밖에서 추가한 서버를 반영합니다.

설정 아래 `Command Templates` 카드에 앱별 설정 명령이 있습니다 —— 항목 편집, 내장 항목 덮어쓰기, 새 에이전트 CLI용 템플릿 추가가 가능하며 미리보기가 복사 버튼의 결과를 그대로 보여줍니다.
