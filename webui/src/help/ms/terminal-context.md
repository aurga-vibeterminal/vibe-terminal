Ejen kod (Claude Code, Codex, Kimi, Pi, OpenCode…) hanya tahu apa yang sampai ke inputnya. `Menambah Konteks` ialah cara menyerahkan laluan fail dan cebisan kod tanpa menaip semula: seret fail daripada `Explorer`, atau seret teks terpilih daripada penyunting kod dan paparan diff Git, terus ke terminal ejen atau input sembang Copilot.

## Menyeret fail masuk

Tekan satu fail — atau pilihan pelbagai — dalam senarai fail `Explorer` dan seretnya:

- **Ke atas terminal**: laluan penuh ditaip pada titik lepasan, dipisahkan ruang, sedia menjadi sebahagian gesaan anda yang seterusnya kepada ejen;
- **Ke atas input sembang Copilot**: laluan disisipkan ke draf mesej, satu setiap baris, supaya anda himunkan gesaan pelbagai fail sebelum menghantar.

Laluan yang diseret daripada pengurus fail OS anda dan dilepaskan pada terminal disebut secara automatik, jadi nama yang mengandungi ruang kekal utuh.

![Menyeret fail daripada senarai Explorer ke terminal menaip laluan penuhnya](images/terminal-context-drag-file.gif)

## Menyeret kod terpilih sebagai konteks

Pilih teks dalam penyunting kod atau diff Git, kemudian tekan pilihan itu sendiri dan seretnya ke terminal:

- apabila terminal **menjalankan ejen kod**, lepasan menyisipkan *blok konteks* — laluan fail, julat baris dan cebisan berbalut pagar kod:

````text
File: crates/vterm/src/ui/terminal_view.rs
Lines: 558-566
```
fn can_accept_native_drop(&self, info: &DndInfo) -> bool { … }
```
````

- apabila terminal pada **gesaan shell tulen**, lepasan menaip teks terpilih mentah, jadi gerak isyarat yang sama berfungsi untuk perintah biasa;
- apabila diff dilihat pada komit tertentu, blok turut membawa baris `Commit:` yang memaku semakan yang tepat.

![Menyeret pilihan penyunting ke terminal ejen melepaskan blok konteks File/Lines](images/terminal-context-drag-selection.mp4)

## Salin konteks

Blok yang sama hidup dalam menu klik kanan: `Copy Context` dalam penyunting kod dan paparan diff menyalinnya ke papan klip. Tampalkan ke input sembang Copilot — atau ejen lain — apabila seret kurang selesa. Animasi di atas menunjukkan blok terhasil dalam input ejen.

## Petua

- Pilihan pelbagai dalam `Explorer` untuk melepaskan beberapa laluan sekaligus;
- input sembang Copilot terus dihantar ke terminal seperti input ditaip (lihat `Sejarah Ejen Kod`), jadi konteks yang ditambah di sana sampai ke ejen tepat seperti anda menaipnya;
- gabungkan dengan `Pengesahan Automatik` supaya ejen bertindak atas konteks yang dilepaskan tanpa gesaan lanjutan.
