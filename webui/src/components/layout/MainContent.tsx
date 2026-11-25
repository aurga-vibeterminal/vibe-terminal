import { useConfig } from '@/context/config-context';
import { ServerSection } from '@/components/sections/ServerSection';
import { CertificatesSection } from '@/components/sections/CertificatesSection';
import { RemoteDesktopSection } from '@/components/sections/RemoteDesktopSection';
import { TerminalSesion } from '@/components/sections/TerminalSesion';
import { PortForwardingSection } from '@/components/sections/PortForwardingSection';
import { AddonSection } from '@/components/sections/AddonSection';
import { LlmSection } from '@/components/sections/LlmSection';
import { LlmStatsSection } from '@/components/sections/LlmStatsSection';
import { VoiceSection } from '@/components/sections/VoiceSection';
import { DdnsSection } from '@/components/sections/DdnsSection';
import { UartSection } from '@/components/sections/UartSection';
import { EnvironmentSection } from '@/components/sections/EnvironmentSection';
import { McpSection } from '@/components/sections/McpSection';
import { HelpSection } from '@/components/sections/HelpSection';
import { StaticHttpSection } from '@/components/sections/StaticHttpSection';

export function MainContent() {
  const { activeSection } = useConfig();

  const sections = {
    server: ServerSection,
    certificates: CertificatesSection,
    'remote-desktop': RemoteDesktopSection,
    terminal: TerminalSesion,
    'port-forwarding': PortForwardingSection,
    addons: AddonSection,
    llm: LlmSection,
    'llm-stats': LlmStatsSection,
    voice: VoiceSection,
    ddns: DdnsSection,
    uart: UartSection,
    environment: EnvironmentSection,
    mcp: McpSection,
    help: HelpSection,
    'static-http': StaticHttpSection,
  };

  const Section = sections[activeSection];

  return (
    <main className="flex-1 overflow-auto bg-golden-cream">
      <div className={`mx-auto p-6 ${activeSection === 'llm-stats' || activeSection === 'help' ? 'max-w-5xl' : 'max-w-3xl'}`}>
        <Section />
      </div>
    </main>
  );
}
