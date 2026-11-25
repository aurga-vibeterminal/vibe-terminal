import type { HelpTocNode } from '../types';

export default [
  {
    id: 'start',
    title: 'เริ่มต้นใช้งาน',
    icon: 'rocket',
    children: [
      { id: 'start-overview', title: 'ภาพรวม', file: 'start-overview.md' },
      { id: 'start-connect', title: 'การเชื่อมต่อไคลเอนต์', file: 'start-connect.md' },
      { id: 'start-webui', title: 'การใช้งาน Web UI', file: 'start-webui.md' },
    ],
  },
  {
    id: 'relay',
    title: 'รีเลย์',
    icon: 'network',
    children: [
      { id: 'relay-overview', title: 'ภาพรวม', file: 'relay-overview.md' },
      { id: 'relay-setup', title: 'คู่มือการตั้งค่า', file: 'relay-setup.md' },
    ],
  },
  {
    id: 'certificates',
    title: 'ใบรับรองและ TLS',
    icon: 'shield',
    children: [
      { id: 'cert-overview', title: 'ภาพรวม', file: 'cert-overview.md' },
      { id: 'cert-sans', title: 'การจัดการโดเมนและ SAN', file: 'cert-sans.md' },
      { id: 'cert-client', title: 'การตรวจสอบแบบเข้มงวดบนไคลเอนต์', file: 'cert-client.md' },
      { id: 'cert-relay', title: 'การตรวจสอบแบบเข้มงวดบนรีเลย์', file: 'cert-relay.md' },
    ],
  },
  {
    id: 'terminal',
    title: 'เทอร์มินัล',
    icon: 'terminal',
    children: [
      { id: 'terminal-usage', title: 'การใช้งานเทอร์มินัล', file: 'terminal-usage.md' },
      { id: 'terminal-cli', title: 'ไคลเอนต์บรรทัดคำสั่ง', file: 'terminal-cli.md' },
      { id: 'terminal-auto-confirm', title: 'ยืนยันอัตโนมัติ', file: 'terminal-auto-confirm.md' },
      { id: 'terminal-wsl', title: 'WSL บน Windows', file: 'terminal-wsl.md' },
      { id: 'terminal-commands', title: 'คำสั่งด่วน', file: 'terminal-commands.md' },
      { id: 'terminal-env', title: 'ตัวแปรสภาพแวดล้อม', file: 'terminal-env.md' },
      { id: 'terminal-uart', title: 'พอร์ตอนุกรม UART', file: 'terminal-uart.md' },
      { id: 'terminal-mcp', title: 'เทอร์มินัล MCP', file: 'terminal-mcp.md' },
      { id: 'terminal-context', title: 'การเพิ่มบริบทให้เอเจนต์', file: 'terminal-context.md' },
      { id: 'terminal-images', title: 'ภาพหน้าจอและรูปภาพ', file: 'terminal-images.md' },
      { id: 'terminal-agent-history', title: 'ประวัติเอเจนต์โค้ด', file: 'terminal-agent-history.md' },
    ],
  },
  {
    id: 'files',
    title: 'ไฟล์',
    icon: 'folder',
    children: [
      { id: 'files-overview', title: 'ภาพรวม', file: 'files-overview.md' },
      { id: 'files-explorer', title: 'ตัวสำรวจไฟล์', file: 'files-explorer.md' },
      { id: 'files-transfer', title: 'อัปโหลดและดาวน์โหลด', file: 'files-transfer.md' },
      { id: 'files-http', title: 'เผยแพร่ด้วยเซิร์ฟเวอร์ HTTP', file: 'files-http.md' },
    ],
  },
  {
    id: 'git',
    title: 'Git',
    icon: 'git-branch',
    children: [
      { id: 'git-overview', title: 'ภาพรวม', file: 'git-overview.md' },
      { id: 'git-llm', title: 'ผู้ช่วย LLM', file: 'git-llm.md' },
      { id: 'git-staging', title: 'การ Stage และ Commit', file: 'git-staging.md' },
      { id: 'git-worktree', title: 'Worktree', file: 'git-worktree.md' },
      { id: 'git-rebase', title: 'Rebase แบบโต้ตอบ', file: 'git-rebase.md' },
      { id: 'git-conflicts', title: 'การแก้ความขัดแย้ง', file: 'git-conflicts.md' },
      { id: 'git-push', title: 'Push, Checkout และอื่น ๆ', file: 'git-push.md' },
    ],
  },
  {
    id: 'remote-desktop',
    title: 'เดสก์ท็อประยะไกลและโปรแกรมจำลอง',
    icon: 'monitor',
    children: [
      { id: 'rd-overview', title: 'ภาพรวม', file: 'remote-desktop-overview.md' },
      { id: 'rd-server', title: 'การตั้งค่าเซิร์ฟเวอร์', file: 'remote-desktop-server.md' },
      { id: 'rd-android', title: 'อุปกรณ์ Android', file: 'remote-desktop-android.md' },
      { id: 'rd-client', title: 'การใช้งานไคลเอนต์', file: 'remote-desktop-client.md' },
      { id: 'rd-desktop-client', title: 'ไคลเอนต์เดสก์ท็อป', file: 'remote-desktop-desktop-client.md' },
    ],
  },
  {
    id: 'voice',
    title: 'การป้อนข้อมูลด้วยเสียง',
    icon: 'mic',
    children: [
      { id: 'voice-server', title: 'การตั้งค่าเซิร์ฟเวอร์', file: 'voice-server.md' },
      { id: 'voice-client', title: 'การใช้งานไคลเอนต์', file: 'voice-client.md' },
    ],
  },
  {
    id: 'remote',
    title: 'การเข้าถึงระยะไกล',
    icon: 'globe',
    children: [
      { id: 'remote-ddns', title: 'DDNS', file: 'remote-ddns.md' },
      { id: 'port-forwarding', title: 'การส่งต่อพอร์ต', file: 'port-forwarding.md' },
    ],
  },
  {
    id: 'ai',
    title: 'ฟีเจอร์ AI',
    icon: 'brain',
    children: [
      { id: 'ai-llm', title: 'LLM', file: 'ai-llm.md' },
      { id: 'ai-mcp', title: 'MCP', file: 'ai-mcp.md' },
    ],
  },
  {
    id: 'translation',
    title: 'การแปลภาษา',
    icon: 'languages',
    children: [
      { id: 'translation-overview', title: 'ภาพรวม', file: 'translation-overview.md' },
      { id: 'translation-server', title: 'การตั้งค่าเซิร์ฟเวอร์', file: 'translation-server.md' },
      { id: 'translation-client', title: 'การใช้งานไคลเอนต์', file: 'translation-client.md' },
    ],
  },
  {
    id: 'addon',
    title: 'แอดออน',
    icon: 'puzzle',
    children: [
      { id: 'addon-overview', title: 'ภาพรวม', file: 'addon-overview.md' },
      { id: 'addon-dev', title: 'ปรับแต่งและพัฒนา', file: 'addon-dev.md' },
    ],
  },
  { id: 'faq', title: 'คำถามที่พบบ่อย', icon: 'circle-help', file: 'faq.md' },
] satisfies HelpTocNode[];
