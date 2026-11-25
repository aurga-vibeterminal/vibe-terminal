Pemboleh ubah persekitaran membolehkan anda mengekalkan set pemboleh ubah berkelompok setiap projek (`ANDROID_HOME`, kunci API, tetapan proksi) dan menerapkannya atau mengosongkannya pada sebarang sesi terminal dengan satu klik. Ia hidup dalam konfigurasi pelayan; nilai disulitkan semasa rehat.

## Penyelenggaraan dalam Web UI

Bahagian `Environment Variables` menyusunnya sebagai pokok:

- **Kumpulan** — bekas tulen, boleh bersarang (klik kanan → `New Group`), untuk penyusunan mengikut projek atau senario;
- **Koleksi** — daun pokok dan pemegang pemboleh ubah sebenar (`New Leaf`); hanya koleksi boleh diterapkan ke terminal;
- Pasangan kunci/nilai setiap koleksi disunting melalui `Edit Variables` (`Add Variable`); nilai tersembunyi secara lalai;
- Koleksi boleh diduplikasi untuk menerbitkan varian (set sama dengan satu kunci diubah).

![Halaman Environment Variables — pokok kumpulan/koleksi](images/terminal-env-tree.jpg)

## Menerapkan ke terminal

Klik kanan dalam terminal → `Envs`:

- submenu mencerminkan pokok daripada Web UI; buka koleksi dan pilih `Apply` untuk menyuntik pemboleh ubahnya ke sesi semasa, atau `Clear` untuk mengeluarkannya semula;
- proses yang baru dimulakan melihat pemboleh ubah (program latar hadapan yang sedang berjalan tidak);
- koleksi muncul terlipat di bawah kumpulannya; keseluruhan item `Envs` bersembunyi selagi pokok tiada koleksi.

![Klik kanan terminal — terapkan dan kosongkan dalam submenu Envs](images/terminal-env-menu.mp4)
