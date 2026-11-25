Secara lalai, pelayan dikongsi menyulitkan sambungan hulunya ke relay tetapi tidak mengesahkan siapa di hujung sana — sikap percaya pada penggunaan pertama yang sama seperti menerima sijil tandatangan sendiri dalam pelayar web. **Strict Relay TLS** mengubah hal itu: pelayan dikongsi menyemak sijil relay terhadap CA akar yang anda bekalkan dan menolak untuk bersambung melainkan rantaian, kesahan dan nama hos semuanya lulus. Ini menjadikan serangan man-in-the-middle terhadap pautan relay praktikalnya mustahil, dengan harga satu langkah manual: memberi pelayan dikongsi CA akar milik relay. Halaman ini menerapkan idea yang sama dengan halaman sebelumnya, Pengesahan Ketat pada Klien, pada pautan relay hulu milik pelayan dikongsi.

## Di mana ia disediakan

Pengesahan ketat dikonfigurasikan pada pelayan dikongsi — mesin yang `Relay Address`-nya menunjuk ke relay. Dalam antara muka webnya, buka bahagian `Server`: di bawah `Relay Secret Key` terdapat baris `Strict Relay TLS` yang menunjukkan keadaan `On` / `Off` semasa. Kembangkan untuk melihat:

1. `Upstream Relay Root CA` — medan berbilang baris untuk sijil awam CA akar milik relay;
2. Butang togol `Off` / `On` — `On` hanya boleh dipilih setelah PEM dapat dibaca sebagai sijil yang sah.

![Antara muka web — blok Strict Relay TLS yang dikembangkan dalam bahagian Server: medan Upstream Relay Root CA dengan togol Off/On dan butang Apply](images/cert-relay-strict-tls.jpg)

Tetapan relay tidak pernah disimpan ketika penyuntingan: semak nilainya kemudian klik `Apply`. Menyimpan terus menyambung semula relay hulu dengan tetapan baharu.

## Membawa CA akar ke pelayan dikongsi

Dua cara, dari sumber yang sama (antara muka web relay itu sendiri):

- **Menampal rentetan kongsi** — klik `QR code` dalam antara muka web pelayan relay. Rentetan kongsi membenamkan CA akar relay bersama alamat dan kunci, jadi menampalkannya ke medan `Relay Address` pelayan dikongsi turut mengisi medan CA secara automatik (lihat Panduan Persediaan di bahagian Relay). Mod ketat kekal dimatikan sehingga anda menyemak CA dan mengaktifkannya sendiri;
- **Salin & tampal** — tekan `Salin CA Akar` pada halaman `Sijil & TLS` milik relay dan tampal PEM ke medan `Upstream Relay Root CA`.

CA akar sama untuk setiap pelayan dikongsi dan tidak pernah berubah merentasi pembaharuan sijil — menetapkannya sekali bagi setiap relay sudah cukup.

Sebelum mempercayai CA akar, sahkan bahawa anda mendapat yang sebenar: halaman `Sijil & TLS` milik relay menunjukkan cap jari SHA-256 milik CA akar. Bandingkan dengan PEM yang anda ingin tampalkan. Jika berbeza, jangan dayakan pengesahan ketat — dapatkan CA melalui saluran yang dipercayai sebagai gantinya.

## Apakah yang mod ketat semak

Dengan togol pada `On`, sambungan `RelayPrimary` hulu mesti membentangkan sijil yang:

1. Berantai kepada CA akar yang dikonfigurasikan — sijil yang ditandatangani oleh sesuatu yang lain, termasuk akar sistem awam, ditolak;
2. Berada dalam tempoh sahnya — sijil yang telah tamat tempoh atau belum sah ditolak;
3. Membawa SAN yang sepadan dengan hos bagi `Relay Address` yang anda sambungkan — sijil yang dikeluarkan untuk nama lain ditolak.

Satu had: hanya sambungan relay hulu yang disahkan. Sambungan balik Terminal dan permintaan mengekalkan polisi sijil sedia ada.

Jika mana-mana semakan gagal, sambungan ditolak dan dicuba semula di latar belakang dengan jarak masa yang meningkat; pelayan dikongsi kekal tidak boleh diakses melalui relay sehingga semakan lulus atau anda mematikan mod ketat semula.

## Kegagalan biasa

- `Pengesahan TLS ketat memerlukan PEM CA akar` — togol diaktifkan tetapi medan CA kosong. Tampal PEM dahulu;
- `SAN tidak sepadan` — anda memanggil relay menggunakan nama atau IP yang tidak ada dalam sijilnya. Tambahkan nama itu pada halaman `Sijil & TLS` milik relay (lihat halaman Pengurusan Domain & SAN) dan keluarkan semula;
- `Sijil tamat tempoh atau belum sah` — jam relay tidak tepat, atau pembaharuan sijil daun gagal. Semak status sijil pada relay;
- `Tidak ditandatangani oleh CA yang dikonfigurasikan` — CA dalam medan milik relay lain, atau sambungan sedang disekat. Bandingkan cap jari sebelum meneruskan.
