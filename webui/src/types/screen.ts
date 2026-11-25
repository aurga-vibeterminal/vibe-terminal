export type ScreenPathOrigin = 'auto' | 'env' | 'override';

export interface ScreenPathInfo {
  value: string | null;
  origin: ScreenPathOrigin;
  valid: boolean;
  detail: string;
}

export interface ScreenConfiguredPaths {
  androidSdkPath: string | null;
  androidStudioPath: string | null;
}

export interface ScreenPathsResponse {
  sdk: ScreenPathInfo;
  agentDir: ScreenPathInfo;
  configured: ScreenConfiguredPaths;
  /** Set when the screen-server was unreachable (GET). */
  error: string | null;
  /** Set when the config was saved but pushing it to the screen-server failed (PUT). */
  pushError: string | null;
}

export interface ScreenEncoderEntry {
  id: string;
  label: string;
  available: boolean;
  reason: string;
}

export interface ScreenConfiguredStream {
  /** Encoder backend id ("auto" | "vendor" | "d3d12" | "mf"); null = auto. */
  encoder: string | null;
  /** Frame rate (15 | 30 | 60); null = 60. */
  fps: number | null;
  /** Capture-resolution ceiling ("4k" | "2.5k" | "2k"); null = "4k". */
  maxResolution: string | null;
  /** Fixed average bitrate in Mbps (50 | 45 | … | 3); null = auto (area-scaled). */
  bitrateMbps: number | null;
  /** macOS headless hosts: capture the main display through a CGVirtualDisplay
   *  (applies to streams started after the push; ignored elsewhere). */
  virtualDisplay: boolean;
}

export interface ScreenEncoderResponse {
  entries: ScreenEncoderEntry[];
  configured: ScreenConfiguredStream;
  /** Backend id currently in effect on the screen-server. */
  current: string | null;
  /** Frame rate currently in effect on the screen-server. */
  fps: number | null;
  /** Max-resolution override in effect on the screen-server (null = "4k"). */
  maxResolution: string | null;
  /** Fixed-bitrate override in effect, kbit/s (null = auto area-scaled). */
  bitrateKbps: number | null;
  /** Set when the screen-server was unreachable (GET). */
  error: string | null;
  /** Set when the config was saved but pushing it to the screen-server failed (PUT). */
  pushError: string | null;
}

/** GET/POST /api/screen/service — SCM state of the Windows service. */
export interface ScreenServiceResponse {
  installed: boolean;
  running: boolean;
  /** User-presentable result/error text (empty on plain status). */
  message: string;
  /** Whether a Virtual Display Driver device is present on the host
   *  (Windows only; null on other platforms). */
  vddInstalled: boolean | null;
}

/** GET /api/screen/input — the Linux `/dev/uinput` probe. */
export interface ScreenInputStatusResponse {
  /** The kernel module is loaded (`/dev/uinput` exists). */
  uinputExists: boolean;
  /** This process can open `/dev/uinput` read-write (the udev/group grant). */
  uinputAccessible: boolean;
  /** Raw probe error text (empty when accessible). */
  detail: string;
  /** Set when the screen-server was unreachable. */
  error: string | null;
}

/** POST /api/screen/consent/reset — forget the portal ScreenCast consent. */
export interface ScreenConsentResetResponse {
  /** How many persisted restore tokens were removed. */
  cleared: number;
  /** Set when the reset failed (screen-server unreachable / non-Linux). */
  error: string | null;
}
