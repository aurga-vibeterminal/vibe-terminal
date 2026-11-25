Tangkapan layar dan terminal bertemu dalam satu alur: satu pintasan global menangkap area mana pun di desktop, Anda beri anotasi lalu salin atau simpan — setelah itu tempel gambar dari papan klip langsung ke terminal, tempat ia mendarat sebagai jalur berkas yang bisa dibuka dan dipakai program yang berjalan (termasuk agen kode). Kedua bagian hidup di klien desktop.

## Mengambil tangkapan layar

- Tekan `Shift`+`Alt`+`A` di mana saja: pintasannya global, jadi menyala apa pun aplikasi yang memiliki fokus selama klien desktop berjalan (di macOS, tekan `Shift`+`Command`+`A`);
- Setiap layar meredup di bawah overlay penangkapan. Seret untuk menggambar area pilihan — boleh membentang di beberapa layar — lalu seret di dalamnya untuk memindah, atau tarik genggamannya untuk mengubah ukuran; lencana di atas pilihan menunjukkan ukurannya dalam piksel;
- Tekan `Esc` kapan pun untuk membatalkan.

![Overlay penangkapan — area pilihan dengan lencana ukuran dan bilah anotasi](images/terminal-images-capture.jpg)

## Menambahkan anotasi

Bilah di bawah pilihan membawa di kiri alat anotasi (persegi dan elips), di tengah enam warna, dan di kanan tiga tombol aksi — batal (`✕`), simpan (panah unduh) dan salin (`✓`).

- Pilih alat dan warna, lalu seret di dalam pilihan untuk menggambar; goresan dipotong mengikuti pilihan;
- `Ctrl`+`Z` membatalkan anotasi terakhir, `Ctrl`+`R` mengulanginya;
- Menyeret area baru di luar pilihan saat ini membuangnya — anotasi termasuk — dan memulai pilihan lain.

## Menyalin atau menyimpan

- `Salin` (`✓`) meletakkan tangkapan beranotasi ke papan klip — siap ditempel ke terminal (di bawah), obrolan, atau aplikasi apa pun lainnya;
- `Simpan` (panah unduh) menulis PNG melalui dialog simpan; nama berkas bawaan `vterm-screenshot-<timestamp>.png`.

## Menempelkan gambar ke terminal

- Salin gambar ke papan klip — dari alat tangkapan di atas, peramban, atau penangkapan bawaan sistem operasi;
- Klik kanan terminal dan pilih `Paste Image`; entri ini hanya muncul saat papan klip benar-benar berisi gambar;
- Gambar diunggah ke server sesi dan disimpan di sana sebagai berkas sementara; jalurnya yang dikutip kemudian diketik di posisi kursor terminal persis seperti Anda mengetikkannya — dibungkus penanda bracketed paste saat aplikasi yang berjalan mendukungnya, sehingga shell, editor, dan agen menerimanya bersih;
- Bagi program itu hanya teks: agen kode seperti Claude Code atau opencode membuka gambar dari jalurnya, dan perintah shell biasa dapat memakainya sebagai argumen.

![Klik kanan terminal — entri Paste Image](images/terminal-images-paste.jpg)

## Catatan

- Jalur yang disisipkan hidup di server sesi: untuk server jarak jauh atau WSL, berkas sementara dibuat di mesin itu, tepat di tempat program yang berjalan di terminal dapat mencapainya;
- Dipadukan dengan `Menambahkan Konteks ke Agen`, ini cara tercepat menunjukkan kepada agen sesuatu di layar Anda — tangkap, tempel, dan agen memegang gambar sebagai berkas;
- Di Linux penangkapan bekerja di bawah X11; sesi Wayland native tidak mendukung pintasan global maupun overlay interaktif, dan bila aplikasi X11 lain sudah menggenggam `Shift`+`Alt`+`A`, pintasan tidak dapat didaftarkan.
