Terminal ialah teras Vibe Terminal: setiap tab menempatkan satu atau lebih sesi terminal, boleh dipecahkan kepada anak tetingkap, atau diseret keluar ke tetingkapnya sendiri. Terminal mencapai pelayan tempatan serta pelayan jauh dan dikongsi melalui relay.

## Mencipta terminal

- Klik butang `+` di bar tajuk untuk membuka tab dengan profil lalai;
- Klik chevron di sebelah `+` untuk menu profil: profil terminal dikumpulkan setiap pelayan (tempatan, bersambung, dikongsi) — buka kumpulan dan pilih profil untuk membuka terminal pada pelayan itu;
- Seret folder tempatan ke bar tajuk dan lepaskannya untuk membuka tab baharu dengan direktori kerja itu (pelayan tempatan sahaja; item yang dilepaskan mesti folder).

![Bar tajuk — butang tab baharu dan menu profil](images/terminal-usage-creation.jpg)

## Penangkapan tetikus dan menu konteks

- Klik kanan dalam terminal membuka menu konteksnya: tampal, pecahkan anak tetingkap, perintah pantas, koleksi pemboleh ubah persekitaran, salin-semua-teks, baris persediaan MCP dan banyak lagi, bergantung pada sesi;
- Aplikasi yang menawan tetikus (opencode, pi, vim, tmux, htop, …) menerima peristiwa tetikus sebaliknya — tahan `Shift` untuk tetap melakukan operasi biasa semasa aplikasi sedemikian berjalan:
  - **Menyalin teks**: tahan `Shift` dan pilih teks, kemudian klik di tempat lain — `Shift` masih ditahan — untuk membatalkan pilihan; pilihan disalin ke papan klip;
  - **Menu konteks**: `Shift` + klik kanan atau `Ctrl` + klik kanan membuka menu konteks; klik kanan sambil menahan butang kiri juga membukanya (pilihan dipadamkan terlebih dahulu);
  - **Pautan**: `Shift` + klik membuka pautan seperti biasa.

## Pecahan dan penyusunan seret

- **Pecah**: klik kanan dalam terminal → `Split` → `Left` / `Right` / `Top` / `Bottom` membahagikan tab kepada anak tetingkap, setiap satu terminal bebas;

  ![Membahagikan tab — menu Split klik kanan dan anak tetingkap yang terhasil](images/terminal-usage-split.gif)

- **Gabung dengan seret**: seret tab ke terminal lain dan zon dok muncul (kiri/kanan/atas/bawah); lepaskan untuk menggabungkan tab ke tab sasaran sebagai anak tetingkap baharu;

  ![Penggabungan seret — tab yang dilepaskan ke terminal lain menjadi anak tetingkap baharu](images/terminal-usage-merge.mp4)

- **Tanggalkan ke tetingkap**: seret tab keluar dari jalur tab (atau ke tetingkap lain) dan keseluruhan tab meninggalkan tetingkapnya sebagai yang baharu. Sesi sentiasa berjalan di pelayan — pecahan, penggabungan dan seretan tidak pernah mengganggu perintah yang sedang berjalan.

## Tab dan kumpulan tab

- **Tab** — satu tab terminal, mungkin dengan beberapa anak tetingkap terbahagi;
- **Kumpulan tab** — bekas yang menyusun tab. Secara lalai kumpulan mengikut pelayan: tab pelayan tempatan berada dalam `Ungrouped`, dan setiap pelayan dikongsi mendapat kumpulan `Name (Server)`; kumpulan tersuai disokong;
- **Kumpulan dan tetingkap** — setiap tetingkap menunjukkan satu kumpulan aktif melalui pemilih kumpulan bar tajuk (atau `All`); `Move Group to New Window` merenggangkan keseluruhan kumpulan ke tetingkapnya sendiri, menauliahkan satu tetingkap atau monitor untuk satu mesin;
- **Kumpulan dan pelayan** — kumpulan mengingati pelayannya, jadi menukar kumpulan memfokuskan semula ruang kerja sesebuah pelayan; menu profil mengikut kumpulan aktif dan hanya menunjukkan profil pelayan yang berkaitan.

![Pemilih kumpulan — senarai kumpulan dan pindah-ke-tetingkap-baharu](images/terminal-usage-groups.jpg)
