import type {
  ServerConfig,
  DeviceConfig,
  PortForwardingConfig,
  DdnsConfig,
} from '@/types';

export const defaultServerConfig: ServerConfig = {
  port: 5153,
  configPort: 5180,
  staticHttpPortStart: 63000,
  staticHttpPortEnd: 63999,
  staticHttpBindAddress: 'local',
  name: 'Vibe Terminal',
  secretKey: '',
  relayAddress: '',
  relaySecretKey: '',
  relayStrictTls: false,
  relayCaPem: '',
  certPath: '/etc/ssl/certs/server.crt',
  keyPath: '/etc/ssl/private/server.key',
  logToFile: true,
  autostart: false,
  autostartSupported: true,
  ddnsDomain: '',
  networkInterfaces: [],
};

export const defaultDeviceConfig: DeviceConfig = {
  baudRate: 115200,
};

export const defaultPortForwardingConfig: PortForwardingConfig = {
  enabled: true,
  idleTimeoutMinutes: 0,
  portRangeStart: 62000,
  portRangeEnd: 62999,
  forwards: [],
};

export const defaultDdnsConfig: DdnsConfig = {
  enabled: false,
  provider: 'cloudflare-ddns',
  config: {},
  pollingIntervalMinutes: 5,
  ipFamily: 'ipv4',
  ddnsDomain: '',
  syncStatus: {
    lastSync: '2025-12-15T10:30:00Z',
    lastIp: '203.0.113.42',
    nextSync: '2025-12-15T10:35:00Z',
    syncing: false,
  },
};
