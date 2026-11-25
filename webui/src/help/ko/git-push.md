브랜치 목록의 마우스 오른쪽 클릭 메뉴에 푸시, 체크아웃, 일상적 브랜치 작업이 모여 있습니다.

## 푸시

브랜치 마우스 오른쪽 클릭 → `Push '{branch}'...`로 푸시 대화상자를 엽니다:

![브랜치 마우스 오른쪽 클릭 — Push 항목](images/git-push-contextmenu.jpg)

- `local → remote` 대상 매핑을 보여 줍니다. 어느 쪽 이름이든 바꿀 수 있습니다(기존 원격 브랜치에 다른 이름으로 푸시하면 확인을 요구);
- 옵션: `Push all branches`, `Set upstream`(첫 푸시에서 업스트림 바인딩), `Push tags`, `Dry run`;
- 역사를 다시 쓰는 `Force`와 더 안전한 `Force with lease`(다른 사람이 푸시했으면 거부 — 권장). 리베이스 후 푸시처럼 역사를 다시 쓰는 작업에는 이들 중 하나가 필요합니다;
- 대화상자에 강제 푸시 위험 힌트가 표시됩니다.

![푸시 대화상자 — local → remote 대상과 옵션 체크박스](images/git-push-dialog.jpg)

## 체크아웃

브랜치 마우스 오른쪽 클릭 → `Checkout`으로 브랜치 전환 — 로컬 브랜치는 직접 체크아웃되고, 원격 브랜치나 태그는 체크아웃 대화상자를 엽니다:

![브랜치 마우스 오른쪽 클릭 — Checkout 항목](images/git-checkout-contextmenu.jpg)

- `New branch name` — 만들 로컬 브랜치. ref에서 미리 채워집니다;
- `Create new local branch`(`-B`), `Track remote as upstream`(`--track`, 원격 브랜치만), `Force`(`-f`), `Merge`(`--merge`).

![체크아웃 대화상자 — 브랜치 이름과 옵션](images/git-checkout-dialog.jpg)

세션의 작업 디렉터리는 전환을 따르고, 터미널과 Git 패널은 즉시 새 브랜치를 반영합니다.

## Detached HEAD

단순 커밋을 체크아웃하면(기록을 훑보다 커밋을 체크아웃하는 등) HEAD가 분리됩니다. 다음 커밋 때 `Detached HEAD` 대화상자가 열립니다: 커밋할 기존 브랜치를 고르거나, 그 자리에서 `+ Create new branch…`, 또는 지금은 `Skip`.

## 그 외 작업

- **스태시** — 변경 헤더에서 모든 변경을 스택에 스태시(메시지 선택). `STASHES` 목록은 `Pop`(적용하고 제거)과 `Drop`(버림, 불가역)을 지원합니다;
- **브랜치/워크트리 삭제** — 항목의 마우스 오른쪽 클릭 메뉴로. 둘 다 불가역 동작 확인 포함;
- **기록과 되돌리기** — `HISTORY`가 커밋을 나열합니다. 마우스 오른쪽 클릭으로 파일 기록, 작업 트리와의 diff, 또는 리비전으로 되돌리기;
- **복사** — 브랜치 이름과 커밋 해시를 원클릭 복사. 에이전트나 동료에게 붙여 넣기에 편합니다.
