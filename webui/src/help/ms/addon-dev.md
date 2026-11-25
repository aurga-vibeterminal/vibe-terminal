Ada dua aras penyesuaian: menampal sumber tambahan sedia ada, atau menulis yang baharu mengikut contoh terbina dalam lalu memuat naiknya.

## Tampal sumber secara panas

Butang pensil pada mana-mana kad tambahan membuka sumber JavaScriptnya untuk disunting. Menyimpan menyimpan petikan yang menimpa yang asal — untuk tambahan terbina dalam dan dimuat naik sama sekali — dan perubahan berkuat kuasa serta-merta tanpa mula semula. Berguna untuk melaraskan prom, membetulkan URL atau memprototaip idea pada sistem hidup.

`Reset` membuang petikan dan memulihkan sumber asal (terbina dalam sahaja — tambahan dimuat naik memang milik anda).

![Menyunting sumber tambahan — petikan tersimpan menimpa yang asal](images/addon-edit-source.jpg)

## Menulis tambahan anda sendiri

Tambahan ialah dua fail:

- `<id>.json` — manifest: `id`, `name`, `description`, `author`, `email`, `date`, `type`, dan `config_fields` (key, label, jenis medan, lalai, keterangan) yang dijadikan borang oleh Web UI;
- `<id>.js` — sumber: fail JavaScript biasa (dijalankan enjin QuickJS terbenam, dengan jambatan `addon.get_config()` ke medan terkonfigurasi) yang mengeksport fungsi yang dipanggil oleh jenisnya — cth. `chat_completion()` untuk penyedia LLM, senarai alat dan panggilan jambatan untuk tambahan MCP.

Permulaan terpantas ialah mengeksport tambahan sedia ada: butang `Export` pada kadnya memuat turun zip memuat manifest dan sumber bersama. Pilih terbina dalam yang paling hampir dengan matlamat anda (mereka juga ada di `crates/vterm-server/src/addon/builtins/`), laraskan id, name, type dan fungsi yang dieksport, kemudian pasang hasilnya dengan `Upload Addon`. Tambahan dimuat naik boleh dipadam semula dengan butang tong sampah.

![Mengeksport tambahan — manifest dan sumber JavaScript dalam satu zip](images/addon-export.jpg)

## Penyimpanan

Tambahan dimuat naik dan petikan sumber hidup dalam folder `addons/` direktori konfigurasi sebagai `<id>.json` + `<id>.js`, jadi bertahan daripada mula semula. Bagi id terbina dalam hanya penimpaan `.js` dihormati — terbina dalam mengekalkan manifestnya sendiri.
