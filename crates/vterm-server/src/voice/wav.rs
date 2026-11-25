//! Minimal WAV + multipart helpers used to forward audio to the
//! vterm-asr-server. We avoid pulling in extra crates for this — the formats
//! are simple and we control both ends.
//!
//! Used by [`super::supervisor::AsrSupervisor::transcribe_samples`] and by the
//! voice API proxy helpers when they need to construct or forward multipart
//! payloads.

use std::io::Write;

/// Encode f32 mono samples (any sample rate the caller has; ASR pipelines
/// resample) as a 16-bit PCM 16kHz-mono WAV. The asr-server's parser only
/// reads WAV, so we always wrap into WAV. Samples are clamped to [-1.0, 1.0]
/// and scaled to signed 16-bit.
pub fn encode_pcm_f32_16k_mono(samples: &[f32]) -> Vec<u8> {
    let num_samples = samples.len();
    let data_size = (num_samples * 2) as u32;
    let mut buf: Vec<u8> = Vec::with_capacity(44 + num_samples * 2);

    // RIFF header
    buf.extend_from_slice(b"RIFF");
    buf.extend_from_slice(&(36 + data_size).to_le_bytes()); // chunk size
    buf.extend_from_slice(b"WAVE");

    // fmt subchunk (PCM, mono, 16kHz, 16-bit)
    buf.extend_from_slice(b"fmt ");
    buf.extend_from_slice(&16u32.to_le_bytes());
    buf.extend_from_slice(&1u16.to_le_bytes()); // PCM
    buf.extend_from_slice(&1u16.to_le_bytes()); // mono
    buf.extend_from_slice(&16000u32.to_le_bytes()); // sample rate
    buf.extend_from_slice(&(16000u32 * 2).to_le_bytes()); // byte rate
    buf.extend_from_slice(&2u16.to_le_bytes()); // block align
    buf.extend_from_slice(&16u16.to_le_bytes()); // bits per sample

    // data subchunk
    buf.extend_from_slice(b"data");
    buf.extend_from_slice(&data_size.to_le_bytes());
    for s in samples {
        let clamped = s.clamp(-1.0, 1.0);
        let v = (clamped * 32767.0).round() as i16;
        buf.extend_from_slice(&v.to_le_bytes());
    }
    buf
}

/// Build a random 16-byte hex multipart boundary. We don't need cryptographic
/// randomness; a wall-clock-seeded counter suffices and avoids pulling in a
/// random crate.
pub fn random_multipart_boundary() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u128)
        .unwrap_or(0);
    // Mix in the address of a local to make collisions across concurrent
    // calls vanishingly unlikely.
    let local_addr = &nanos as *const _ as u128;
    let mixed = nanos ^ (local_addr << 64) ^ (std::process::id() as u128);
    format!("----vibesupervisor{mixed:032x}")
}

/// Construct a single-file `multipart/form-data` body containing one part
/// named `field_name` with the given filename, content-type, and bytes.
pub fn build_multipart_file(
    boundary: &str,
    field_name: &str,
    filename: &str,
    content_type: &str,
    bytes: &[u8],
) -> Vec<u8> {
    let mut body = Vec::with_capacity(
        boundary.len() + field_name.len() + filename.len() + content_type.len() + bytes.len() + 128,
    );
    let _ = write!(
        body,
        "--{boundary}\r\nContent-Disposition: form-data; name=\"{field_name}\"; filename=\"{filename}\"\r\nContent-Type: {content_type}\r\n\r\n"
    );
    body.extend_from_slice(bytes);
    let _ = write!(body, "\r\n--{boundary}--\r\n");
    body
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wav_header_is_well_formed() {
        let samples = vec![0.0, 0.5, -0.5, 1.0, -1.0];
        let wav = encode_pcm_f32_16k_mono(&samples);
        assert!(wav.starts_with(b"RIFF"));
        assert_eq!(&wav[8..12], b"WAVE");
        // 5 samples * 2 bytes + 44-byte header
        assert_eq!(wav.len(), 44 + 10);
        // fmt chunk: PCM/mono/16k/16-bit
        assert_eq!(&wav[12..16], b"fmt ");
        assert_eq!(u16::from_le_bytes([wav[20], wav[21]]), 1); // PCM
        assert_eq!(u16::from_le_bytes([wav[22], wav[23]]), 1); // mono
        assert_eq!(
            u32::from_le_bytes([wav[24], wav[25], wav[26], wav[27]]),
            16000
        );
    }

    #[test]
    fn wav_clamps_overshoot() {
        let wav = encode_pcm_f32_16k_mono(&[10.0, -10.0]);
        // First sample clamped to +32767, second to -32767
        let s0 = i16::from_le_bytes([wav[44], wav[45]]);
        let s1 = i16::from_le_bytes([wav[46], wav[47]]);
        assert_eq!(s0, 32767);
        assert_eq!(s1, -32767);
    }

    #[test]
    fn multipart_body_round_trips_one_file() {
        let boundary = "----testboundary";
        let body = build_multipart_file(boundary, "file", "a.wav", "audio/wav", b"\x00\x01\x02");
        let s = std::str::from_utf8(&body).unwrap();
        assert!(s.contains("------testboundary\r\n"));
        assert!(s.contains("name=\"file\""));
        assert!(s.contains("filename=\"a.wav\""));
        assert!(s.contains("Content-Type: audio/wav"));
        assert!(s.ends_with(&format!("\r\n--{}--\r\n", boundary)));
    }

    #[test]
    fn random_boundary_is_unique_is_hex() {
        let a = random_multipart_boundary();
        let b = random_multipart_boundary();
        assert!(a.starts_with("----vibesupervisor"));
        // Uniqueness across consecutive calls (very high probability).
        assert_ne!(a, b);
        let tail = &a["----vibesupervisor".len()..];
        assert!(tail.chars().all(|c| c.is_ascii_hexdigit()));
        assert_eq!(tail.len(), 32);
    }
}
