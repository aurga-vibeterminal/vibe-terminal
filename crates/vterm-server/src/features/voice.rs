//! Voice features: audio transcription (ASR addon) and voice-input text
//! polishing (LLM addon).

use tracing::warn;
use vterm_core::protocol::Message;

use crate::connection::Connection;

impl Connection {
    pub(crate) async fn handle_voice_message(&mut self, message: Message) -> anyhow::Result<()> {
        match message {
            Message::TranscribeAudio {
                data,
                compressed,
                window_id,
                event_id,
                context: _,
            } => {
                let voice = self.voice_supervisor.clone();
                let addon_host = self.addon_host.clone();
                let config = self.config_manager.snapshot().await;
                if !config.asr.enabled {
                    self.send_message(Message::TranscribeAudioResult {
                        window_id,
                        event_id,
                        text: String::new(),
                        error: Some(crate::voice::VOICE_DISABLED_ERROR.to_string()),
                        decompress_ms: None,
                        transcribe_ms: None,
                    })
                    .await?;
                    return Ok(());
                }
                let tx = self.tx.clone();
                tokio::spawn(async move {
                    let decompress_ms;
                    let raw_data: Vec<u8> = if compressed {
                        let decomp_start = std::time::Instant::now();
                        let decompressed = match decompress_gzip(&data) {
                            Ok(d) => d,
                            Err(e) => {
                                let elapsed = decomp_start.elapsed().as_secs_f64() * 1000.0;
                                let _ = tx.send(Message::TranscribeAudioResult {
                                    window_id,
                                    event_id: event_id.clone(),
                                    text: String::new(),
                                    error: Some(format!("Decompression failed: {}", e)),
                                    decompress_ms: Some(elapsed),
                                    transcribe_ms: None,
                                });
                                return;
                            }
                        };
                        decompress_ms = Some(decomp_start.elapsed().as_secs_f64() * 1000.0);
                        decompressed
                    } else {
                        decompress_ms = None;
                        data.into()
                    };
                    let samples = decode_f32_samples(&raw_data);
                    let transcribe_start = std::time::Instant::now();

                    // Transcription always goes through the selected ASR addon
                    // (the supervised vterm-voice is reached via the vibe-voice
                    // builtin addon, whose base_url defaults to the supervised
                    // endpoint). There's no separate "local" path anymore.
                    let provider = config.asr.provider.clone();

                    let result: anyhow::Result<String> = if provider.is_empty() {
                        Err(anyhow::anyhow!(
                            "no ASR provider configured; select one in Voice settings"
                        ))
                    } else {
                        // run_transcription is sync (rquickjs + blocking HTTP);
                        // run it on spawn_blocking so we don't stall the async
                        // runtime. Language is no longer a top-level param —
                        // it lives in the addon's config (the vibe-voice addon
                        // declares a `language` field) and the JS descriptor
                        // folds it into the request.
                        let addon_host = addon_host.clone();
                        let config = config.clone();
                        let samples = samples.clone();
                        tokio::task::spawn_blocking(move || {
                            let res = crate::addon::run_transcription(
                                &addon_host,
                                &config,
                                &provider,
                                &samples,
                                Some(&config.asr.provider_config),
                                None,
                            );
                            match (res.success, res.text, res.error) {
                                (true, Some(text), _) => Ok(text),
                                (_, _, Some(e)) => Err(anyhow::anyhow!(e)),
                                _ => Err(anyhow::anyhow!(
                                    "voice addon returned no text and no error"
                                )),
                            }
                        })
                        .await
                        .map_err(|e| anyhow::anyhow!("voice addon task panicked: {e}"))
                        .and_then(|r| r)
                    };

                    let transcribe_ms = Some(transcribe_start.elapsed().as_secs_f64() * 1000.0);
                    let msg = match result {
                        Ok(text) => Message::TranscribeAudioResult {
                            window_id,
                            event_id,
                            text,
                            error: None,
                            decompress_ms,
                            transcribe_ms,
                        },
                        Err(e) => Message::TranscribeAudioResult {
                            window_id,
                            event_id,
                            text: String::new(),
                            error: Some(e.to_string()),
                            decompress_ms,
                            transcribe_ms,
                        },
                    };
                    let _ = tx.send(msg);
                });
            }
            Message::PolishVoiceInputText {
                window_id,
                component_id,
                request_id,
                context,
                text,
            } => {
                let session_id = context.session_id();
                let config = self.config_manager.snapshot().await;
                let addon_host = self.addon_host.clone();
                let context_text = self
                    .session_manager
                    .voice_input_polish_context(&context)
                    .await;
                let tx = self.tx.clone();
                tokio::spawn(async move {
                    let result = tokio::task::spawn_blocking(move || {
                        crate::voice::polish_voice_input_text(
                            &addon_host,
                            &config,
                            &text,
                            &context,
                            &context_text,
                        )
                    })
                    .await;

                    let (text, error) = match result {
                        Ok(Ok(text)) => (Some(text), None),
                        Ok(Err(error)) => (None, Some(error)),
                        Err(error) => (
                            None,
                            Some(format!("Voice input polish task failed: {error}")),
                        ),
                    };

                    let _ = tx.send(Message::PolishVoiceInputTextResult {
                        window_id,
                        component_id,
                        session_id,
                        request_id,
                        text,
                        error,
                    });
                });
            }
            other => {
                warn!("Unhandled voice message: {}", other.msg_name());
            }
        }

        Ok(())
    }
}

fn decode_f32_samples(data: &[u8]) -> Vec<f32> {
    data.chunks_exact(4)
        .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect()
}

fn decompress_gzip(data: &[u8]) -> anyhow::Result<Vec<u8>> {
    use flate2::read::GzDecoder;
    use std::io::Read;
    let mut decoder = GzDecoder::new(data);
    let mut decompressed = Vec::with_capacity(data.len() * 2);
    decoder.read_to_end(&mut decompressed)?;
    Ok(decompressed)
}
