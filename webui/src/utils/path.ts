/**
 * Normalize a filesystem path for display.
 *
 * Strips the Win32 extended-length prefix:
 *   \\?\C:\foo\bar        -> C:\foo\bar
 *   \\?\UNC\server\share -> \\server\share
 * Non-prefixed paths (POSIX or plain Windows) are returned unchanged.
 */
export function formatPath(path: string): string {
  if (path.startsWith('\\\\?\\UNC\\')) {
    return `\\\\${path.slice(8)}`;
  }
  if (path.startsWith('\\\\?\\')) {
    return path.slice(4);
  }
  return path;
}
