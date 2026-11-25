Sijil daun hanya sah untuk nama-nama yang tersenarai dalam **Nama Alternatif Subjeknya (SAN)**. Apabila klien bersambung, nama hos atau IP dalam URL-nya mesti sepadan tepat dengan salah satu entri itu — wildcard dan padanan awalan tidak disokong. Halaman ini merangkumi pengurusan senarai itu di pelayan; pemeriksaan di sisi klien ada pada halaman terakhir, Pengesahan Ketat pada Klien.

## Pengesanan automatik

Pelayan mengesan secara berterusan nama-nama yang menjadikannya boleh dihubungi — nama hos semasa dan alamat antara muka rangkaian yang aktif — dan menyenaraikannya di bawah `Dikesan secara automatik`. Entri ini mengikut mesin dan tidak boleh diedit secara manual: tambah antara muka rangkaian atau ubah nama hos, senarai akan mengikut pada pembaharuan seterusnya.

## Domain dan alamat IP tambahan

Pengesanan automatik tidak mengetahui semuanya: nama DDNS, entri DNS split-horizon atau alamat di sebalik NAT perlu disenaraikan secara manual. Tambahkan di bawah `Domain dan alamat IP tambahan`:

1. Taip nama hos atau alamat IP (contohnya `server.home.arpa` atau `10.0.0.8`) dan tekan `Tambah`;
2. Ulangi untuk setiap nama yang akan digunakan klien. Buang entri dengan butang tong kitar semula;
3. Semak `Pratonton SAN akhir` — pandangan gabungan entri automatik dan manual yang akan dibawa oleh sijil seterusnya;
4. Tekan `Simpan & Keluarkan Semula`. Menyimpan menandatangani sijil daun baharu 90 hari dengan senarai SAN baharu dan memuatkan semulanya secara terus — sambungan sedia ada kekal berjalan, dan jabat tangan baharu serta-merta menggunakan sijil baharu.

## Pengeluaran semula manual dan status

Panel `Sijil Daun Pelayan` menunjukkan sijil yang aktif: lencana status, subjek, cap jari SHA-256, tempoh sah dan entri SAN. Lencana status bermaksud:

- `Tersedia` — sijil daun yang sah telah dimuatkan dan sedang digunakan;
- `Kemas kini gagal` — percubaan pengeluaran semula yang terakhir gagal; sijil terdahulu kekal aktif dan ralat dipaparkan pada halaman;
- `Belum dijana` — tiada CA atau sijil daun lagi; gunakan `Jana CA & Sijil` untuk mencipta kedua-duanya.

`Keluarkan Semula Sijil Daun` menandatangani sijil daun baharu dengan serta-merta tanpa menunggu pencetus automatik — berguna sejurus selepas mengubah nama hos atau antara muka, atau untuk memasukkan suntingan SAN yang dibuat di luar antara muka web.

## Mengedar CA akar

Klien yang akan menggunakan pengesahan ketat memerlukan sijil awam CA akar. Dapatkan daripada kad `CA Akar Peribadi`:

- `Muat Turun CA Akar` menyimpan fail PEM;
- `Salin CA Akar` meletakkan PEM dalam papan keratan, sedia untuk ditampal ke medan CA klien.

CA akar tidak pernah berubah merentasi pembaharuan sijil daun mahupun suntingan SAN — mengedarnya sekali sudah cukup. Sebelum mendayakan pengesahan ketat di mana-mana, bandingkan cap jari SHA-256 sijil yang ditunjukkan pada halaman ini dengan apa yang diterima klien (lihat halaman terakhir, Pengesahan Ketat pada Klien).

> Muat turun dan papan keratan hanya mengandungi sijil awam CA — bukan kunci peribadi. Serahkannya hanya kepada orang yang anda mahu mampu mengesahkan pelayan anda.
