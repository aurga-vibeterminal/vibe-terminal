// Vibe Voice (Local ASR) Addon
//
// Implements transcribe_descriptor(audio, options), returning a standard
// envelope:
//
//   { "code": 0, "descriptor": { ... } }   — success; Rust executes the
//                                            descriptor to do the actual
//                                            multipart POST + response parse
//   { "code": <nonzero>, "error": "..." }  — failure; Rust surfaces the error
//
// Why a descriptor and not the full HTTP call here: rquickjs strings are
// UTF-8 and have no ergonomic Uint8Array interop, so binary audio stays on
// the Rust side. The descriptor is just a description of how to call the
// provider — URL, headers, query params, and where in the JSON response the
// transcript lives. This keeps the JS layer thin and lets new providers be
// added as pure-JS configuration.
//
//   audio    = { filename, mime, sample_count, duration_s }
//   options  = {} (reserved; currently unused)
//
// Descriptor shape (inside the success envelope):
//   {
//     "url": "<base_url>/v1/audio/transcriptions",
//     "headers": { "Authorization": "Bearer <token>" },  // omitted when no token
//     "response_text_path": "text",                      // top-level JSON key
//     "query": { "language": "..." }                     // omitted when "auto"/empty
//   }

function transcribe_descriptor(audio, options) {
  // Normalize the configured base_url so we always dial the loopback IPv4
  // literal, never `localhost`. On Windows, `localhost` frequently resolves
  // to ::1 first; since vterm-server's supervisor always binds 127.0.0.1
  // (IPv4 only), an IPv6 dial gets connection-refused. We rewrite any
  // `localhost` spelling to `127.0.0.1`. Trailing slashes are also stripped.
  var baseUrl = (addon.get_config("base_url") || "https://127.0.0.1:8398")
    .replace(/\/+$/, "")
    .replace(/localhost/g, "127.0.0.1");
  // access_token is mirrored from asr.access_token by update_voice_config
  // (not an editable addon field) so the addon always sees the same token
  // the supervisor passes to the asr-server via --access-token.
  var accessToken = addon.get_config("access_token") || "";
  // Language is per-provider config (not a per-utterance caller argument).
  // "auto" / empty → let the asr-server auto-detect; otherwise forward as a
  // ?language= query param on the transcription request.
  var language = (addon.get_config("language") || "auto").trim();

  var descriptor = {
    url: baseUrl + "/v1/audio/transcriptions",
    response_text_path: "text"
  };

  var headers = {};
  if (accessToken) {
    headers["Authorization"] = "Bearer " + accessToken;
  }
  if (Object.keys(headers).length > 0) {
    descriptor.headers = headers;
  }

  if (language && language.toLowerCase() !== "auto") {
    descriptor.query = { language: language };
  }

  // Success envelope — Rust extracts .descriptor.
  return JSON.stringify({ code: 0, descriptor: descriptor });
}
