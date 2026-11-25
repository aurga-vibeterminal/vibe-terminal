//! Parsing of encrypted server share strings for `vtermc -a`.
//!
//! The webui ServerSection encrypts the server info as XTEA-CBC and formats
//! it as `vtshare1:<iv hex>:<ciphertext hex>`; clicking the pairing QR code
//! copies this string. This module mirrors the desktop client's
//! implementation (`vterm/src/ui/server_manager/share_payload.rs`)
//! so the CLI can decrypt a pasted share string with the same rules — keep
//! the copies in sync.

use serde::Deserialize;

const SHARE_PREFIX: &str = "vtshare1:";
/// Four big-endian words of the shared ASCII key. The webui derives them
/// from `VibeTermQrKeyV1` and reads the missing 16th byte as 0, hence the
/// trailing NUL here.
const XTEA_KEY: [u8; 16] = *b"VibeTermQrKeyV1\0";
const XTEA_DELTA: u32 = 0x9e37_79b9;
const XTEA_ROUNDS: u32 = 32;
const BLOCK_SIZE: usize = 8;

#[derive(Deserialize)]
struct SharePayload {
    name: String,
    host: String,
    port: u16,
    secret_key: String,
}

/// Server info decoded from a share string, ready to save in the client
/// config.
#[derive(Clone)]
pub struct SharedServer {
    pub name: String,
    /// `wss://host:port` with IPv6 literals bracketed — the address format
    /// the server manager stores.
    pub host: String,
    pub secret_key: String,
}

/// Parse and decrypt a share string. Returns `None` for any other input.
pub fn parse_share_string(text: &str) -> Option<SharedServer> {
    let rest = text.trim().strip_prefix(SHARE_PREFIX)?;
    let (iv_hex, ciphertext_hex) = rest.split_once(':')?;
    let iv = unhex(iv_hex)?;
    let ciphertext = unhex(ciphertext_hex)?;
    if ciphertext.is_empty() || ciphertext.len() % BLOCK_SIZE != 0 || iv.len() != BLOCK_SIZE {
        return None;
    }

    let key = xtea_key_words();
    let mut plaintext = Vec::with_capacity(ciphertext.len());
    let mut previous = iv;
    for block in ciphertext.chunks(BLOCK_SIZE) {
        let mut left = be_word(block, 0);
        let mut right = be_word(block, 4);
        decrypt_xtea_block(&mut left, &mut right, &key);
        let mut plain_block = [0u8; BLOCK_SIZE];
        write_be_word(&mut plain_block, 0, left);
        write_be_word(&mut plain_block, 4, right);
        for (byte, prev) in plain_block.iter_mut().zip(&previous) {
            *byte ^= prev;
        }
        previous.copy_from_slice(block);
        plaintext.extend_from_slice(&plain_block);
    }

    // The webui pads with 1..=8 bytes of the pad length (8 when aligned).
    let padding = *plaintext.last()? as usize;
    if padding == 0 || padding > BLOCK_SIZE || padding > plaintext.len() {
        return None;
    }
    let plain = &plaintext[..plaintext.len() - padding];
    let payload: SharePayload = serde_json::from_slice(plain).ok()?;

    Some(SharedServer {
        name: payload.name,
        host: format_host(&payload.host, payload.port),
        secret_key: payload.secret_key,
    })
}

/// `wss://host:port`, bracketing IPv6 literals.
fn format_host(host: &str, port: u16) -> String {
    if host.contains(':') {
        format!("wss://[{host}]:{port}")
    } else {
        format!("wss://{host}:{port}")
    }
}

