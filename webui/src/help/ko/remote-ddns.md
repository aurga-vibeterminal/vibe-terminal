가정과 사무실 회선은 보통 예고 없이 바뀌는 동적 공인 IP를 가집니다. DDNS는 고정 도메인을 머신의 현재 IP로 계속 향하게 합니다 — 클라이언트와 다른 서버는 주소가 아니라 이름만 기억하면 됩니다.

## 동작 방식

![DDNS 아키텍처 — 관리자가 주기적으로 로컬 IP 감지, 애드온이 공급자 API로 DNS 레코드 갱신, 클라이언트는 도메인으로 연결](diagram/ddns)

전체 파이프라인은 서버 백그라운드에서 자동 실행됩니다:

1. **DDNS 관리자**는 고정 주기(기본 5분마다, `TERMINAL_SERVER_DDNS_POLL_INTERVAL_SECS` 환경 변수로 조절 가능)로 새로 고칩니다. 설정 저장은 즉시 새로 고침을 유발하고, 서비스가 비활성인 동안엔 아무것도 실행되지 않습니다;
2. **로컬 IP 감지**는 선택된 네트워크 인터페이스를 먼저 읽고, 필요하면 경로 기반 탐색으로 폴백하며, 결과를 `IP Family` 설정(IPv4 / IPv6 / 듀얼)으로 걸러 줍니다;
3. **DDNS 애드온** — 샌드박스된 JS 스크립트 — 이 복호화된 설정과 현재 IP를 받아 HTTP로 DNS 공급자의 API를 호출합니다. 예를 들어 내장 Cloudflare 애드온은 도메인으로 존을 찾아 A / AAAA 레코드를 현재 IP로 다시 쓰고, 주소군이 사라지면 낡은 레코드를 지웁니다;
4. **클라이언트**는 도메인을 최신 IP로 풀어 이름으로 연결합니다.

## 설정

Web UI의 `DDNS` 섹션을 엽니다:

1. `Enable`을 켭니다. 다 채우는 동안 서비스를 꺼 둬도 됩니다 — 비활성 서비스는 설정을 저장할 뿐 실행하지 않습니다;
2. `Provider` 아래에서 공급자를 고릅니다. `Cloudflare DDNS`가 내장되어 있습니다;
3. 공급자 필드를 채웁니다(Cloudflare 예):
   - `Email` — Cloudflare 계정 이메일;
   - `API Token` — Zone → DNS → Edit 권한이 있는 토큰;
   - `Domain` — 존 루트 도메인, 예: `example.com`;
   - `Subdomain` — 존 내의 레코드 이름. `sub.example.com`이면 `sub`. 존 루트에는 비워 둡니다;
4. `Network Interface & IP`에서 `Interface`와 `IP Family`를 고릅니다. `Dual Stack`은 인터페이스에 IPv4와 IPv6 주소가 모두 있어야 합니다;
5. `Save`를 클릭한 뒤 `Sync Now`로 검증합니다. `Sync Status`는 마지막으로 푸시된 IP와 동기 시간을, 실패 시 애드온의 오류 메시지도 보여 줍니다.

## 보안

공급자 자격 증명(API 토큰)은 저장 전에 암호화되고(`enc-v2:`), 애드온 실행 중에만 메모리에서 복호화됩니다. 애드온은 외부로 나가는 HTTP만 능력으로 하는 샌드박스 JS 런타임에서 실행됩니다.

## 공급자 확장

공급자는 애드온입니다: JSON 매니페스트(Web UI가 폼으로 바꾸는 설정 필드 선언) + `run()`을 노출하는 JS 스크립트. 내장 `cloudflare-ddns`가 참고 구현입니다 — 사용자 지정 공급자는 `Addons` 섹션에서 zip으로 업로드할 수 있습니다.

## 다른 기능과의 연계

- 도메인이 풀리면 `Server` 섹션의 QR 코드가 `Connection Type`을 `DDNS Domain`으로 전환할 수 있어, 클라이언트는 도메인 연결을 스캔합니다;
- 다른 서버는 `Relay Address`에 `wss://your-domain:port`를 넣어 도메인 위로 릴레이 사슬을 굴릴 수 있습니다(Relay 장 참조).
