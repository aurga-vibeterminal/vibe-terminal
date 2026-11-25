Area `CHANGES` panel Git mencakup putaran commit sehari-hari: meninjau perubahan, staging, membuang, stash, dan commit dengan pesan.

## Stage dan unstage

- Perubahan belum ter-stage didaftar di bawah `CHANGES`, tiap berkas bertanda lencana status — `M` diubah, `A` ditambahkan, `D` dihapus, `?` tak terlacak, `!` konflik (baris submodul membawa tag `SUB` ekstra);
- Mengarahkan kursor ke baris mengungkap tombol aksi inline: `+` me-stage berkas belum ter-stage, `−` men-unstage yang ter-stage, dan `×` pada baris belum ter-stage membuangnya; tombol `Stage All Changes` / `Unstage All Changes` di header bagian menangani semuanya sekaligus;
- `Ctrl`-klik mengalihkan berkas ke pilihan banyak dan `Shift`-klik memilih rentang — tombol aksi lalu berlaku pada setiap berkas terpilih;
- Mengklik nama berkas membuka diff-nya agar Anda meninjau perubahan sebelum memutuskan;
- Klik kanan berkas menawarkan `View File History`, `Stash This File`, `Copy Full Path` / `Copy Relative Path`, `Reveal in File Explorer`, dan pintasan `cd` yang melompatkan terminal ke folder berkas.

![Bagian STAGED CHANGES — berkas ter-stage dengan tombol Unstage tiap baris](images/git-staging-staged.jpg)

## Commit

1. Stage yang seharusnya masuk;
2. ketik pesan di kotak commit (`Ctrl+Enter` commit) atau klik tombol AI untuk menghasilkan (lihat Bantuan LLM);
3. klik `Commit`.

![Kotak commit — masukan pesan dengan tombol buat AI](images/git-staging-commit.jpg)

Setelah commit, arahkan kursor ke header riwayat — bilah dengan nama cabang di atas riwayat commit — dan tombol `Fetch`, `Pull`, dan `Push` memudar masuk di kanannya. `Push` langsung mengirim cabang saat ini ke upstream-nya; untuk opsi seperti `Force with lease`, gunakan dialog push daftar cabang (lihat Push, Checkout & Lainnya).

![Mengarahkan kursor ke header riwayat mengungkap tombol Fetch, Pull, dan Push](images/git-staging-push.jpg)

## Stash

- Tombol arsip di header `CHANGES` membuka dialog `Stash All Changes`, tempat pesan opsional menggambarkan stash;
- klik kanan berkas belum ter-stage yang dipilih menawarkan `Stash This File` (atau `Stash <n> Selected Files` untuk pilihan banyak) untuk me-stash jalur itu saja — baris submodul dikecualikan;
- stash terkumpul di bawah `STASHES` sebagai entri `WIP on <cabang>`; membuka satu mendaftar berkas yang ditangkap, dan mengklik berkas menampilkan diff-nya;
- mengarahkan kursor ke baris stash memperlihatkan `Apply` (pulihkan dan simpan), `Pop` (terapkan dan hapus), dan `Drop` (hapus tanpa menerapkan) — `Pop` dan `Drop` meminta konfirmasi dulu, dan menu klik kanan menambah `View Diff`;
- bagian `STASHES` tetap tersembunyi selama daftar stash kosong.

![Dialog Stash All Changes dengan pesan opsionalnya](images/git-staging-stash-all.jpg)

![Bagian STASHES — stash terbuka ke berkasnya, dengan tombol Apply, Pop, dan Drop](images/git-staging-stashes.jpg)

![Pop menerapkan stash dan menghapusnya dari daftar setelah konfirmasi](images/git-staging-stash-pop.jpg)

## Membuang

`Discard Changes` pada berkas belum ter-stage membuang seluruh perubahan yang belum di-commit di dalamnya — ada dialog konfirmasi dan aksinya **tidak dapat dibatalkan**.

> Commit hanya menyertakan konten ter-stage; perubahan belum ter-stage tetap di pohon kerja.
