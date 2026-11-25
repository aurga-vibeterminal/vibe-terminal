Saat Anda menjalankan agen kode (Claude Code, Codex, Kimi, Pi, OpenCode…) di terminal, percakapannya terjebak di jendela terminal itu — tergulir di antara keluaran perintah, hilang begitu layar bergulir. `Riwayat Percakapan Agen Kode` mengangkat seluruh percakapan ke panel khusus: setiap tukar pesan tampil terstruktur, percakapan lampau dapat diganti, pesan dapat diterjemahkan — dan Anda dapat mengirim perintah ke terminal langsung dari panel.

## Membuka panel

Klik tombol obrolan (`AI`) di sisi kanan bilah judul untuk menggugurkan panel Copilot. Dropdown di atasnya mendaftar **semua percakapan lampau** agen yang berjalan di sesi terminal aktif, terbaru lebih dulu; memilih satu memuatnya.

![Panel Copilot — dropdown percakapan dan alur pesan](images/terminal-agent-history.jpg)

## Cara kerjanya

- Server tahu aplikasi agen mana yang berjalan di terminal dan mengurai **berkas sesi natifnya** (format JSONL/riwayat tiap agen di bawah direktori proyeknya); perubahan berkas mengalir langsung, menjaga tampilan percakapan mutakhir;
- Agen yang didukung: `claude` / `codex` / `kimi` / `pi` / `opencode`; aplikasi lain belum punya panel percakapan;
- Pesan membawa peran (Anda / agen) dan tipe (teks / penalaran **Thinking**), berikut nama model dan waktu, dirender sebagai Markdown.

## Fitur panel

- **Ganti percakapan**: dropdown `Conversations` di atas berpindah antar sesi lampau, tiap entri menampilkan ringkasan dan waktunya;
- **Terjemahkan**: pesan agen mana pun dapat diterjemahkan atas permintaan; terjemahan menggantikan teks di tempatnya (dan dapat dialihkan kembali);
- **Kirim perintah langsung**: ketik di kotak bawah (Shift+Enter untuk baris baru) dan tekan Enter — perintah masuk langsung ke terminal saat ini, seperti mengetik di kotak masukan agen sendiri tanpa menempati tampilan terminal.

## Keserasian dengan fitur lain

- Panel dan terminal adalah dua tampilan satu sesi: pertanyaan agen di terminal tetap dijawab di sana (atau lewat Konfirmasi Otomatis); panel untuk meninjau riwayat, bukan mencegat interaksi;
- Digabung dengan `Terminal MCP` (halaman sebelumnya), agen eksternal juga dapat membaca layar terminal dan menjalankan perintah — percakapan yang ditampilkan di sini plus aksi MCP membentuk catatan lengkap kerja agen.
