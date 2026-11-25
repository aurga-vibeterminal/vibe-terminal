Terminal adalah inti Vibe Terminal: setiap tab menampung satu atau beberapa sesi terminal, dapat dipecah menjadi panel, atau diseret ke jendelanya sendiri. Terminal menjangkau server lokal serta server jarak jauh dan berbagi melalui relay.

## Membuat terminal

- Klik tombol `+` di bilah judul untuk membuka tab dengan profil bawaan;
- Klik tanda chevron di samping `+` untuk menu profil: profil terminal yang dikelompokkan per server (lokal, terhubung, berbagi) — buka grup dan pilih profil untuk membuka terminal di server itu;
- Seret folder lokal ke bilah judul dan lepaskan untuk membuka tab baru dengan direktori kerja itu (hanya server lokal; item yang dijatuhkan harus berupa folder).

![Bilah judul — tombol tab baru dan menu profil](images/terminal-usage-creation.jpg)

## Penangkapan mouse dan menu konteks

- Klik kanan di dalam terminal membuka menu konteksnya: tempel, pisahkan panel, Perintah Cepat, koleksi lingkungan, salin-semua-teks, baris pengaturan MCP, dan lainnya, tergantung sesi;
- Aplikasi yang menangkap mouse (opencode, pi, vim, tmux, htop, …) menerima event mouse sebagai gantinya — tahan `Shift` untuk tetap melakukan operasi biasa selama aplikasi seperti itu berjalan:
  - **Menyalin teks**: tahan `Shift` dan pilih teksnya, lalu klik di tempat lain — `Shift` masih ditahan — untuk membatalkan pilihan; pilihan tersalin ke clipboard;
  - **Menu konteks**: `Shift` + klik kanan atau `Ctrl` + klik kanan membuka menu konteks; klik kanan sambil menahan tombol kiri juga membukanya (pilihan dibersihkan terlebih dahulu);
  - **Tautan**: `Shift` + klik membuka tautan seperti biasa.

## Pemisahan dan organisasi seret

- **Pisah**: klik kanan di dalam terminal → `Split` → `Left` / `Right` / `Top` / `Bottom` membagi tab menjadi panel, masing-masing terminal independen;

  ![Memisah tab — menu Split klik kanan dan panel yang dihasilkan](images/terminal-usage-split.gif)

- **Gabung dengan seret**: seret tab ke terminal lain dan zona docking muncul (kiri/kanan/atas/bawah); lepaskan untuk menggabungkan tab ke tab target sebagai panel baru;

  ![Menggabungkan dengan seret — tab yang dijatuhkan ke terminal lain menjadi panel baru](images/terminal-usage-merge.mp4)

- **Lepas ke jendela**: seret tab keluar dari deret tab (atau ke jendela lain) dan seluruh tab meninggalkan jendelanya sebagai jendela baru. Sesi selalu berjalan di server — pemisahan, penggabungan, dan penyeretan tidak pernah mengganggu perintah yang berjalan.

## Tab dan grup tab

- **Tab** — satu tab terminal, mungkin dengan beberapa panel terpisah;
- **Grup tab** — wadah yang mengatur tab. Secara bawaan grup mengikuti server: tab server lokal berada di `Ungrouped`, dan setiap server berbagi mendapat grup `Name (Server)`; grup kustom didukung;
- **Grup dan jendela** — setiap jendela menampilkan satu grup aktif melalui pemilih grup bilah judul (atau `All`); `Move Group to New Window` melepaskan seluruh grup ke jendelanya sendiri, mengabdikan satu jendela atau monitor untuk satu mesin;
- **Grup dan server** — grup mengingat servernya, sehingga berpindah grup memfokuskan ulang ruang kerja server; menu profil mengikuti grup aktif dan hanya menampilkan profil server yang relevan.

![Pemilih grup — daftar grup dan pindah-ke-jendela-baru](images/terminal-usage-groups.jpg)
