Ada dua cara menggunakan WSL di Windows: cara ringan menambah shell WSL sebagai profil terminal, cara penuh memasang vterm-server dalam WSL. Yang pertama hanya memberi shell; yang kedua membuka pengurusan fail, panel Git, MCP dan segalanya.

## Cara ringan: profil terminal WSL

Di Windows, **profil terminal WSL dimuatkan secara automatik**: semasa permulaan pelayan mengesan `wsl.exe` dan mengenumerasi pengedaran terpasang, menambah satu entri profil setiap distro ke senarai (cth. `Ubuntu (WSL)`; entri generik `WSL` muncul apabila tiada distro tersenarai).

Untuk menggunakannya, cuma pilih entri WSL dalam menu profil bar tajuk klien — distro dilancarkan dan terus menjatuhkan anda ke terminalnya, seperti membuka yang tempatan.

Had: sesi sebegitu berjalan di pelayan pihak Windows — **penjelajah fail, panel Git, pemboleh ubah persekitaran dan seumpamanya pada klien tidak berfungsi** (ia beroperasi dengan semantik sistem fail Windows dan tidak dapat melihat sistem fail Linux dalam WSL).

![Menu profil terminal — distro WSL tersenarai secara automatik](images/terminal-wsl-profile.jpg)

## Cara penuh: pasang vterm-server dalam WSL

Anggap WSL sebagai mesin Linux tersendiri dan pasang vterm-server headless. Laman web menawarkan skrip satu baris — tampalkannya ke terminal WSL:

```bash
curl -fsSL https://vterminal.dev/install.sh | bash
```

Skrip secara automatik:

1. Mengesan persekitaran WSL/headless dan memuat turun vterm-server **headless** ke `~/.local/vterm`;
2. Memasang **perkhidmatan pengguna systemd** dan terus memulakannya — ia berjalan pada setiap daftar masuk (permulaan WSL termasuk), tanpa permulaan manual;
3. Menambah `~/.local/vterm` ke PATH anda.

> Apabila systemd dilumpuhkan dalam WSL, skrip menunjukkan penambahan `[boot]` `systemd=true` ke `/etc/wsl.conf`, menjalankan `wsl --shutdown` di pihak Windows, dan menjalankan semula pemasang.

## Membuka halaman konfigurasi

Dalam WSL:

```bash
vterm-server --open
```

Perintah mencetak alamat Web UI (`http://localhost:<config-port>/`) dan cuba melancarkan pelayar (dalam WSL melalui `wslview`, pelayar lalai Windows). Terima kasih kepada pemajuan localhost WSL2, **membuka alamat yang sama dalam pelayar Windows** berfungsi sama baiknya. Semasa pelayan berjalan, URL yang dicetak membawa token log masuk sekali guna (`?ticket=...`, dikeluarkan melalui soket tempatan) maka halaman terbuka dalam keadaan sudah log masuk.

## (Pilihan) Kongsi pelayan WSL ke relay hos

Jika mesin Windows turut menjalankan vterm-server (sebagai relay), pelayan WSL boleh menempel di bawahnya supaya klien melihat kedua-dua "mesin" melalui satu sambungan tempatan:

1. Pada Web UI pelayan Windows, klik kod QR untuk menyalin rentetan perkongsian;
2. Buka Web UI pelayan WSL (alamat daripada bahagian sebelumnya), klik butang tampal yang muncul di sebelah medan `Relay Address` untuk mengisi alamat dan kunci secara automatik, kemudian `Apply`;
3. Selagi vterm-server Windows berjalan, klien melihat terminal, fail dan Git pelayan WSL di bawah relay tempatan.

> Di bawah rangkaian NAT lalai WSL2, WSL mencapai hos Windows melalui alamat hos dari pihak WSL (gerbang lalai — nameserver dalam `cat /etc/resolv.conf`); dengan rangkaian cermin `localhost` berfungsi. Lihat bab Relay untuk butiran relay.
