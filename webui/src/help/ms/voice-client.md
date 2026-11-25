Tekan `Ctrl` dua kali berturutan pantas (dalam ~300 ms) di mana-mana dalam klien desktop untuk membuka dialog input suara.

## Merakam

Dialog terus mula merakam: gelombang cecair mengikuti suara anda, pemilih atas menukar mikrofon (pilihan diingati), dan pemasa menunjukkan masa berlalu. `Stop` mengakhiri klip dan menghantarnya untuk transkripsi (`Reset` memulakan semula).

![Fasa rakaman — gelombang, pemilih mikrofon dan pemasa](images/voice-client-record.jpg)

## Pemprosesan

Animasi aliran zarah bermain selagi pelayan mentranskripsi audio.

## Penyuntingan

Transkrip mendarat dalam kotak teks boleh sunting:

- pil mod menogol antara `original` dan `polished`; `Polish` menjalankan LLM sekali, dan `Auto polish` melakukannya tepat selepas setiap transkripsi;
- `Re-record` membuang teks dan bermula semula; `Insert` merakam lagi dan menjahit transkrip baharu ke teks sedia ada pada karet (menggantikan sebarang pilihan), supaya anda boleh melanjutkan atau membetulkan apa yang sudah didikte — karet mendarat tepat selepas cebisan yang disisipkan;
- layari kursor ke kotak teks untuk mendedahkan butang salin di penjurunya kanan atas: klik untuk menyalin teks dan ia bertukar menjadi tanda semak; menyunting teks atau menukar mod mempersenjatainya semula untuk salinan lain;
- `Submit` (atau `Ctrl+Enter`) menaip teks ke terminal aktif; `Esc` menutup dialog.

![Fasa penyuntingan — pil mod, auto polish, baris tindakan dengan Insert, dan butang salin ketika dilayari](images/voice-client-editing.jpg)

## Persediaan polesan

`Polish` memerlukan LLM polesan suara: dalam bahagian `LLM` Web UI, aktifkan `Enable voice text polish` dalam senarai setiap ciri (penyedia biasa berserta togol ini sahaja yang diperlukan). Tanpanya butang `Polish` kekal kelabu dan pilihan auto polish tersembunyi.
