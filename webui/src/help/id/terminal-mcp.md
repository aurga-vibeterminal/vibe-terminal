Setiap terminal adalah warga kelas satu MCP (Model Context Protocol): agen eksternal mengemudikan terminal lewat alat MCP server — daftar terminal, baca layar, jalankan perintah, tunggu selesai — memungkinkan agen pengodean yang sudah Anda pakai mengoperasikan shell interaktif nyata.

## Alat MCP terminal milik server

Sekitar sesi terminal, klien MCP dapat:

- `list_terminals` — semua terminal terbuka; `get_terminal` — judul, cwd, dan status aplikasi satu terminal;
- `get_screen` — teks layar tampak saat ini; `get_scrollback` — baris riwayat terbaru;
- `send_command` — menjalankan baris perintah (Enter ditambahkan); `send_input` — teks mentah tanpa Enter (mengonfirmasi `y`, mengisi formulir); `send_key` — tombol bernama (Enter, Ctrl+C, panah);
- `clear_terminal` — mengosongkan buffer; `resize_terminal` — mengubah ukuran PTY; `wait_terminal` — memblokir sampai terminal tidak lagi sibuk.

## Menunjuk terminal: Terminal ID

Panggilan alat menargetkan terminal lewat Terminal ID-nya. Klik kanan di dalam terminal → `MCP` → `Copy Terminal ID`, lalu tempel ID ke agen Anda agar ia mengemudikan tepat terminal itu.

![Klik kanan terminal — submenu MCP](images/terminal-mcp-menu.jpg)

## Pendaftaran aplikasi agen sekali klik

Submenu `MCP` juga mendaftar aplikasi agen yang terdeteksi di server (CLI editor dan sejenisnya). Mengklik aplikasi mengetik perintah `mcp add` siap pakai ke terminal — **tanpa baris baru di akhir**; tinjau dan tekan Enter sendiri untuk mendaftarkan layanan MCP server ini ke agen itu. Entri bersembunyi selama aplikasi sudah berjalan di sesi, agar teks tidak mendarat di UI-nya.

> Alur tipikal: salin Terminal ID ke agen Anda → `send_command` mulai build → `wait_terminal` sampai selesai → `get_screen` membaca hasilnya. Bersama Konfirmasi Otomatis (lihat bab terminal), tugas interaktif panjang berjalan nyaris tanpa pengawasan.
