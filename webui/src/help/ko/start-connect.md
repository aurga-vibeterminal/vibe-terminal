Web UI의 페어링 QR 코드로 클라이언트를 이 서버에 연결하거나, 직접 입력할 수 있습니다. 클라이언트에 필요한 모든 정보(주소, 포트, 시크릿 키)는 Server 섹션에 표시됩니다.

## 서버 준비

- `Server` 섹션을 열고 `Secret Key`가 설정되어 있는지 확인합니다. 설정되지 않았다면 주사위 버튼으로 생성합니다. 키를 다시 생성하면 페어링된 모든 클라이언트가 무효화되어 다시 페어링해야 합니다.
- `QR Code` 블록을 펼치고 다른 머신이 이 서버에 접근할 때 사용할 `Network Interface`와 `IP Version`(IPv4 / IPv6)을 선택합니다.
- DDNS 도메인이 설정되어 있다면 `Connection Type`을 `DDNS Domain`으로 전환해 QR 코드에 IP 주소 대신 도메인을 포함시킬 수 있습니다.
- `Pairing Preview`는 클라이언트가 실제로 연결할 연결 URL(`wss://host:port`)을 보여줍니다.
- `QR code`를 클릭하면 암호화된 공유 문자열이 클립보드에 복사됩니다. 클라이언트는 이를 원탭 자동 입력에 사용합니다.

## 데스크톱 클라이언트

- 제목 표시줄의 서버 이름 버튼을 클릭하고 메뉴에서 `Servers`를 선택해 Server Manager 대화상자를 엽니다.

![데스크톱 클라이언트 — 서버 이름 버튼으로 열리는 서버 메뉴](images/start-connect-server-menu.jpg)

- 서버 목록 옆의 `+` 버튼을 클릭해 Add Server 폼을 시작합니다.
- 공유 문자열이 클립보드에 있으면 폼 오른쪽 위에 클립보드 아이콘이 나타납니다. 클릭하면 문자열을 복호화해 `Title`, `Address`, `Secret Key`를 자동 입력합니다.
- 없다면 직접 입력합니다: 표시 이름, 연결 URL(`wss://host:port`, IPv6 주소는 대괄호), 시크릿 키.
- `Save`를 클릭한 뒤 새 서버를 선택해 세션을 엽니다.

![데스크톱 클라이언트 — Add Server 폼이 있는 Server Manager](images/start-connect-add-server.jpg)
