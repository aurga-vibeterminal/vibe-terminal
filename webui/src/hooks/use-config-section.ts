import { useState, useEffect, useCallback, useMemo, useRef } from 'react';

export interface UseConfigSectionOptions {
  /** When false, changes are only persisted on explicit save() calls. Defaults to true. */
  autoSave?: boolean;
  /** When false, fetching and auto-saving are paused. Defaults to true. */
  enabled?: boolean;
}

interface UseConfigSection<T> {
  data: T | null;
  savedData: T | null;
  loading: boolean;
  saving: boolean;
  loadError: string | null;
  saveError: string | null;
  isDirty: boolean;
  updateField: <K extends keyof T>(key: K, value: T[K]) => void;
  updateData: (partial: Partial<T>) => void;
  /** Persists `override` when given, otherwise the latest edited data. */
  save: (override?: T) => Promise<boolean>;
  reset: () => void;
}

const AUTO_SAVE_DELAY_MS = 500;

export function useConfigSection<T>(
  fetchFn: () => Promise<T>,
  saveFn: (data: T) => Promise<T>,
  options?: UseConfigSectionOptions,
): UseConfigSection<T> {
  const [data, setData] = useState<T | null>(null);
  const [savedData, setSavedData] = useState<T | null>(null);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [loadError, setLoadError] = useState<string | null>(null);
  const [saveError, setSaveError] = useState<string | null>(null);
  const dataRef = useRef<T | null>(null);
  const lastFailedSnapshotRef = useRef<string | null>(null);

  useEffect(() => {
    dataRef.current = data;
  }, [data]);

  const dataSnapshot = useMemo(() => (data === null ? null : JSON.stringify(data)), [data]);
  const savedSnapshot = useMemo(() => (savedData === null ? null : JSON.stringify(savedData)), [savedData]);
  const isDirty = dataSnapshot !== null && savedSnapshot !== null && dataSnapshot !== savedSnapshot;
  const enabled = options?.enabled ?? true;

  useEffect(() => {
    let cancelled = false;
    if (!enabled) {
      setLoading(false);
      setLoadError(null);
      setSaveError(null);
      return () => {
        cancelled = true;
      };
    }
    setLoading(true);
    setLoadError(null);
    setSaveError(null);
    fetchFn()
      .then((result) => {
        if (!cancelled) {
          setData(result);
          setSavedData(result);
        }
      })
      .catch((err) => {
        if (!cancelled) setLoadError(err instanceof Error ? err.message : 'Failed to load');
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, [enabled, fetchFn]);

  const updateField = useCallback(<K extends keyof T>(key: K, value: T[K]) => {
    setSaveError(null);
    setData((prev) => (prev ? { ...prev, [key]: value } : prev));
  }, []);

  const updateData = useCallback((partial: Partial<T>) => {
    setSaveError(null);
    setData((prev) => (prev ? { ...prev, ...partial } : prev));
  }, []);

  const save = useCallback(async (override?: T) => {
    const snapshot = override ?? dataRef.current;
    if (!snapshot) return true;

    const snapshotValue = JSON.stringify(snapshot);
    if (snapshotValue === savedSnapshot) {
      return true;
    }

    setSaving(true);
    setSaveError(null);
    try {
      const saved = await saveFn(snapshot);
      const currentValue = dataRef.current === null ? null : JSON.stringify(dataRef.current);
      setSavedData(saved);
      if (currentValue === snapshotValue) {
        setData(saved);
      }
      lastFailedSnapshotRef.current = null;
      return true;
    } catch (err) {
      lastFailedSnapshotRef.current = snapshotValue;
      setSaveError(err instanceof Error ? err.message : 'Failed to save');
      return false;
    } finally {
      setSaving(false);
    }
  }, [saveFn, savedSnapshot]);

  const autoSave = options?.autoSave ?? true;

  useEffect(() => {
    if (!enabled || !autoSave || loading || saving || !isDirty || dataSnapshot === null) {
      return;
    }

    if (lastFailedSnapshotRef.current === dataSnapshot) {
      return;
    }

    const timer = window.setTimeout(() => {
      void save();
    }, AUTO_SAVE_DELAY_MS);

    return () => {
      window.clearTimeout(timer);
    };
  }, [autoSave, dataSnapshot, enabled, isDirty, loading, save, saving]);

  const reset = useCallback(() => {
    setSaveError(null);
    if (savedData) setData({ ...savedData });
  }, [savedData]);

  return { data, savedData, loading, saving, loadError, saveError, isDirty, updateField, updateData, save, reset };
}
