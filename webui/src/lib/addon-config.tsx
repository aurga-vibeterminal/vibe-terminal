/**
 * Shared helpers for rendering addon config fields across section pages
 * (Voice, DDNS, LLM, ...).
 *
 * Contract enforced by these helpers:
 * - The default value declared in an addon's manifest (`config_fields[].default`)
 *   is shown ONLY as the field's placeholder — the input stays empty until
 *   the user types a value, so what is displayed (and later saved) always
 *   reflects the user's explicit choice. An empty string in the saved config
 *   is treated as "not set".
 */
import type { AddonConfigField, AddonInfo } from '@/types';
import { SecretField } from '@/components/shared/SecretField';
import { ToggleField } from '@/components/shared/ToggleField';

const inputClassName = [
  'w-full bg-bg-inset px-2 py-1.5 text-sm font-normal',
  'rounded-standard border border-golden-sand',
  'transition-colors duration-vt-base',
  'focus:outline-none focus:border-amber-gold',
  'disabled:opacity-50 disabled:cursor-not-allowed',
  'text-warm-black',
].join(' ');

const textareaClassName = [
  'w-full bg-bg-inset px-2 py-1.5 text-sm font-normal',
  'rounded-standard border border-golden-sand',
  'transition-colors duration-vt-base',
  'focus:outline-none focus:border-amber-gold',
  'disabled:opacity-50 disabled:cursor-not-allowed',
  'text-warm-black resize-y min-h-[80px]',
].join(' ');

/**
 * Resolve the effective value for each config field of an addon from the
 * saved user config alone.
 *
 * Behavior:
 * - If the saved map has the key (even as `''`), that value wins; the user
 *   may intentionally clear a field, and the placeholder then carries the
 *   manifest default hint.
 * - If the saved map does NOT have the key (never touched), the field stays
 *   empty — the manifest default is applied server-side at runtime and is
 *   advertised here only through the placeholder.
 */
export function resolveAddonConfig(
  addon: AddonInfo,
  saved: Record<string, string>,
): Record<string, string> {
  const merged: Record<string, string> = {};
  for (const field of addon.configFields) {
    merged[field.key] = saved[field.key] ?? '';
  }
  return merged;
}

/**
 * Render a single addon config field with the right input control for its
 * `fieldType`. The placeholder always carries the manifest default so an
 * emptied field still hints at the default value.
 */
export function renderAddonConfigField(
  field: AddonConfigField,
  value: string,
  onChange: (value: string) => void,
  disabled: boolean,
) {
  const placeholder = field.defaultValue ?? '';
  switch (field.fieldType) {
    case 'boolean':
      return (
        <ToggleField
          enabled={value === 'true'}
          onChange={(v) => onChange(v ? 'true' : 'false')}
          label={field.label}
          disabled={disabled}
        />
      );

    case 'number':
      // String-based like the integer case: an emptied field stays empty so
      // the placeholder (manifest default) remains visible, instead of
      // NumberInput's Number(value) || 0 snapping '' to 0.
      return (
        <input
          type="number"
          step="any"
          value={value}
          onChange={(e) => onChange(e.target.value)}
          className={inputClassName}
          placeholder={placeholder}
          disabled={disabled}
        />
      );

    case 'integer':
      return (
        <input
          type="number"
          step={1}
          value={value}
          onChange={(e) => onChange(e.target.value)}
          className={inputClassName}
          placeholder={placeholder}
          disabled={disabled}
        />
      );

    case 'password':
      return (
        <SecretField
          value={value}
          onChange={(e) => onChange(e.target.value)}
          placeholder={placeholder}
          disabled={disabled}
        />
      );

    case 'textarea':
      return (
        <textarea
          value={value}
          onChange={(e) => onChange(e.target.value)}
          className={textareaClassName}
          placeholder={placeholder}
          disabled={disabled}
        />
      );

    case 'text':
    default:
      return (
        <input
          type="text"
          value={value}
          onChange={(e) => onChange(e.target.value)}
          className={inputClassName}
          placeholder={placeholder}
          disabled={disabled}
        />
      );
  }
}
