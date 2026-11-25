`vtermc`는 커맨드라인 클라이언트입니다. 서버에 연결해 터미널 세션을 만들거나 연결하고 저장된 서버 목록을 관리합니다 — 모두 일반 터미널 안에서, SSH 너머로, 또는 헤드리스 머신에서. 데스크톱 클라이언트와 `~/.vibeterminal/client.json`을 공유하므로 양쪽이 같은 서버를 봅니다.

## 터미널 시작

모드 플래그 없이 `vtermc`를 실행하면 로컬 서버(기본 `wss://localhost:5153`)에 감지된 셸로 새 터미널을 만들고 현재 터미널로 스트리밍합니다. 원격 셸에서 `exit`(또는 `Ctrl+D`)을 입력하면 세션이 끝납니다.

세션은 서버에 존재합니다. 연결이 끊겨도 — 네트워크 끊김, 창 닫기, `vtermc` 종료 — 세션은 계속 실행되며 나중에 `vtermc -s`로 다시 연결할 수 있습니다.

- `vtermc -p <id>` — 특정 터미널 프로필로 시작(프로필 목록은 Web UI의 `Profiles` 섹션에 있습니다);
- `vtermc -p` — 프로필 ID 없음: 서버의 프로필을 대화형으로 나열합니다;
- `vtermc -w <dir>` — 새 세션의 작업 디렉터리를 설정합니다.

stdin이 터미널이 아닐 때(파이프나 파일) `vtermc`는 배치 모드로 전환합니다: stdin이 세션에 공급되고 세션이 닫힐 때까지 출력이 스트리밍됩니다. 스크립트나 에이전트에서 유용합니다:

```sh
echo "make test" | vtermc
```

## 서버 연결

- `--server wss://host:port` — 로컬 대신 특정 서버에 연결(IPv6 주소는 대괄호로: `wss://[fd00::1]:5153`). `VTERM_SERVER` 환경 변수로 같은 기본값을 설정할 수 있습니다;
- 핸드셰이크는 서버의 시크릿 키로 응답됩니다: `-a`/`-s`로 추가된 서버는 `client.json`에 키를 가집니다. 로컬 서버는 `TERMINAL_GUI_SECRET_KEY` 환경 변수(또는 `server.json`의 평문 키)로 폴백합니다.

## 서버 추가

`-a`는 서버를 `client.json`에 저장하고 종료합니다. 터미널은 열지 않습니다. 두 가지 방법이 있습니다:

- `vtermc -a` — 대화형 프롬프트: `Server name`, `Server address`, `Secret key`. 주소는 `wss://host:port`여야 합니다(잘못된 입력은 다시 묻고, 빈 주소는 취소). 빈 이름은 주소로 폴백합니다;
- `vtermc -a "vtshare1:..."` — 암호화된 공유 문자열 붙여넣기. Web UI `Server` 섹션에서 페어링 `QR Code`를 클릭하면 클립보드에 복사됩니다. `vtermc`가 복호화해 이름, 주소, 시크릿 키를 자동으로 채웁니다.

알아두면 좋은 세부 사항:

- 기존 주소를 다시 추가하면 항목이 복제되지 않고 그 자리에서 갱신됩니다(id는 유지);
- 로컬 주소는 거부됩니다 — `Local`은 암묵적으로 항상 사용 가능하며 항목이 필요 없습니다;
- 항목은 데스크톱 클라이언트의 Server Manager에도 표시됩니다.

## 서버와 세션 선택

`vtermc -s`는 두 개의 전체 화면 픽커를 엽니다(대화형 터미널 필요):

1. **서버 목록** — 처음은 `Local`, 이어 관리되는 모든 서버가 `Name  (address)` 형식으로. 화살표 키 또는 `j`/`k`로 이동, 숫자로 점프, `Enter` 선택, `q` 취소;
2. **터미널 목록** — 처음은 `New Terminal`, 이어 해당 서버의 라이브 세션(작업 디렉터리, 제목, 소유 서버 포함). 기존 세션에 연결하거나 새로 만듭니다(이후 일반 `vtermc` 실행처럼 `-p` / `-w`가 적용됩니다).

연결해도 세션이 중단되지 않습니다 — 연결 사이에도 서버에서 계속 실행됩니다.

### 서버 제거

서버 목록에서 관리되는 서버로 이동해 `d`를 누르면 행이 빨개지고 확인을 요청합니다. `y`(또는 `Enter`)를 누르면 `client.json`에서 항목을 삭제하고, 다른 키는 중단합니다. `Local`은 삭제할 수 없습니다.

## 환경 변수

새 세션은 셸에서 고정된 집합을 상속합니다 — `PATH`, `HOME`, `USER`, `SHELL`, `TERM`, `WT_PROFILE_ID`, `PSMODULEPATH`:

- `--env-inherit <patterns>` — 쉼표로 구분된 목록(또는 `VTERM_ENV_INHERIT` 환경 변수). `*` 와일드카드 지원;
- `-e VAR=VALUE` — 추가 변수 주입(반복 가능);
- `--no-env` — 상속 변수 없이 세션을 시작합니다.

```sh
vtermc -w ~/project -e RUST_LOG=debug --env-inherit "PATH,HOME,TERM"
```

## 전체 옵션

```text
-p, --profile [<ID>]          터미널 프로필 선택(생략 시 대화형 목록)
-w, --working-dir <DIR>       작업 디렉터리 설정
-s, --select                  서버와 기존 터미널을 대화형으로 선택
-a, --add [<SHARE>]           서버 추가(vtshare1: 공유 문자열, 또는 대화형)
    --server <URL>            서버 WebSocket URL [env: VTERM_SERVER]
-e, --env <VAR=VALUE>         추가 환경 변수
    --no-env                  환경 변수 상속 안 함
    --env-inherit <PATTERNS>  상속할 환경 변수 패턴 [env: VTERM_ENV_INHERIT]
```
