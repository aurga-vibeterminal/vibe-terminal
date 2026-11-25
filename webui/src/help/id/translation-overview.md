Terjemahan mengubah teks yang Anda temui di klien menjadi bahasa pilihan Anda: balasan chat AI, pesan commit, komentar kode, atau seleksi apa pun di editor kode maupun diff. Pekerjaannya dilakukan model terjemahan yang dikonfigurasi di server — klien hanya mengirim teks, merender hasilnya, dan menyimpannya di cache agar Anda bisa bolak-balik antara teks asli dan terjemahan.

## Di mana terjemahan tersedia

- **Panel Copilot** — setiap header pesan (pesan Anda, balasan model, dan blok `Berpikir...`) memiliki ikon terjemahan. Satu klik mengganti isi pesan dengan terjemahannya; satu klik lagi memulihkan aslinya;
- **Draf pesan commit** — kotak commit di panel Git menampilkan tombol terjemahan begitu draf Anda membentang beberapa baris. Tombol ini menerjemahkan seluruh draf di tempat, dan satu klik lagi mengembalikan susunan kata asli Anda;
- **Popup detail commit** — mengarahkan kursor ke commit di riwayat Git membuka popup detail; ikon terjemahan di samping hash mengubah pesan commit ke bahasa Anda;
- **Editor kode & tampilan diff** — pilih apa saja (seleksi blok juga termasuk), klik kanan dan pilih `Terjemahkan`. Lencana bola kecil menandai baris yang diterjemahkan di gutter, dan kartu berisi terjemahan terbuka di samping seleksi.

## Cara kerjanya

1. Klien mengirim teks ke server — beserta nama file bila teks diseleksi dari editor atau diff, supaya model tahu sintaks komentar apa yang dipakai file itu;
2. LLM terjemahan di server menerjemahkannya ke bahasa sasaran yang dikonfigurasi di Web UI. Format Markdown (judul, daftar, tabel, blok kode) dipertahankan; pada seleksi dari file, penanda komentar seperti `//` atau `#` dikenali sebagai dekorasi dan dibuang, sementara kode dan pengidentifikasi tetap tak tersentuh;
3. Klien merender hasilnya sebagai Markdown di tempat permintaan berasal — mengganti langsung untuk pesan chat, atau dalam kartu yang bisa digulir untuk seleksi editor dan diff.

## Apa yang Anda butuhkan

- Server dengan fitur terjemahan diaktifkan — konfigurasi sekali saja dijelaskan di *Penyiapan Server* (bagian `LLM` pada Web UI);
- Klien desktop yang tersambung ke server itu. Tidak ada yang dikonfigurasi di sisi klien: semua permukaan otomatis memakai setelan server yang sama.

Jika terjemahan belum dikonfigurasi, klien menampilkan error dari server (misalnya di kartu terjemahan) alih-alih hasil.
