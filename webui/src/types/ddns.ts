export interface DdnsSyncStatus {
  lastSync: string | null;
  lastIp: string | null;
  nextSync: string | null;
  syncing: boolean;
}

export interface DdnsConfig {
  enabled: boolean;
  provider: string;
  config: Record<string, string>;
  pollingIntervalMinutes: number;
  ipFamily: string;
  networkInterface?: string;
  syncStatus: DdnsSyncStatus;
  ddnsDomain: string;
}
