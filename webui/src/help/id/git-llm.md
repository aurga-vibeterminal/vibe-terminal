Kecerdasan panel Git digerakkan LLM: pesan commit sekali klik, pembacaan diff berbantu AI, dan penyelesaian konflik oleh agen hunk demi hunk. Semuanya bergantung pada penyedia LLM yang dikonfigurasi di Web UI.

## Mengonfigurasi penyedia

Buka bagian `LLM` di Web UI:

- Pilih `Provider` dan isi kredensialnya (bidang ditentukan manifest penyedia); konfigurasinya global — klasifikasi terminal, pesan commit Git, asisten obrolan, dan fitur AI lain berbagi penyedia yang sama;
- Penyedia adalah addon bertipe LLM: yang umum terpasang bawaan, dan penyedia kustom dapat diunggah sebagai zip di bagian `Addons` (manifest mendeklarasikan bidang konfigurasi yang diubah Web UI menjadi formulir).

![Bagian LLM Web UI — pilih penyedia dan isi kredensialnya](images/git-llm-provider-config.jpg)

## Fitur LLM di Git

- **Pembuatan pesan commit** — klik tombol AI di samping kotak commit; LLM membaca diff ter-stage dan menulis pesan, diketik ke kotak dengan animasi. Pesan tetap dapat disunting — commit hanya saat Anda puas;
- **Analisis diff** — tombol berkilau di header tampilan diff menganalisis diff saat ini;
- **Agen AI konflik** — mode AI editor merge rebase menyelesaikan hunk satu per satu dengan penjelasan dan keyakinan (lihat Menyelesaikan Konflik).

![Pesan commit yang dihasilkan AI diketik ke kotak commit](images/git-llm-commit-message.jpg)

> LLM hanya membaca konten repositori untuk menghasilkan teks; commit dan merge selalu terjadi saat Anda mengonfirmasinya.
