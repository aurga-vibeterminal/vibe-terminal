모든 것은 Web UI의 `Remote Desktop` 섹션에 있습니다. 페이지는 서버의 플랫폼에 적용되는 설정을 보여 줍니다.

## 메인 스위치

`Enable Remote Desktop` — 켜면 연결된 원격 클라이언트가 이 머신의 데스크톱을 보고 입력을 보낼 수 있습니다.

## 플랫폼 설정

**Windows**

- `Windows Service` — 화면 서버를 자동 시작 시스템 서비스로 등록해, 로그인 세션 없이도 재부팅을 견딥니다. `Install` / `Uninstall`은 서버에서 UAC 프롬프트를 일으킵니다(대기 중 버튼에 진행 상태 표시). 상태 배지가 실행 중 / 설치됨 / 미설치를 구분합니다.
- `Headless machines (no monitor)` — 연결된(또는 깨어 있는) 디스플레이가 없으면 Windows는 캡처 가능한 데스크톱을 유지하지 않습니다. 페이지는 오픈소스 Virtual Display Driver 다운로드(ARM64 / x64 자동 선택) 링크를 제공합니다. 설치하면 가상 데스크톱이 항상 존재합니다.

![Web UI — Windows의 Remote Desktop 섹션(서비스와 헤드리스 카드)](images/remote-desktop-server-windows.jpg)

**Linux**

- `Unlock on Connect` — 스트림이 시작되면 잠긴 세션을 자동으로 잠금 해제합니다(환경이 지원할 때);
- 화면 캡처는 Portal을 거칩니다. `Screen Recording Permission` 카드의 `Reset Permission`이 저장된 Portal 동의를 지워 다른 화면을 고를 수 있게 합니다.
- `Auto-start on Login`(`Server` 섹션) — 디스플레이 매니저가 있는 호스트에서는 서버가 그래픽 세션과 함께 시작하므로 캡처가 전체 데스크톱 환경을 물려받고, 헤드리스 호스트에서는 로그인 전 부팅 시 시작합니다. 로컬에서 아무도 로그인하지 않은 상태에서 데스크톱에 도달하려면 디스플레이 매니저(GDM, SDDM 등)에서 자동 로그인도 활성화하세요. 보안을 위해 자동 화면 잠금은 켜 둘 수 있습니다 — 잠긴 상태에서도 세션과 서버는 계속 실행되며, `Unlock on Connect`(또는 `loginctl unlock-session`)가 원격으로 세션을 엽니다.
![Web UI — 화면 녹화 권한 카드(Linux)](images/remote-desktop-server-permission.jpg)


**macOS**

- `Screen Recording Permission` — 시스템은 캡처에 명시적 승인을 요구합니다. `Grant Permission`으로 시스템 대화상자를 엽니다. `Reset Permission`은 막힌 상태에서 복구합니다.

## 화면 스트리밍 도구(Android 지원)

`Android SDK directory`와 `Android Studio directory`가 adb와 에뮬레이터를 찾습니다. 자동 감지로 두려면 비워 둡니다. 설정하면 페이지에 해석된 경로와 그 출처(auto / env / override)가 표시됩니다. `Reset to Auto-Detect`로 기본값을 복원합니다. 이것들이 무엇을 가능하게 하는지는 Android 기기를 참고하세요.

## 스트림 부호화

- `Encoder` — Auto 또는 특정 백엔드. 서버는 사용 가능한 인코더를 나열하고, 불가한 것은 이유와 함께 회색으로 만듭니다;
- `Frame rate` — 60 / 30 / 15 fps;
- `Max resolution` — 캡처 쪽 상한(4K / 2.5K / 2K);
- `Bitrate` — `Auto`는 해상도에 따라 면적 비례(1080p까지 기본률, 4K에서 약 4배, 40 Mbps 상한). 고정값은 실행 중 스트림에 즉시 적용되고, 인코더 / fps / 해상도는 이후 시작되는 스트림에 적용됩니다;
- 카드 하단에 현재 유효한 인코더 / fps / 해상도 / 비트레이트가 표시됩니다.

![Web UI — 스트림 부호화 설정](images/remote-desktop-server-stream.jpg)

`Save`를 클릭해 변경을 적용합니다.
