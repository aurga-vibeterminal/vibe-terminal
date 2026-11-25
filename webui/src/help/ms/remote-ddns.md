Jalur lebar rumah dan pejabat lazimnya datang dengan IP awam dinamik yang berubah tanpa amaran. DDNS mengekalkan domain tetap mengarah kepada IP semasa mesin — klien dan pelayan lain hanya perlu mengingati nama, bukan alamat.

## Cara ia berfungsi

![Seni bina DDNS — pengurus mengesan IP lokal berkala, addon mengemas kini rekod DNS melalui API penyedia, klien bersambung mengikut domain](diagram/ddns)

Keseluruhan saluran paip berjalan secara automatik di latar belakang pelayan:

1. **Pengurus DDNS** menyegarkan pada kitaran tetap (setiap 5 minit secara lalai, boleh laras melalui pemboleh ubah persekitaran `TERMINAL_SERVER_DDNS_POLL_INTERVAL_SECS`); menyimpan konfigurasi mencetuskan penyegaran segera, dan tiada apa berjalan selagi perkhidmatan dilumpuhkan;
2. **Pengesanan IP lokal** membaca antara muka rangkaian terpilih dahulu, jatuh kepada penemuan berasaskan laluan, dan menapis hasilnya mengikut tetapan `IP Family` (IPv4 / IPv6 / dwi);
3. **Addon DDNS** — skrip JS berkotak pasir — menerima konfigurasi nyahsulit dan IP semasa lalu memanggil API penyedia DNS melalui HTTP. Addon Cloudflare terbina dalam contohnya mencari zon mengikut domain dan menulis semula rekod A / AAAA kepada IP semasa, memadam rekod basi apabila satu keluarga alamat hilang;
4. **Klien** meleraikan domain kepada IP terkini dan bersambung mengikut nama.

## Konfigurasi

Buka bahagian `DDNS` Web UI:

1. Hidupkan `Enable`. Anda juga boleh membiarkan perkhidmatan mati sambil mengisi segalanya — perkhidmatan dilumpuhkan menyimpan konfigurasinya tetapi tidak pernah berjalan;
2. pilih penyedia di bawah `Provider`; `Cloudflare DDNS` terbina dalam;
3. isikan medan penyedia (contoh Cloudflare):
   - `Email` — e-mel akaun Cloudflare;
   - `API Token` — token dengan kebenaran Zone → DNS → Edit;
   - `Domain` — domain akar zon, cth. `example.com`;
   - `Subdomain` — nama rekod dalam zon, cth. `sub` untuk `sub.example.com`; biarkan kosong untuk akar zon;
4. dalam `Network Interface & IP`, pilih `Interface` dan `IP Family`. `Dual Stack` memerlukan antara muka mempunyai kedua-dua alamat IPv4 dan IPv6;
5. klik `Save`, kemudian `Sync Now` untuk mengesahkan. `Sync Status` menunjukkan IP terakhir yang didorong dan masa segerak, berserta mesej ralat addon apabila gagal.

## Keselamatan

Kredensial penyedia (token API) disulitkan sebelum disimpan (`enc-v2:`) dan hanya disyahsulitkan dalam memori selagi addon berjalan; addon dilaksanakan dalam masa jalanan JS berkotak pasir dengan HTTP keluar sebagai satu-satunya keupayaannya.

## Meluaskan penyedia

Penyedia ialah addon: manifes JSON (mengisytiharkan medan konfigurasi yang dijadikan Web UI borang) berserta skrip JS yang mendedahkan `run()`. `cloudflare-ddns` terbina dalam ialah pelaksanaan rujukan — penyedia tersuai boleh dimuat naik sebagai zip dalam bahagian `Addons`.

## Bekerja dengan ciri lain

- Sekali domain terleraikan, kod QR bahagian `Server` boleh menukar `Connection Type` kepada `DDNS Domain`, supaya klien mengimbas sambungan domain;
- pelayan lain boleh meletakkan `wss://your-domain:port` dalam `Relay Address` mereka untuk menjalankan rantaian relay melalui domain (lihat bab Relay).
