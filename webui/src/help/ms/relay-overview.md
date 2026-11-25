Relay menyelesaikan satu masalah: kebanyakan mesin berada di belakang NAT atau IP dinamik, jadi klien tidak boleh mendail setiap satunya secara langsung. Biarkan satu mesin yang boleh dicapai awam bertindak sebagai titik masuk relay; setiap mesin lain mendail keluar kepadanya. Klien kemudiannya mengekalkan satu sambungan ke relay dan mencapai terminal setiap pelayan dikongsi melaluinya.

## Seni bina

![Seni bina relay — setiap klien mengekalkan satu sambungan ke relay; pelayan dikongsi mendail relay ke hulu](diagram/relay)

- **Pelayan relay** — titik masuk awam yang disambung oleh klien dan pelayan dikongsi. Biarkan `Relay Address`-nya kosong.
- **Pelayan dikongsi** — mesin yang `Relay Address`-nya menunjuk ke relay. Ia mendail ke hulu dan berkongsi terminalnya melaluinya. Jalur berwarna pada setiap nod dikongsi dalam rajah mencerminkan jalur yang dilukis klien desktop pada tab pelayan itu — satu warna setiap pelayan.
- Sambungan sentiasa berasal dari dalam: pelayan dikongsi mendail relay, jadi tiada pemajuan port atau IP awam diperlukan pada mereka.

## Apa yang dilihat klien

Sekali bersambung ke relay, setiap pelayan dikongsi berkelakuan seperti mesin tempatan:

- Tab terminal membawa jalur berwarna dan ikon OS jauh, mengenal pasti mesin sekali imbas (lihat Panduan Persediaan);
- menu profil mengumpulkan terminal setiap pelayan — buka terminal pada sebarang mesin secara langsung;
- kumpulan tab boleh direnggangkan ke tetingkapnya sendiri, menauliahkan satu tetingkap untuk satu mesin.

## Model keselamatan

Klien dan pelayan dikongsi dipasangkan dengan kunci rahsia yang sama; sambungan ditunnelkan melalui relay. Kunci bergerak dalam kod QR pemasangan / rentetan perkongsian, tidak pernah dalam teks jelas.

Arahan langkah demi langkah ada di halaman seterusnya, Panduan Persediaan.
