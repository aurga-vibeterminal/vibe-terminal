import type { HelpTocNode } from '../types';

export default [
  {
    id: 'start',
    title: '시작하기',
    icon: 'rocket',
    children: [
      { id: 'start-overview', title: '개요', file: 'start-overview.md' },
      { id: 'start-connect', title: '클라이언트 연결', file: 'start-connect.md' },
      { id: 'start-webui', title: 'Web UI 사용법', file: 'start-webui.md' },
    ],
  },
  {
    id: 'relay',
    title: '릴레이',
    icon: 'network',
    children: [
      { id: 'relay-overview', title: '개요', file: 'relay-overview.md' },
      { id: 'relay-setup', title: '설정 가이드', file: 'relay-setup.md' },
    ],
  },
  {
    id: 'certificates',
    title: '인증서 및 TLS',
    icon: 'shield',
    children: [
      { id: 'cert-overview', title: '개요', file: 'cert-overview.md' },
      { id: 'cert-sans', title: '도메인 및 SAN 관리', file: 'cert-sans.md' },
      { id: 'cert-client', title: '클라이언트 엄격 검증', file: 'cert-client.md' },
      { id: 'cert-relay', title: '릴레이 엄격 검증', file: 'cert-relay.md' },
    ],
  },
  {
    id: 'terminal',
    title: '터미널',
    icon: 'terminal',
    children: [
      { id: 'terminal-usage', title: '터미널 사용법', file: 'terminal-usage.md' },
      { id: 'terminal-cli', title: '커맨드라인 클라이언트', file: 'terminal-cli.md' },
      { id: 'terminal-auto-confirm', title: '자동 확인', file: 'terminal-auto-confirm.md' },
      { id: 'terminal-wsl', title: 'Windows의 WSL', file: 'terminal-wsl.md' },
      { id: 'terminal-commands', title: '빠른 명령', file: 'terminal-commands.md' },
      { id: 'terminal-env', title: '환경 변수', file: 'terminal-env.md' },
      { id: 'terminal-uart', title: 'UART 시리얼', file: 'terminal-uart.md' },
      { id: 'terminal-mcp', title: '터미널 MCP', file: 'terminal-mcp.md' },
      { id: 'terminal-context', title: '에이전트에 컨텍스트 추가', file: 'terminal-context.md' },
      { id: 'terminal-images', title: '스크린샷과 이미지', file: 'terminal-images.md' },
      { id: 'terminal-agent-history', title: '코드 에이전트 기록', file: 'terminal-agent-history.md' },
    ],
  },
  {
    id: 'files',
    title: '파일',
    icon: 'folder',
    children: [
      { id: 'files-overview', title: '개요', file: 'files-overview.md' },
      { id: 'files-explorer', title: '파일 탐색기', file: 'files-explorer.md' },
      { id: 'files-transfer', title: '업로드와 다운로드', file: 'files-transfer.md' },
      { id: 'files-http', title: 'HTTP 서버로 공유', file: 'files-http.md' },
    ],
  },
  {
    id: 'git',
    title: 'Git',
    icon: 'git-branch',
    children: [
      { id: 'git-overview', title: '개요', file: 'git-overview.md' },
      { id: 'git-llm', title: 'LLM 지원', file: 'git-llm.md' },
      { id: 'git-staging', title: '스테이징과 커밋', file: 'git-staging.md' },
      { id: 'git-worktree', title: '워크트리', file: 'git-worktree.md' },
      { id: 'git-rebase', title: '인터랙티브 리베이스', file: 'git-rebase.md' },
      { id: 'git-conflicts', title: '충돌 해결', file: 'git-conflicts.md' },
      { id: 'git-push', title: '푸시, 체크아웃 등', file: 'git-push.md' },
    ],
  },
  {
    id: 'remote-desktop',
    title: '원격 데스크톱과 시뮬레이터',
    icon: 'monitor',
    children: [
      { id: 'rd-overview', title: '개요', file: 'remote-desktop-overview.md' },
      { id: 'rd-server', title: '서버 설정', file: 'remote-desktop-server.md' },
      { id: 'rd-android', title: 'Android 기기', file: 'remote-desktop-android.md' },
      { id: 'rd-client', title: '클라이언트 사용법', file: 'remote-desktop-client.md' },
      { id: 'rd-desktop-client', title: '데스크톱 클라이언트', file: 'remote-desktop-desktop-client.md' },
    ],
  },
  {
    id: 'voice',
    title: '음성 입력',
    icon: 'mic',
    children: [
      { id: 'voice-server', title: '서버 설정', file: 'voice-server.md' },
      { id: 'voice-client', title: '클라이언트 사용법', file: 'voice-client.md' },
    ],
  },
  {
    id: 'remote',
    title: '원격 접근',
    icon: 'globe',
    children: [
      { id: 'remote-ddns', title: 'DDNS', file: 'remote-ddns.md' },
      { id: 'port-forwarding', title: '포트 포워딩', file: 'port-forwarding.md' },
    ],
  },
  {
    id: 'ai',
    title: 'AI 기능',
    icon: 'brain',
    children: [
      { id: 'ai-llm', title: 'LLM', file: 'ai-llm.md' },
      { id: 'ai-mcp', title: 'MCP', file: 'ai-mcp.md' },
    ],
  },
  {
    id: 'translation',
    title: '번역',
    icon: 'languages',
    children: [
      { id: 'translation-overview', title: '개요', file: 'translation-overview.md' },
      { id: 'translation-server', title: '서버 설정', file: 'translation-server.md' },
      { id: 'translation-client', title: '클라이언트 사용법', file: 'translation-client.md' },
    ],
  },
  {
    id: 'addon',
    title: '애드온',
    icon: 'puzzle',
    children: [
      { id: 'addon-overview', title: '개요', file: 'addon-overview.md' },
      { id: 'addon-dev', title: '사용자 지정 및 개발', file: 'addon-dev.md' },
    ],
  },
  { id: 'faq', title: 'FAQ', icon: 'circle-help', file: 'faq.md' },
] satisfies HelpTocNode[];
