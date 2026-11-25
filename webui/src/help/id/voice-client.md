Tekan `Ctrl` dua kali beruntun cepat (dalam ~300 ms) di mana pun di klien desktop untuk membuka dialog input suara.

## Merekam

Dialog langsung mulai merekam: gelombang cair mengikuti suara Anda, pemilih atas mengganti mikrofon (pilihan diingat), dan pengatur waktu menampilkan durasi. `Stop` mengakhiri klip dan mengirimnya untuk transkripsi (`Reset` mengulang).

![Fase perekaman — gelombang, pemilih mikrofon, dan pengatur waktu](images/voice-client-record.jpg)

## Pemrosesan

Animasi aliran partikel bermain selama server mentranskripsi audio.

## Penyuntingan

Transkrip mendarat di kotak teks yang dapat disunting:

- pil mode beralih antara `original` dan `polished`; `Polish` menjalankan LLM sekali, dan `Auto polish` melakukannya tepat setelah tiap transkripsi;
- `Re-record` membuang teks dan mulai dari awal; `Insert` merekam lagi dan menyambung transkrip baru ke teks yang ada di posisi karet (menggantikan seleksi apa pun), sehingga Anda dapat memperluas atau mengoreksi yang sudah didikte — karet mendarat tepat setelah potongan yang disisipkan;
- arahkan kursor ke kotak teks untuk mengungkap tombol salin di sudut kanan atasnya: klik untuk menyalin teks dan ia berubah jadi centang; menyunting teks atau berganti mode mempersenjatainya kembali untuk salinan lain;
- `Submit` (atau `Ctrl+Enter`) mengetik teks ke terminal aktif; `Esc` menutup dialog.

![Fase penyuntingan — pil mode, auto polish, baris aksi dengan Insert, dan tombol salin saat diarahkan](images/voice-client-editing.jpg)

## Penyiapan polesan

`Polish` butuh LLM polesan suara: di bagian `LLM` Web UI, aktifkan `Enable voice text polish` dalam daftar per-fitur (penyedia biasa plus sakelar ini saja yang diperlukan). Tanpanya tombol `Polish` tetap kelabu dan opsi auto polish tersembunyi.
