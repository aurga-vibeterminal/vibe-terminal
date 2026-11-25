Setelah server dikonfigurasi, semua permukaan klien menawarkan terjemahan lewat ikon bola yang sama. Hasil di-cache per item, sehingga bolak-balik tidak pernah mengulang permintaan.

## Pesan di panel chat

Setiap header pesan memiliki ikon terjemahan — di samping waktu pada pesan Anda, di kanan nama pengirim dan waktu pada balasan model, dan di kanan label `Berpikir...` pada blok penalaran:

- **Klik pertama** mengirim Markdown mentah pesan ke server dan menukar isi pesan dengan terjemahannya. Ikon menyala selama terjemahan ditampilkan;
- **Klik lagi** memulihkan teks asli;
- Terjemahan menempel pada pesan selama percakapan tetap terbuka, termasuk blok `Berpikir...` setelah Anda bentangkan.

Pesan terjemahan dirender sebagai Markdown biasa, jadi daftar, tabel, dan blok kode mempertahankan bentuknya.

![Pesan chat dengan ikon terjemahan di header dan isi terjemahan di bawahnya](images/translation-chat-message.jpg)

## Draf pesan commit

Di kotak commit panel Git, tombol terjemahan muncul di samping tombol AI begitu draf Anda melebihi satu baris:

- **Klik** untuk menerjemahkan seluruh draf — teks terjemahan menggantikan draf di kotak sebagai pratinjau hanya-baca;
- **Klik lagi** untuk kembali ke susunan kata asli Anda (drafnya sendiri tidak pernah diubah);
- Mengetik dinonaktifkan selama pratinjau ditampilkan; kembalikan dulu, lalu lanjutkan mengedit.

![Kotak commit dengan tombol terjemahan dan pratinjau terjemahan](images/translation-commit-draft.jpg)

## Popup detail commit

Arahkan kursor ke commit di riwayat Git untuk membuka popup detail. Di samping tombol salin hash ada ikon terjemahan: klik untuk menerjemahkan pesan commit di dalam popup; klik lagi untuk melihat aslinya. Pesan terjemahan dirender seperti aslinya — baris subjek, isi Markdown, dan blok kode termasuk.

![Popup detail commit — tombol salin dan terjemahkan pada baris hash](images/translation-git-tip.jpg)

## Seleksi di editor kode

Pilih teks di editor kode — seleksi biasa maupun blok (persegi) sama-sama berfungsi — klik kanan dan pilih `Terjemahkan` (entri pertama menu):

1. Lencana bola padat muncul di atas nomor baris pertama yang diseleksi;
2. Kartu terjemahan langsung terbuka dengan penampung *Translating…* dan terisi saat server menjawab;
3. Kartunya lengket: tetap di tempat saat Anda menggulir, dan hanya tombol tutupnya, `Esc`, atau klik di luar yang menutupnya. Mengarahkan kursor ke lencana membukanya kembali.

Setiap seleksi yang diterjemahkan mendapat lencananya sendiri, dan menerjemahkan baris yang sama lagi hanya menampilkan ulang kartu yang di-cache. Lencana hilang saat Anda membuka file lain.

![Seleksi di editor kode dengan Terjemahkan di menu konteks](images/translation-editor-menu.jpg)

## Seleksi di tampilan diff

Tampilan diff bekerja dengan cara yang sama: pilih baris di badan diff, klik kanan, `Terjemahkan`. Lencana menggantikan nomor baris baru pada baris pertama yang diseleksi, dan kartu hasilnya menambat di bawahnya. Seleksi di mode edit juga bisa diterjemahkan; berpindah ke file atau commit lain menghapus lencana.

![Diff dengan seleksi yang diterjemahkan — lencana gutter dan kartu terjemahan](images/translation-diff-card.jpg)

## Apa yang diterjemahkan

Server memberi tahu model asal teksnya:

- **Prosa** (pesan chat, pesan commit) — format Markdown dipertahankan dan hanya bahasa alaminya yang diterjemahkan;
- **Seleksi dari file** (editor, diff) — nama file ikut bersama permintaan, jadi model tahu penanda komentar file itu (`//`, `#`, `--`, …). Penanda komentar dibuang dan teks komentarnya diterjemahkan sebagai prosa; Markdown di dalam komentar (judul, daftar, blok kode) bertahan. Jika seleksinya kode biasa, hanya komentar dan literal string yang diterjemahkan — pengidentifikasi dan strukturnya tetap tak tersentuh.
