Secara lalai, klien menyulitkan setiap sambungan tetapi tidak mengesahkan siapa di hujung sana — sikap percaya pada penggunaan pertama yang sama seperti menerima sijil tandatangan sendiri dalam pelayar web. **Pengesahan ketat** mengubah hal itu: klien menyemak sijil pelayan terhadap CA akar yang anda bekalkan dan menolak untuk bersambung melainkan rantaian, kesahan dan SAN semuanya lulus. Ini menjadikan serangan man-in-the-middle terhadap sambungan praktikalnya mustahil, dengan harga satu langkah manual: memberi klien CA akar milik pelayan.

## Di mana ia disediakan

Pengesahan ketat dikonfigurasikan secara satu-pelayan-satu-tetapan dalam panel penyuntingan pelayan klien, di sebelah alamat dan kunci rahsia:

1. `Sijil CA Akar (PEM)` — medan berbilang baris untuk sijil awam CA akar milik pelayan;
2. `Pengesahan CA ketat (rantaian, kesahan & SAN)` — hanya boleh dipilih selepas CA diisi.

![Panel penyuntingan pelayan dalam klien desktop: medan berbilang baris Sijil CA Akar (PEM) dengan kotak semak pengesahan ketat di bawahnya](images/client-ca-fields.jpg)

## Membawa CA akar ke klien

Dua cara, dari sumber yang sama (halaman `Sijil & TLS` milik pelayan — lihat halaman Pengurusan Domain & SAN):

- **Salin & tampal** — tekan `Salin CA Akar` dalam antara muka web dan tampal PEM ke medan `Sijil CA Akar (PEM)` milik klien;
- **Import rentetan kongsi** — rentetan kongsi yang dihasilkan klien membenamkan CA akar bersama alamat dan kunci. Mengimport kongsi sedemikian mengisi medan CA secara automatik; penerima hanya perlu menandai kotak semak pengesahan ketat.

CA akar sama untuk setiap klien dan tidak pernah berubah merentasi pembaharuan sijil — menetapkannya sekali bagi setiap pelayan sudah cukup.

## Apakah yang mod ketat semak

Dengan kotak semak didayakan, setiap sambungan ke pelayan itu mesti membentangkan sijil yang:

1. Berantai kepada CA akar yang dikonfigurasikan — sijil yang ditandatangani oleh sesuatu yang lain ditolak;
2. Berada dalam tempoh sahnya — sijil yang telah tamat tempoh atau belum sah ditolak;
3. Membawa SAN yang sepadan dengan alamat yang anda sambungkan — sijil yang dikeluarkan untuk nama lain ditolak.

Jika mana-mana semakan gagal, sambungan ditolak dengan ralat yang menerangkan semakan mana yang gagal. Tanpa kotak semak (atau tanpa CA), sambungan kekal disulitkan tetapi semakan-semakan ini dilangkau.

## Sahkan cap jari

Sebelum mempercayai CA akar, sahkan bahawa anda mendapat yang sebenar: halaman `Sijil & TLS` menunjukkan cap jari SHA-256 milik CA akar. Bandingkan dengan PEM yang anda ingin tampalkan (atau PEM yang dibenamkan dalam kongsi yang diterima). Jika berbeza, jangan dayakan pengesahan ketat — dapatkan CA melalui saluran yang dipercayai sebagai gantinya.

## Kegagalan biasa

- `SAN tidak sepadan` — anda bersambung menggunakan nama atau IP yang tidak ada dalam sijil. Tambahkannya pada pelayan (lihat halaman Pengurusan Domain & SAN) dan keluarkan semula;
- `Sijil tamat tempoh atau belum sah` — jam pelayan tidak tepat, atau pembaharuan sijil daun gagal. Semak status sijil dalam antara muka web;
- `Tidak ditandatangani oleh CA yang dikonfigurasikan` — CA dalam medan milik pelayan lain, atau sambungan sedang disekat. Bandingkan cap jari sebelum meneruskan.
