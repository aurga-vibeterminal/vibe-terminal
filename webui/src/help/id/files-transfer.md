Transfer berkas antara mesin Anda dan server adalah seret & lepas, dan arah adalah maknanya: **seret masuk = unggah ke server, seret keluar = unduh ke mesin ini**. Transfer menumpang koneksi klien-server terenkripsi; server jarak jauh dan berbagi berperilaku seperti lokal.

## Mengunggah: seret masuk

Seret berkas (atau seluruh folder) dari pengelola berkas OS ke daftar berkas dan lepaskan:

- di mode pohon, **menjatuhkan pada baris folder** mengunggah ke folder itu; **menjatuhkan di akar** (garis sisip antar baris) mengunggah ke direktori akar saat ini;
- baris transfer muncul di bawah daftar menampilkan `Uploading`, nama berkas, dan bilah kemajuan;
- penyelesaian menampilkan `Uploaded n file(s)`; kegagalan menyertakan alasan (mis. izin ditolak) dan daftar disegarkan memperlihatkan berkas baru.

## Mengunduh: seret keluar

Tekan satu berkas di daftar, seret keluar jendela klien, dan jatuhkan ke pengelola berkas OS (atau desktop):

- berkas mengalir turun dari server ke tempat Anda menjatuhkannya;
- baris transfer menampilkan kemajuan dengan cara sama; **baris unduhan selesai dapat diklik** dan membuka folder unduhan lokal;
- pilih banyak dan seret beberapa berkas sekaligus.

![Kemajuan transfer — baris unggah/unduh](images/files-transfer.jpg)

> Seret & lepas memindahkan berkas tunggal antara mesin ini dan satu server; untuk memindahkan data antara dua server, gabungkan Publikasikan dengan Server HTTP atau perintah terminal (scp/rsync).
