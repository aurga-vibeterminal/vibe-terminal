Pelayan mempunyai hos Model Context Protocol terbina dalam, dan bahagian `MCP` dalam Web UI digunakan untuk menetapkannya: Code Agent (Claude Code, Codex, Kimi Code, Pi, OpenCode, Qwen Code, DeepSeek Harness, …) boleh bersambung dari sembang masing-masing dan bekerja terus pada mesin anda — membaca dan mengawal terminal, malah mengendalikan EasyEDA Pro.

## Tetapan pelayan

Aktifkan pelayan MCP dan tetapkan:

- `Port` (lalai `6180`) dan `Bind Address` — `127.0.0.1` mengekalkan endpoint di localhost sahaja; `0.0.0.0` membukanya ke rangkaian;
- `Access Token` — diperlukan untuk permintaan bukan localhost: pemanggil jauh mesti membawanya sebagai Bearer token, manakala permintaan dari localhost sentiasa dibenarkan. Ia tidak pernah kosong — pemasangan baharu disertakan token rawak; butang dadu menjana semula, butang salin mengeluarkannya;
- `Stream Throttle` — selang minimum antara dorongan strim skrin kepada agent yang bersambung.

Perubahan berkuat kuasa selepas `Save`; pelayan memuat semula tanpa mula semula.

## Endpoint

Setiap set keupayaan dilayan pada URL tersendiri, `http://localhost:{port}/{id}/mcp`:

- **Terminal MCP** (`/terminal/mcp`) — permukaan terminal terbina dalam. Agent boleh menyenaraikan terminal, membaca skrin kelihatan dan skrol balik, menaip input mentah dan kekunci bernama, mengubah saiz serta menunggu prom. Panggilan alat menyasarkan satu terminal melalui Terminal ID-nya — klik kanan terminal → `MCP` → `Copy Terminal ID` kemudian tampal ID ke dalam konteks agent (lihat Terminal MCP di bahagian Terminals);
- **EasyEDA Pro MCP** (`/easyeda/mcp`) — addon MCP terbina dalam yang menjambatani EasyEDA Pro kepada agent: satu set alat tetap kecil beserta katalog panggilan `eda.*` yang boleh dicari, disampaikan kepada sambungan penyambung EasyEDA melalui WebSocket. Aktifkan addon di bahagian `Addons` dahulu — endpointnya muncul dalam senarai selepas dipasang, dan lencana menunjukkan sama ada daemon addon sedang bersambung. Medan `Timeout` addon mengehadkan setiap permintaan berjambatan (naikkan untuk operasi EDA yang perlahan);
- **Addon MCP lain** — sebarang addon jenis MCP yang dipasang menambah endpointnya dengan cara yang sama, masing-masing dengan lencana bersambung/terputus secara langsung.

Pilih CLI agent anda pada menu lungsur di atas senarai; butang salin di sebelah endpoint kemudian memberikan arahan persediaan sedia dijalankan untuk aplikasi itu.

## Menyambungkan agent

Token tidak pernah ditampal secara manual. Pelayan mengekalkan profil persekitaran `Vibe Terminal MCP Bearer Token` yang dijana secara automatik (lihat bahagian Environment Variables) yang mengeksport `VTERM_MCP_TOKEN`; sapukan pada terminal sebelum menyambungkan agent, kemudian gunakan arahan yang disalin:

- **Aplikasi arahan shell** (Claude Code, Codex) — jalankan arahan `mcp add` yang disalin dalam terminal biasa. Codex membaca token dari `VTERM_MCP_TOKEN` melalui `--bearer-token-env-var`, jadi tiada rahsia berakhir dalam baris arahan;
- **Aplikasi perbualan** (Kimi Code, Pi, OpenCode, Qwen Code, DeepSeek Harness) — mulakan aplikasi dengan profil disapukan, kemudian hantar prom yang disalin ke dalam perbualannya. Ejen mengemas kini konfigurasi MCP secara langsung. Kimi menggunakan `bearerTokenEnvVar`; Pi, OpenCode dan Qwen Code menggunakan token Bearer daripada `VTERM_MCP_TOKEN`, jadi token itu sendiri tidak pernah muncul dalam sembang.

Dalam Pi dan Qwen Code, `/mcp` memeriksa sambungan.

Kad `Command Templates` di bawah tetapan menyimpan arahan persediaan mengikut aplikasi: sunting entri, batalkan gantian terbina dalam, atau tambah templat untuk CLI agent baharu — pratonton menunjukkan tepat apa yang akan dihasilkan oleh butang salin.
