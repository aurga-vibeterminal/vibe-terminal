export type IpFamily = 'ipv4' | 'ipv6';

export interface ServerNetworkInterface {
  id: string;
  name: string;
  ipv4Addresses: string[];
  ipv6Addresses: string[];
}

export interface ServerConfig {
  port: number;
  configPort: number;
  staticHttpPortStart: number;
  staticHttpPortEnd: number;
  staticHttpBindAddress: 'local' | 'any';
  name: string;
  secretKey: string;
  relayAddress: string;
  relaySecretKey: string;
  relayStrictTls: boolean;
  relayCaPem: string;
  certPath: string;
  keyPath: string;
  logToFile: boolean;
  autostart: boolean;
  autostartSupported?: boolean;
  /** Output-only: host OS (`std::env::consts::OS`), e.g. "linux" — drives
      platform-specific hints in the UI. */
  platform?: string;
  ddnsDomain: string;
  networkInterfaces: ServerNetworkInterface[];
}
