/** A live, server-side port forward created on demand (e.g. by the HTTP share flow). */
export interface PortForward {
  targetPort: number;
  proxyPort: number;
  activeConnections: number;
}

export interface CreatePortForwardRequest {
  targetPort: number;
}

export interface PortForwardShare {
  shareToken?: string;
  ticket?: string;
}

export interface PortForwardingConfig {
  enabled: boolean;
  idleTimeoutMinutes: number;
  portRangeStart: number;
  portRangeEnd: number;
  forwards: PortForward[];
}
