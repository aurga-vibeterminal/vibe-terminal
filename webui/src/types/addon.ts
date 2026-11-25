export type AddonFieldType = 'text' | 'password' | 'boolean' | 'number' | 'integer' | 'textarea';

export interface AddonConfigField {
  key: string;
  label: string;
  fieldType: AddonFieldType;
  required: boolean;
  defaultValue?: string;
  description?: string;
}

export interface AddonInfo {
  id: string;
  name: string;
  description: string;
  author: string;
  /** Author contact email from the manifest (may be empty). */
  email: string;
  /** Release/version date from the manifest (may be empty). */
  date: string;
  addonType: string;
  builtin: boolean;
  /**
   * True when the served source is a user-authored override (builtin) or
   * an uploaded addon the user owns. Drives the "Modified"/"Uploaded"
   * badge and the availability of "Reset".
   */
  modified: boolean;
  enabled: boolean;
  configFields: AddonConfigField[];
  userConfig: Record<string, string>;
}

export interface AddonDetail extends AddonInfo {
  source: string;
}

export interface AddonConfigUpdate {
  enabled: boolean;
  userConfig: Record<string, string>;
}
