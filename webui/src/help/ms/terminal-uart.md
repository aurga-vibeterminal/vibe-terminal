Siri UART menukar konsol siri peranti (port nyahpepijat, papan terbenam, SBC) kepada tab terminal yang boleh anda tonton dan berinteraksi seperti terminal lain.

## Mengkonfigurasi port

Dalam bahagian `UART` Web UI → `Add UART Port`:

- `Name` — dipaparkan sebagai tajuk terminal (cth. "Debug Console");
- `Serial Port` — pilih daripada port yang dikesan (bilangan dipaparkan di bahagian atas halaman);
- `Baud Rate` / `Data Bits` / `Parity` / `Stop Bits` / `Flow Control` — padankan perisian tegar peranti;
- `Terminal Type` dan `Encoding` — pengekodan lalainya Auto (UTF-8) dan boleh mengikut perisian tegar;
- `Show Advanced Options` mendedahkan parameter siri lanjutan.

Port tersimpan muncul dalam senarai `Configured UART Ports` dan boleh disunting atau dibuang pada bila-bila masa.

![Halaman tetapan UART — borang port](images/terminal-uart-config.jpg)

## Penggunaan dalam klien

Port terkonfigurasi terbuka sebagai tab terminal seperti sesi pelayan biasa:

- Data siri dipaparkan ketika tiba; input papan kekunci terus menuju port;
- menu klik kanan menawarkan `Clear` (kosongkan skrin) dan submenu `Encoding` (tukar pengekodan secara langsung; yang semasa bertanda semak);
- sesi UART tiada ciri shell (tiada Perintah Pantas, Apps atau item menu Envs).

Kegunaan tipikal: konsol penghala/suis, log papan terbenam, pemantauan jig flash.
