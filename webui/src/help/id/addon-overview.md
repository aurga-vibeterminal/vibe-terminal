Addon memperluas server dengan kemampuan baru tanpa menyentuh inti: penyedia LLM, backend pengenalan suara, updater DNS dinamis, permukaan alat MCP tambahan, dan deteksi aplikasi agen kode semuanya hidup sebagai addon. Bagian `Addons` pada Web UI mengelolanya.

## Kategori

Setiap addon mendeklarasikan tipe, dan bagian mengelompokkan kartu sesuai itu:

- **Applications** — mengajarkan pengamat terminal mengenali CLI agen kode (Claude Code, Codex, Kimi Code, Pi, dan OpenCode bawaan): deteksi proses, status sibuk/menganggur, hook otomatisasi;
- **LLM Providers** — backend penyelesaian obrolan yang dipilih fitur AI (penyedia kompatibel OpenAI bawaan);
- **Voice / ASR** — backend pengenalan suara (addon bawaan `vibe-voice` menunjuk server ASR lokal yang diawasi);
- **Dynamic DNS** — menjaga hostname mengikuti alamat saat ini (Cloudflare bawaan);
- **MCP** — permukaan alat MCP tambahan yang dilayani di host MCP (EasyEDA Pro bawaan);
- lainnya masuk grup **Other**.

## Addon bawaan dan unggahan

- Addon **bawaan** ikut di dalam program dan selalu tersedia. Manifest dan sumbernya tetap murni — sumber yang ditimpa dapat dipulihkan kapan saja;
- Addon **unggahan** milik Anda: zip berisi manifest dan sumber JavaScript, dipasang lewat tombol `Upload Addon`;
- Lencana `Modified` menandai addon yang sumbernya Anda timpa.

Setiap kartu menampilkan tipe, lencana, dan sakelar aktifkan; saat dibuka memperlihatkan deskripsi, penulis, bidang konfigurasi, dan kode sumber.

![Bagian Addons — kartu berkelompok dengan lencana dan sakelar aktifkan](images/addon-section.jpg)

## Konfigurasi

Manifest addon mendeklarasikan bidang konfigurasinya, dan kartu yang dibuka mengubahnya menjadi formulir. Nilai disimpan per addon dan hanya berlaku selama diaktifkan; bidang sandi tetap disamarkan. Menonaktifkan addon mempertahankan konfigurasinya — saat diaktifkan kembali, Anda melanjutkan dari sebelumnya.
