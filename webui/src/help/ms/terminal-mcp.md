Setiap terminal ialah warganegara kelas pertama MCP (Model Context Protocol): ejen luaran memandu terminal melalui alat MCP pelayan — senaraikan terminal, baca skrin, jalankan perintah, tunggu penyiapan — membolehkan ejen pengekodan yang sudah anda guna mengendalikan shell interaktif sebenar.

## Alat MCP terminal pelayan

Sekitar sesi terminal, klien MCP boleh:

- `list_terminals` — semua terminal terbuka; `get_terminal` — tajuk, cwd dan status aplikasi satu terminal;
- `get_screen` — teks skrin kelihatan semasa; `get_scrollback` — baris sejarah terkini;
- `send_command` — melaksanakan baris perintah (Enter dilampirkan); `send_input` — teks mentah tanpa Enter (mengesahkan `y`, mengisi borang); `send_key` — kekunci bernama (Enter, Ctrl+C, anak panah);
- `clear_terminal` — kosongkan penimbal; `resize_terminal` — saiz semula PTY; `wait_terminal` — menyekat sehingga terminal tidak lagi sibuk.

## Mengalamatkan terminal: Terminal ID

Panggilan alat menyasarkan terminal melalui Terminal ID-nya. Klik kanan dalam terminal → `MCP` → `Copy Terminal ID`, kemudian tampal ID kepada ejen anda supaya ia memandu tepat terminal itu.

![Klik kanan terminal — submenu MCP](images/terminal-mcp-menu.jpg)

## Pendaftaran aplikasi ejen satu klik

Submenu `MCP` turut menyenaraikan aplikasi ejen yang dikesan pada pelayan (CLI penyunting dan seumpamanya). Mengklik aplikasi menaip perintah `mcp add` sedia guna ke terminal — **tanpa baris baharu di penghujung**; semak ia dan tekan Enter sendiri untuk mendaftarkan perkhidmatan MCP pelayan ini dengan ejen itu. Entri bersembunyi selagi aplikasi sudah berjalan dalam sesi, supaya teks tidak mendarat dalam UI aplikasi.

> Gelung tipikal: salin Terminal ID kepada ejen anda → `send_command` memulakan binaan → `wait_terminal` sehingga selesai → `get_screen` membaca hasilnya. Bersama Pengesahan Automatik (lihat bab terminal), tugas interaktif panjang berjalan hampir tanpa pengawasan.
