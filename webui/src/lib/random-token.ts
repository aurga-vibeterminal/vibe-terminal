/// Generate a 32-char random token from the URL-safe alphanumeric alphabet.
/// Uses crypto.getRandomValues when available (browser) and falls back to
/// Math.random for non-secure contexts. Sufficient for loopback/service
/// bearer tokens.
export function generateRandomToken(): string {
  const alphabet = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';
  const out: string[] = [];
  if (typeof crypto !== 'undefined' && crypto.getRandomValues) {
    const buf = new Uint32Array(32);
    crypto.getRandomValues(buf);
    for (let i = 0; i < 32; i++) {
      out.push(alphabet[buf[i] % alphabet.length]);
    }
  } else {
    for (let i = 0; i < 32; i++) {
      out.push(alphabet[Math.floor(Math.random() * alphabet.length)]);
    }
  }
  return out.join('');
}
