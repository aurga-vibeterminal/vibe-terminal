Variabel lingkungan memungkinkan Anda memelihara kumpulan variabel terkelompok per proyek (`ANDROID_HOME`, kunci API, pengaturan proxy) dan menerapkannya atau menghapusnya pada sesi terminal mana pun sekali klik. Mereka hidup di konfigurasi server; nilai dienkripsi saat tersimpan.

## Pemeliharaan di Web UI

Bagian `Environment Variables` mengaturnya sebagai pohon:

- **Grup** — wadah murni, dapat bersarang (klik kanan → `New Group`), untuk mengatur per proyek atau skenario;
- **Koleksi** — daun pohon dan pemegang variabel sesungguhnya (`New Leaf`); hanya koleksi yang dapat diterapkan ke terminal;
- Pasangan kunci/nilai tiap koleksi disunting lewat `Edit Variables` (`Add Variable`); nilai tersembunyi secara bawaan;
- Koleksi dapat diduplikasi untuk menurunkan varian (kumpulan sama dengan satu kunci diubah).

![Halaman Environment Variables — pohon grup/koleksi](images/terminal-env-tree.jpg)

## Menerapkan ke terminal

Klik kanan di dalam terminal → `Envs`:

- submenu mencerminkan pohon dari Web UI; buka koleksi dan pilih `Apply` untuk menyuntikkan variabelnya ke sesi saat ini, atau `Clear` untuk menghapusnya kembali;
- proses yang baru dimulai melihat variabel (program latar depan yang sudah berjalan tidak);
- koleksi muncul terlipat di bawah grupnya; seluruh item `Envs` bersembunyi selama pohon tak punya koleksi.

![Klik kanan terminal — terapkan dan hapus di submenu Envs](images/terminal-env-menu.mp4)
