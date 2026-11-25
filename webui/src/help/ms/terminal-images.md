Tangkapan skrin dan terminal bertemu dalam satu aliran: satu pintasan global merentasi mana-mana kawasan desktop, anda beri anotasi kemudian salin atau simpan — selepas itu tampal imej daripada papan klip terus ke terminal, di mana ia mendarat sebagai laluan fail yang boleh dibuka dan diguna oleh program yang berjalan (ejen kod termasuklah). Kedua-dua bahagian hidup dalam klien desktop.

## Mengambil tangkapan skrin

- Tekan `Shift`+`Alt`+`A` di mana sahaja: pintasan itu global, jadi ia berpunca tidak kira apl mana yang memegang fokus selagi klien desktop berjalan (pada macOS, tekan `Shift`+`Command`+`A`);
- Setiap skrin malap di bawah lapisan tudung tangkapan. Seret untuk melukis kawasan pilihan — ia boleh merentas beberapa skrin — kemudian seret di dalamnya untuk mengalih, atau tarik pemegangnya untuk mengubah saiz; satu lencana di atas pilihan menunjukkan saiznya dalam piksel;
- Tekan `Esc` pada bila-bila masa untuk membatalkan.

![Lapisan tudung tangkapan — kawasan pilihan dengan lencana saiz dan bar anotasi](images/terminal-images-capture.jpg)

## Menganotasi

Bar di bawah pilihan membawa di kiri alat anotasi (segi empat dan elips), di tengah enam warna, dan di kanan tiga butang tindakan — batal (`✕`), simpan (anak panah muat turun) dan salin (`✓`).

- Pilih alat dan warna, kemudian seret di dalam pilihan untuk melukis; kesan dipotong mengikut pilihan;
- `Ctrl`+`Z` mengundurkan anotasi terakhir, `Ctrl`+`R` mengulanginya;
- Menyeret kawasan baharu di luar pilihan semasa membuangnya — anotasi sekali — dan memulakan pilihan lain.

## Menyalin atau menyimpan

- `Salin` (`✓`) meletakkan tangkapan beranotasi ke papan klip — sedia ditampal ke terminal (di bawah), sembang, atau mana-mana aplikasi lain;
- `Simpan` (anak panah muat turun) menulis PNG melalui dialog simpan; nama fail lalai ialah `vterm-screenshot-<timestamp>.png`.

## Menampal imej ke dalam terminal

- Salin imej ke papan klip — daripada alat tangkapan di atas, pelayar, atau cekupan skrin sendiri sistem pengendalian;
- Klik kanan terminal dan pilih `Paste Image`; entri ini hanya muncul apabila papan klip benar-benar mengandungi imej;
- Imej dimuat naik ke pelayan sesi dan disimpan di sana sebagai fail sementara; laluannya yang disebut kemudian ditaip di kedudukan kursor terminal tepat seperti anda menaipnya — dibalut penanda bracketed paste apabila apl yang berjalan menyokongnya, supaya shell, penyunting dan ejen menerimanya bersih;
- Bagi program, ia cuma teks: ejen kod seperti Claude Code atau opencode membuka imej melalui laluan, dan arahan shell biasa boleh menggunakannya sebagai hujah.

![Klik kanan terminal — entri Paste Image](images/terminal-images-paste.jpg)

## Nota

- Laluan yang disisipkan hidup pada pelayan sesi: untuk pelayan jauh atau WSL, fail sementara dibuat pada mesin itu, tepat di tempat program yang berjalan dalam terminal boleh mencapainya;
- Digabungkan dengan `Menambah Konteks kepada Ejen`, ini cara terpantas menunjukkan kepada ejen sesuatu di skrin anda — cekup, tampal, dan ejen memegang imej sebagai fail;
- Pada Linux, cekupan berfungsi di bawah X11; sesi Wayland asli tidak menyokong pintasan global mahupun lapisan tudung interaktif, dan jika apl X11 lain sudah memegang `Shift`+`Alt`+`A`, pintasan tidak dapat didaftarkan.
