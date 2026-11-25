Git 패널의 `CHANGES` 영역은 일상의 커밋 루프를 다룹니다: 변경 검토, 스테이징, 버리기, 스태시, 메시지 커밋.

## 스테이지와 해제

- 스테이지되지 않은 변경은 `CHANGES` 아래 나열되고, 각 파일에 상태 배지가 붙습니다 — `M` 수정, `A` 추가, `D` 삭제, `?` 추적 안 됨, `!` 충돌(서브모듈 행에는 추가 `SUB` 태그);
- 행에 마우스를 올리면 인라인 동작 버튼이 나타납니다: `+`는 스테이지 안 된 파일을 스테이지, `−`는 스테이지된 것을 해제, 스테이지 안 된 행의 `×`는 버립니다. 섹션 헤더의 `Stage All Changes` / `Unstage All Changes` 버튼이 한 번에 처리합니다;
- `Ctrl`-클릭으로 다중 선택에 토글, `Shift`-클릭으로 범위 선택 — 동작 버튼이 선택한 모든 파일에 적용됩니다;
- 파일 이름을 클릭하면 diff가 열려 결정 전에 변경을 검토할 수 있습니다;
- 파일의 마우스 오른쪽 클릭은 `View File History`, `Stash This File`, `Copy Full Path` / `Copy Relative Path`, `Reveal in File Explorer`, 터미널을 파일의 폴더로 점프시키는 `cd` 바로 가기를 제공합니다.

![STAGED CHANGES 섹션 — 행별 Unstage 버튼이 있는 스테이지된 파일](images/git-staging-staged.jpg)

## 커밋

1. 포함할 것을 스테이지;
2. 커밋 상자에 메시지 입력(`Ctrl+Enter` 커밋) 또는 AI 버튼으로 생성(LLM 지원 참조);
3. `Commit` 클릭.

![커밋 상자 — AI 생성 버튼이 있는 메시지 입력](images/git-staging-commit.jpg)

커밋 후, 기록 헤더(커밋 기록 위의 브랜치 이름 있는 막대)에 마우스를 올리면 오른쪽에 `Fetch`, `Pull`, `Push` 버튼이 나타납니다. `Push`는 현재 브랜치를 즉시 업스트림으로 보냅니다. `Force with lease` 같은 옵션은 브랜치 목록의 푸시 대화상자를 대신 사용하세요(푸시, 체크아웃 등 참조).

![기록 헤더에 마우스를 올리면 Fetch, Pull, Push 버튼이 나타남](images/git-staging-push.jpg)

## 스태시

- `CHANGES` 헤더의 보관 버튼이 `Stash All Changes` 대화상자를 열고, 선택적 메시지가 스태시를 설명합니다;
- 스테이지 안 된 선택 파일의 마우스 오른쪽 클릭은 `Stash This File`(다중 선택이면 `Stash <n> Selected Files`)을 제공해 그 경로만 스태시합니다 — 서브모듈 행은 제외;
- 스태시는 `STASHES` 아래 `WIP on <branch>` 항목으로 모입니다. 펼치면 담은 파일이 나열되고, 파일을 클릭하면 diff가 표시됩니다;
- 스태시 행에 마우스를 올리면 `Apply`(복원하고 유지), `Pop`(적용하고 제거), `Drop`(적용 없이 제거)이 나타납니다 — `Pop`과 `Drop`은 먼저 확인을 요청하고, 마우스 오른쪽 클릭 메뉴에는 `View Diff`도 있습니다;
- 스태시 목록이 비어 있는 동안 `STASHES` 섹션은 숨겨져 있습니다.

![선택적 메시지가 있는 Stash All Changes 대화상자](images/git-staging-stash-all.jpg)

![STASHES 섹션 — 스태시를 펼쳐 파일과 Apply, Pop, Drop 버튼](images/git-staging-stashes.jpg)

![Pop은 확인 후 스태시를 적용하고 목록에서 제거](images/git-staging-stash-pop.jpg)

## 버리기

스테이지 안 된 파일의 `Discard Changes`는 그 안의 커밋 안 된 변경을 모두 버립니다 — 확인 대화상자가 있으며 이 동작은 **되돌릴 수 없습니다**.

> 커밋은 스테이지된 내용만 포함합니다. 스테이지 안 된 변경은 작업 트리에 남습니다.
