import { useConnection } from '@/context/connection-context';

export function useConnectionStatus() {
  return useConnection();
}
