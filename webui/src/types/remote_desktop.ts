export interface RemoteDesktopConfig {
  enabled: boolean;
  platform: string;
  /** Host CPU architecture reported by the server (e.g. "x86_64", "aarch64"). */
  arch: string;
  requiresPermission: boolean;
  permissionGranted: boolean | null;
}

export interface RemoteDesktopPermissionResult {
  granted: boolean;
  message: string;
}
