Windows에서 WSL을 쓰는 방법은 두 가지가 있습니다. 가벼운 방법은 WSL 셸을 터미널 프로필로 추가하는 것이고, 완전한 방법은 WSL 안에 vterm-server를 설치하는 것입니다. 전자는 셸만 주고, 후자는 파일 관리, Git 패널, MCP 등 모든 것을 엽니다.

## 가벼운 방법: WSL 터미널 프로필

Windows에서는 **WSL 터미널 프로필이 자동으로 로드됩니다**. 시작 시 서버가 `wsl.exe`를 감지해 설치된 배포판을 나열하고, 배포판마다 프로필 항목을 프로필 목록에 추가합니다(예: `Ubuntu (WSL)`; 나열된 배포판이 없으면 일반 `WSL` 항목이 나타남).

사용하려면 클라이언트의 제목 표시줄 프로필 메뉴에서 WSL 항목을 고르기만 하면 됩니다 — 배포판이 실행되고 바로 그 터미널로 떨어집니다. 로컬을 여는 것과 같습니다.

제한: 이런 세션은 Windows 쪽 서버에서 실행됩니다 — 클라이언트의 **파일 탐색기, Git 패널, 환경 변수 등은 동작하지 않습니다**(Windows 파일 시스템 의미론으로 동작하며 WSL 안의 Linux 파일 시스템을 볼 수 없습니다).

![터미널 프로필 메뉴 — 자동 나열된 WSL 배포판](images/terminal-wsl-profile.jpg)

## 완전한 방법: WSL 안에 vterm-server 설치

WSL을 자체 Linux 머신으로 취급하고 헤드리스 vterm-server를 설치합니다. 웹사이트가 원라인 스크립트를 제공합니다 — WSL 터미널에 붙여넣으세요:

```bash
curl -fsSL https://vterminal.dev/install.sh | bash
```

스크립트는 자동으로:

1. WSL/헤드리스 환경을 감지하고 **헤드리스** vterm-server를 `~/.local/vterm`에 내려받습니다;
2. **systemd 사용자 서비스**를 설치하고 즉시 시작합니다 — 매 로그인(WSL 시작 포함)마다 실행되며 수동 시작이 필요 없습니다;
3. `~/.local/vterm`을 PATH에 추가합니다.

> WSL에서 systemd가 꺼져 있으면, 스크립트는 `/etc/wsl.conf`에 `[boot]` `systemd=true`를 추가하고, Windows 쪽에서 `wsl --shutdown`을 실행하고, 설치 프로그램을 다시 실행하라고 안내합니다.

## 설정 페이지 열기

WSL 안에서:

```bash
vterm-server --open
```

이 명령은 Web UI 주소(`http://localhost:<config-port>/`)를 출력하고 브라우저 실행을 시도합니다(WSL에서는 `wslview` 경유, Windows 기본 브라우저). WSL2의 localhost 포워딩 덕분에 **Windows 브라우저에서 같은 주소를 여는 것**도 동일하게 동작합니다. 서버가 실행 중이면 출력되는 주소에 일회성 로그인 토큰(`?ticket=...`, 로컬 소켓으로 발급)이 붙어 페이지를 열기만 하면 로그인됩니다.

## (선택) WSL 서버를 호스트 릴레이에 공유

Windows 머신도 vterm-server를 실행 중이라면(릴레이 역할), WSL 서버를 그 아래에 붙여 클라이언트가 하나의 로컬 연결로 두 "머신"을 모두 보게 할 수 있습니다:

1. Windows 서버의 Web UI에서 QR 코드를 클릭해 공유 문자열을 복사;
2. WSL 서버의 Web UI(이전 절의 주소)를 열고, `Relay Address` 필드 옆에 나타나는 붙여넣기 버튼을 클릭해 주소와 키를 자동 입력한 뒤 `Apply`;
3. Windows의 vterm-server가 실행되는 동안 클라이언트는 로컬 릴레이 아래에서 WSL 서버의 터미널, 파일, Git을 봅니다.

> WSL2의 기본 NAT 네트워킹에서 WSL은 WSL 쪽에서 본 호스트 주소(기본 게이트웨이 — `cat /etc/resolv.conf`의 nameserver)로 Windows 호스트에 도달합니다. 미러 네트워킹에서는 `localhost`가 동작합니다. 릴레이 세부 사항은 Relay 장을 참고하세요.
