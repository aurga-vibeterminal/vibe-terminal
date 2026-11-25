export type TerminalPatternKind = 'confirmation' | 'selection' | 'completion' | 'ignore';

export interface TerminalPatternConfig {
  id: string;
  kind: TerminalPatternKind;
  substrings: string[];
  enabled: boolean;
}

export interface TerminalEnvVar {
  name: string;
  value: string;
  enabled: boolean;
}

export interface TerminalConfig {
  enabled: boolean;
  autoConfirm: boolean;
  analysisDelaySeconds: number;
  confirmationGraceSeconds: number;
  notifyOnCompletion: boolean;
  patterns: TerminalPatternConfig[];
  envVars: TerminalEnvVar[];
}
