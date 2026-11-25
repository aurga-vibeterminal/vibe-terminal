export type { ServerConfig, ServerNetworkInterface, IpFamily } from './server';
export type { CertificateInfo, CertificateStatus } from './certificates';
export type { DeviceConfig } from './device';
export type {
  PortForwardingConfig,
  PortForward,
  CreatePortForwardRequest,
  PortForwardShare,
} from './port-forwarding';
export type {
  DdnsConfig,
  DdnsSyncStatus,
} from './ddns';
export type {
  AddonConfigField,
  AddonConfigUpdate,
  AddonDetail,
  AddonFieldType,
  AddonInfo,
} from './addon';
export type {
  TerminalConfig,
  TerminalEnvVar,
  TerminalPatternConfig,
  TerminalPatternKind,
} from './terminal';
export type {
  EnvironmentConfig,
  EnvironmentGroup,
  EnvironmentLeaf,
  EnvironmentNode,
  EnvironmentNodeKind,
  EnvironmentVariable,
  CreateNodeRequest,
  RenameNodeRequest,
  UpdateLeafRequest,
  MoveNodeRequest,
} from './environment';
export type { CodeAnalysisConfig, TranslationConfig, TerminalLlmConfig, GitLlmConfig, LlmConfig, LlmProviderConfig, LlmStatsResponse, LlmStatsSummary, LlmStatsByAddon, LlmStatsByCategory, LlmStatsByModel, LlmStatsTimeBucket } from './llm';
export type {
  VoiceConfig,
  AsrServerConfig,
  AsrModelStatus,
  LocalAsrModelAvailability,
  LocalAsrModelScan,
  LocalAsrModelSource,
} from './voice';
export type { McpConfig, McpEndpoint, McpTemplate, McpTemplateUpdate } from './mcp';
export type { RemoteDesktopConfig, RemoteDesktopPermissionResult } from './remote_desktop';
export type { StaticHttpService } from './static-http';
export type {
  ScreenPathOrigin,
  ScreenPathInfo,
  ScreenConfiguredPaths,
  ScreenPathsResponse,
  ScreenEncoderEntry,
  ScreenServiceResponse,
  ScreenInputStatusResponse,
  ScreenConsentResetResponse,
  ScreenConfiguredStream,
  ScreenEncoderResponse,
} from './screen';

export type UartParity = 'none' | 'odd' | 'even';
export type UartStopBits = 'one' | 'two';
export type UartFlowControl = 'none' | 'software' | 'hardware';

export interface UartPortConfig {
  id: string;
  name: string;
  port: string;
  baudRate: number;
  dataBits: number;
  stopBits: UartStopBits;
  parity: UartParity;
  flowControl: UartFlowControl;
  encoding?: string;
}

export interface UartConfig {
  ports: UartPortConfig[];
}

export type ConfigSection =
  | 'server'
  | 'certificates'
  | 'remote-desktop'
  | 'terminal'
  | 'port-forwarding'
  | 'addons'
  | 'llm'
  | 'llm-stats'
  | 'voice'
  | 'ddns'
  | 'uart'
  | 'environment'
  | 'mcp'
  | 'help'
  | 'static-http';
