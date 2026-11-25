파일 탐색기는 서버 쪽 파일 시스템을 다룹니다. 왼쪽 패널 상단의 도구 모음이 Recent / Explorer / Git / Debug / Search를 전환합니다. Explorer가 파일 브라우저입니다.

## Home 버튼: 터미널 cwd로 점프

도구 모음의 `Home`(집) 버튼을 클릭:

- `Explorer` 보기로 전환합니다;
- **활성 터미널의 작업 디렉터리**(cwd)로 점프합니다;
- 그 디렉터리를 트리로 고정하고 검색 상자를 비웁니다 — 항상 "터미널이 있는 곳에서" 시작합니다.

![일반 모드의 파일 탐색기 — Home, 이동 경로(Breadcrumb), 평면 목록](images/files-explorer.jpg)

## 트리 모드와 검색

- **트리로 고정**: 이동 경로 오른쪽 끝의 핀 버튼이 평면 목록과 트리 모드를 전환합니다. 트리 모드에서 폴더는 접을 수 있는 디렉터리 트리로 펼쳐집니다(Home 점프는 자동으로 고정);
- **검색**: 상단의 검색 상자가 입력에 맞춰 현재 목록을 이름으로 걸러 줍니다. Recent 컨텍스트 메뉴의 `Search in Folder`는 디렉터리 전체에 재귀 검색을 실행합니다.

![트리 모드의 파일 탐색기 — 고정된 루트가 접을 수 있는 디렉터리 트리로 펼쳐짐](images/files-explorer-tree.jpg)

## 기본 작업

파일이나 폴더를 마우스 오른쪽 클릭:

- **만들기**: `New Folder` / `New File`;
- **편집**: `Rename`; `Copy` / `Cut` / `Paste`(디렉터리 간 동작);
- **삭제**: `Delete`, 확인 대화상자 포함;
- **터미널 연동**: `Open in Terminal`은 그 디렉터리에서 새 터미널 탭을 엽니다. `cd to Directory`는 현재 터미널을 그곳으로 cd합니다;
- **OS 연동**: `Reveal in File Explorer`는 로컬 디렉터리를 시스템 파일 관리자에서 보여 줍니다;
- **경로**: `Copy Full Path` / `Copy Relative Path`;
- **공유**: `Host with HTTP server`는 디렉터리를 HTTP 파일 서비스로 공개합니다(HTTP 서버로 공유 참조). `Open in App`은 서버에서 감지된 에이전트 앱으로 엽니다.

모든 작업은 서버 쪽 파일 시스템에 작용합니다. 원격 세션도 로컬와 똑같이 동작합니다.
