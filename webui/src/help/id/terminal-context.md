Agen kode (Claude Code, Codex, Kimi, Pi, OpenCode…) hanya tahu apa yang sampai ke masukannya. `Menambahkan Konteks` adalah cara menyerahkan jalur berkas dan potongan kode tanpa mengetik ulang: seret berkas dari `Explorer`, atau seret teks terpilih dari editor kode dan tampilan diff Git, langsung ke terminal agen atau masukan obrolan Copilot.

## Menyeret berkas masuk

Tekan satu berkas — atau pilihan banyak — di daftar berkas `Explorer` dan seret:

- **Ke terminal**: jalur lengkap diketik di titik jatuh, dipisah spasi, siap menjadi bagian prompt Anda berikutnya ke agen;
- **Ke masukan obrolan Copilot**: jalur disisipkan ke draf pesan, satu per baris, agar Anda merakit prompt multi-berkas sebelum mengirim.

Jalur yang diseret dari pengelola berkas OS dan dijatuhkan pada terminal dikutip otomatis, sehingga nama berisi spasi utuh.

![Menyeret berkas dari daftar Explorer ke terminal mengetikkan jalur lengkapnya](images/terminal-context-drag-file.gif)

## Menyeret kode terpilih sebagai konteks

Pilih teks di editor kode atau diff Git, lalu tekan pilihannya sendiri dan seret ke terminal:

- saat terminal **menjalankan agen kode**, jatuh menyisipkan *blok konteks* — jalur berkas, rentang baris, dan potongan dalam pagar kode:

````text
File: crates/vterm/src/ui/terminal_view.rs
Lines: 558-566
```
fn can_accept_native_drop(&self, info: &DndInfo) -> bool { … }
```
````

- saat terminal di **prompt shell polos**, jatuh mengetikkan teks terpilih mentah, sehingga isyarat sama berlaku juga untuk perintah biasa;
- saat diff dilihat pada commit tertentu, blok juga membawa baris `Commit:` yang mengunci revisi persisnya.

![Menyeret pilihan editor ke terminal agen menjatuhkan blok konteks File/Lines](images/terminal-context-drag-selection.mp4)

## Salin konteks

Blok yang sama hidup di menu klik kanan: `Copy Context` di editor kode dan tampilan diff menyalinnya ke papan klip. Tempel ke masukan obrolan Copilot — atau agen lain — saat menyeret kurang praktis. Animasi di atas menunjukkan blok hasilnya di masukan agen.

## Tips

- Pilihan banyak di `Explorer` untuk menjatuhkan beberapa jalur sekaligus;
- masukan obrolan Copilot terkirim langsung ke terminal seperti ketikan (lihat `Riwayat Agen Kode`), jadi konteks yang ditambahkan di sana sampai ke agen persis seperti Anda mengetikkannya;
- padukan dengan `Konfirmasi Otomatis` agar agen bertindak pada konteks yang dijatuhkan tanpa permintaan lagi.
