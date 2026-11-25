Ada dua tingkat kustomisasi: menambal sumber addon yang ada, atau menulis yang baru mencontoh bawaan lalu mengunggahnya.

## Patch sumber panas

Tombol pensil pada kartu addon mana pun membuka sumber JavaScript-nya untuk disunting. Menyimpan menyimpan snapshot yang menimpa yang asli — untuk addon bawaan maupun unggahan — dan perubahan langsung berlaku tanpa restart. Berguna untuk menyetel prompt, memperbaiki URL, atau memprototipe ide di sistem hidup.

`Reset` membuang snapshot dan memulihkan sumber asli (bawaan saja — addon unggahan memang milik Anda).

![Menyunting sumber addon — snapshot tersimpan menimpa yang asli](images/addon-edit-source.jpg)

## Menulis addon sendiri

Addon adalah dua berkas:

- `<id>.json` — manifest: `id`, `name`, `description`, `author`, `email`, `date`, `type`, dan `config_fields` (key, label, tipe bidang, bawaan, deskripsi) yang diubah Web UI menjadi formulir;
- `<id>.js` — sumber: berkas JavaScript biasa (dijalankan mesin QuickJS tersemat, dengan jembatan `addon.get_config()` ke bidang terkonfigurasi) yang mengekspor fungsi yang dipanggil tipenya — mis. `chat_completion()` untuk penyedia LLM, daftar alat dan panggilan jembatan untuk addon MCP.

Awalan tercepat adalah mengekspor addon yang ada: tombol `Export` pada kartunya mengunduh zip berisi manifest dan sumber sekaligus. Pilih bawaan yang paling dekat dengan tujuan Anda (mereka juga ada di `crates/vterm-server/src/addon/builtins/`), sesuaikan id, name, type, dan fungsi yang diekspor, lalu pasang hasilnya dengan `Upload Addon`. Addon unggahan dapat dihapus lagi dengan tombol tempat sampah.

![Mengekspor addon — manifest dan sumber JavaScript dalam satu zip](images/addon-export.jpg)

## Penyimpanan

Addon unggahan dan snapshot sumber hidup di folder `addons/` dari direktori konfigurasi sebagai `<id>.json` + `<id>.js`, sehingga bertahan dari restart. Untuk id bawaan hanya penimpaan `.js` yang dihormati — bawaan mempertahankan manifestnya sendiri.