/// XTEA block decryption — the inverse of the webui's `encryptXteaBlock`.
fn decrypt_xtea_block(left: &mut u32, right: &mut u32, key: &[u32; 4]) {
    let mut sum = XTEA_DELTA.wrapping_mul(XTEA_ROUNDS);
    for _ in 0..XTEA_ROUNDS {
        *right = right.wrapping_sub(
            (((*left << 4) ^ (*left >> 5)).wrapping_add(*left))
                ^ sum.wrapping_add(key[((sum >> 11) & 3) as usize]),
        );
        sum = sum.wrapping_sub(XTEA_DELTA);
        *left = left.wrapping_sub(
            (((*right << 4) ^ (*right >> 5)).wrapping_add(*right))
                ^ sum.wrapping_add(key[(sum & 3) as usize]),
        );
    }
}

fn xtea_key_words() -> [u32; 4] {
    [
        be_word(&XTEA_KEY, 0),
        be_word(&XTEA_KEY, 4),
        be_word(&XTEA_KEY, 8),
        be_word(&XTEA_KEY, 12),
    ]
}

fn be_word(bytes: &[u8], offset: usize) -> u32 {
    u32::from_be_bytes([
        bytes.get(offset).copied().unwrap_or(0),
        bytes.get(offset + 1).copied().unwrap_or(0),
        bytes.get(offset + 2).copied().unwrap_or(0),
        bytes.get(offset + 3).copied().unwrap_or(0),
    ])
}

fn write_be_word(out: &mut [u8], offset: usize, word: u32) {
    out[offset..offset + 4].copy_from_slice(&word.to_be_bytes());
}

fn unhex(hex: &str) -> Option<Vec<u8>> {
    if hex.len() % 2 != 0 {
        return None;
    }
    (0..hex.len() / 2)
        .map(|index| u8::from_str_radix(&hex[index * 2..index * 2 + 2], 16).ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Vector produced by the webui implementation with fixed IV
    /// `0001020304050607` for the payload
    /// `{"type":"vibe-terminal-server","name":"Test Server","host":"192.168.1.10","port":8780,"secret_key":"mySecretKey123"}`.
    const VECTOR: &str = "vtshare1:0001020304050607:a85b448077b55fbe4e7237db78380b1b1cbae9d47d2457d143a5fab478d44e63c3d5acc8b0de33a72200a032ecd069d13d394f734e249842a43fbc74d81aaf458b0eeb28ee9df904b1136a88364070e81476a97270f1177f59762481406ff0ad89a3c3175f82e15c10f9f34b19ee9f7db3812c1ba14da132";

    #[test]
    fn decrypts_webui_share_string() {
        let shared = parse_share_string(VECTOR).expect("webui vector must decrypt");
        assert_eq!(shared.name, "Test Server");
        assert_eq!(shared.host, "wss://192.168.1.10:8780");
        assert_eq!(shared.secret_key, "mySecretKey123");
    }

    #[test]
    fn brackets_ipv6_hosts() {
        let shared = parse_share_string(ipv6_vector()).expect("ipv6 vector must decrypt");
        assert_eq!(shared.name, "v6");
        assert_eq!(shared.host, "wss://[fd00::1a]:8781");
        assert_eq!(shared.secret_key, "");
    }

    #[test]
    fn rejects_other_input() {
        assert!(parse_share_string("hello").is_none());
        assert!(parse_share_string("").is_none());
        assert!(parse_share_string("vtshare1:zz:badd").is_none());
        assert!(parse_share_string("vtshare1:00:aa").is_none());
        assert!(parse_share_string("vtqr1:00:aa").is_none());
    }

    /// Vector for `{"type":"vibe-terminal-server","name":"v6","host":"fd00::1a","port":8781,"secret_key":""}`
    /// with the same fixed IV.
    fn ipv6_vector() -> &'static str {
        "vtshare1:0001020304050607:a85b448077b55fbe4e7237db78380b1b1cbae9d47d2457d143a5fab478d44e6322678844239dc10eaa21276030ce84e83e002de3ce6fc40782c0293e59278221677cff2794314611fc6005fe011648e6720b5b55e4844e5ddf7e12fdf83f304d"
    }
}
