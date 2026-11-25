import {
  createContext,
  useContext,
  useState,
  useEffect,
  useCallback,
  type ReactNode,
} from 'react';

export type ConnectionStatus = 'connected' | 'disconnected' | 'reconnecting';

interface ConnectionContextValue {
  status: ConnectionStatus;
  lastChecked: Date | null;
}

const ConnectionContext = createContext<ConnectionContextValue | null>(null);

export function ConnectionProvider({ children }: { children: ReactNode }) {
  const [status, setStatus] = useState<ConnectionStatus>('connected');
  const [lastChecked, setLastChecked] = useState<Date | null>(null);

  const checkConnection = useCallback(() => {
    // Simulate occasional disconnects for demo
    const rand = Math.random();
    if (rand < 0.95) {
      setStatus('connected');
    } else if (rand < 0.98) {
      setStatus('reconnecting');
      setTimeout(() => setStatus('connected'), 2000);
    } else {
      setStatus('disconnected');
      setTimeout(() => setStatus('connected'), 3000);
    }
    setLastChecked(new Date());
  }, []);

  useEffect(() => {
    checkConnection();
    const interval = setInterval(checkConnection, 10000);
    return () => clearInterval(interval);
  }, [checkConnection]);

  return (
    <ConnectionContext.Provider value={{ status, lastChecked }}>
      {children}
    </ConnectionContext.Provider>
  );
}

export function useConnection() {
  const ctx = useContext(ConnectionContext);
  if (!ctx)
    throw new Error('useConnection must be used within ConnectionProvider');
  return ctx;
}
