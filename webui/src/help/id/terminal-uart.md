Serial UART mengubah konsol serial perangkat (port debug, papan embedded, SBC) menjadi tab terminal yang dapat Anda saksikan dan gunakan seperti terminal lainnya.

## Mengonfigurasi port

Di bagian `UART` Web UI → `Add UART Port`:

- `Name` — ditampilkan sebagai judul terminal (mis. "Debug Console");
- `Serial Port` — pilih dari port yang terdeteksi (jumlahnya ditampilkan di atas halaman);
- `Baud Rate` / `Data Bits` / `Parity` / `Stop Bits` / `Flow Control` — sesuaikan firmware perangkat;
- `Terminal Type` dan `Encoding` — enkode bawaannya Auto (UTF-8) dan dapat mengikuti firmware;
- `Show Advanced Options` membuka parameter serial lanjutan.

Port tersimpan muncul di daftar `Configured UART Ports` dan dapat disunting atau dihapus kapan saja.

![Halaman pengaturan UART — formulir port](images/terminal-uart-config.jpg)

## Penggunaan di klien

Port terkonfigurasi terbuka sebagai tab terminal seperti sesi server biasa:

- Data serial ditampilkan saat tiba; masukan keyboard langsung menuju port;
- menu klik kanan menawarkan `Clear` (mengosongkan layar) dan submenu `Encoding` (ganti enkode langsung; yang aktif dicentang);
- sesi UART tanpa fitur shell (tanpa Perintah Cepat, Apps, atau item menu Envs).

Kegunaan tipikal: konsol router/switch, log papan embedded, pemantauan jig flash.
