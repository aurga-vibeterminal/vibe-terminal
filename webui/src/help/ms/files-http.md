`Host with HTTP server` menerbitkan sebarang direktori pada pelayan sebagai perkhidmatan fail HTTP dengan satu klik: peranti lain pada rangkaian yang sama (atau melalui relay) — telefon, komputer lain, rig ujian — boleh melayari dan memuat turun direktori itu daripada pelayar. Ideal untuk berkongsi artifak binaan dan bundel log, atau menyerahkan fail kepada peranti tanpa klien terpasang.

## Cara menggunakannya

Tiga pintu masuk, semuanya pada direktori: menu klik kanan senarai fail, menu klik kanan entri Recent, dan menu klik kanan serampang roti → `Host with HTTP server`.

Mengkliknya:

1. Pelayan memulakan perkhidmatan HTTP statik berakar pada direktori itu, dengan port diambil daripada julat port Static HTTP;
2. **Pelayan tempatan**: URL `http://127.0.0.1:port/` disalin terus ke papan klip, dengan mesej status "HTTP server: … (disalin ke papan klip)";
3. **Pelayan jauh** (bersambung melalui relay): klien secara automatik meminta pemetaan proksi songsang dan menyalin URL perkongsian yang boleh dicapai dari luar — sekatan LAN bukan penghalang.

Buka URL itu dalam pelayar sebarang peranti dan direktori boleh dilayari dan dimuat turun seperti senarai web biasa.

## Mengurus perkhidmatan berjalan

Setiap perkhidmatan HTTP terus berjalan sehingga dihentikan:

- Buka bahagian `Static HTTP` Web UI; senarai `Services` menunjukkan setiap perkhidmatan berjalan dan akar dokumennya;
- setiap entri mempunyai butang henti; julat port dikonfigurasikan pada halaman yang sama.

> Perkhidmatan ialah senarai fail baca-sahaja — tiada titik akhir tulisan; kandungan direktori kelihatan secara langsung ketika berubah, sesuai untuk log dan artifak yang dihasilkan berterusan.
