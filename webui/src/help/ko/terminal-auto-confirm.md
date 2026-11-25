서버 측 터미널 분석기는 터미널 출력을 지켜봅니다. 명령 프롬프트를 감지하고, 작업 완료를 인식하며, LLM으로 터미널이 보여 주는 내용을 분류합니다. `Auto Confirm`은 가장 많이 쓰이는 기능으로, 터미널이 평범한 확인 프롬프트(`Proceed? [Y/n]` 등)에서 멈췄을 때 Enter를 대신 보내 주어 긴 작업이 질문에 걸려 멈추지 않게 합니다.

## 설정

Web UI의 `Terminal Analysis` 섹션:

- `Terminal observer` — 서버 측 분석의 마스터 스위치;
- `Auto confirm` — **평범한 확인**으로 분류된 프롬프트에만 Enter를 보냅니다. 다른 질문에 절대 맹목적으로 답하지 않습니다;
- `Analysis delay (seconds)` — 에이전트가 유휴 상태가 된 뒤 분석까지 기다리는 시간. 작업 사이의 불필요한 LLM 호출을 피합니다;
- `Confirmation grace (seconds)` — 확인이 감지된 뒤 사용자가 개입할 수 있는 유예 시간;
- `Notify on completion` — 출력이 작업 완료로 분류되면 이벤트를 내보냅니다(알림에 연결하세요).

> 페이지 안내: 에이전트 자체의 자동 모드가 켜져 있다면, 두 자동화가 서로 겹쳐 답하지 않도록 클라이언트에서 `Auto confirm`을 끄기를 권장합니다.

## LLM 의존성

분류는 LLM이 수행합니다. `LLM` 섹션에서 `Provider`를 고르고 자격 증명을 입력하세요(터미널 분류, Git 커밋 메시지, 채팅 도우미 등 AI 기능이 공유). LLM이 없어도 분석기는 정확도가 낮아진 채 동작합니다 — 이 경우 자동 확인은 꺼 두세요.

![Terminal Analysis 설정 페이지](images/terminal-auto-confirm.jpg)

![LLM 설정 페이지 — 공급자를 고르고 자격 증명 입력](images/terminal-auto-confirm-llm.jpg)
