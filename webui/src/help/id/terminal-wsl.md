Ada dua cara memakai WSL di Windows: cara ringan menambahkan shell WSL sebagai profil terminal, cara lengkap memasang vterm-server di dalam WSL. Yang pertama hanya memberi shell; yang kedua membuka manajemen berkas, panel Git, MCP, dan segalanya.

## Cara ringan: profil terminal WSL

Di Windows, **profil terminal WSL dimuat otomatis**: saat mulai, server mendeteksi `wsl.exe` dan menghitung distribusi terpasang, menambahkan satu entri profil per distro ke daftar (mis. `Ubuntu (WSL)`; entri generik `WSL` muncul saat tidak ada distro terdaftar).

Untuk memakainya, cukup pilih entri WSL di menu profil bilah judul klien — distro diluncurkan dan langsung menjatuhkan Anda ke terminalnya, seperti membuka lokal.

Keterbatasan: sesi semacam itu berjalan di server sisi Windows — **penjelajah berkas, panel Git, variabel lingkungan, dan kawanannya di klien tidak berfungsi** (mereka beroperasi dengan semantik sistem berkas Windows dan tidak dapat melihat sistem berkas Linux di dalam WSL).

![Menu profil terminal — distro WSL yang terdaftar otomatis](images/terminal-wsl-profile.jpg)

## Cara lengkap: pasang vterm-server di dalam WSL

Perlakukan WSL sebagai mesin Linux tersendiri dan pasang vterm-server headless. Situs web menawarkan skrip satu baris — tempel ke terminal WSL:

```bash
curl -fsSL https://vterminal.dev/install.sh | bash
```

Skrip otomatis:

1. Mendeteksi lingkungan WSL/headless dan mengunduh vterm-server **headless** ke `~/.local/vterm`;
2. Memasang **layanan pengguna systemd** dan langsung menjalankannya — ia berjalan setiap login (termasuk mulai WSL), tanpa perlu start manual;
3. Menambahkan `~/.local/vterm` ke PATH Anda.

> Saat systemd di WSL nonaktif, skrip menunjuk penambahan `[boot]` `systemd=true` ke `/etc/wsl.conf`, menjalankan `wsl --shutdown` di sisi Windows, dan menjalankan ulang instalator.

## Membuka halaman konfigurasi

Di dalam WSL:

```bash
vterm-server --open
```

Perintah mencetak alamat Web UI (`http://localhost:<config-port>/`) dan mencoba membuka browser (di WSL lewat `wslview`, browser bawaan Windows). Berkat penerusan localhost WSL2, **membuka alamat yang sama di browser Windows** bekerja sama baiknya. Saat server berjalan, URL yang dicetak membawa token masuk sekali pakai (`?ticket=...`, diterbitkan lewat socket lokal) sehingga halaman terbuka dalam keadaan sudah masuk.

## (Opsional) Berbagi server WSL ke relay host

Jika mesin Windows juga menjalankan vterm-server (sebagai relay), server WSL dapat menempel di bawahnya agar klien melihat kedua "mesin" lewat satu koneksi lokal:

1. Di Web UI server Windows, klik QR code untuk menyalin string berbagi;
2. Buka Web UI server WSL (alamat dari bagian sebelumnya), klik tombol tempel yang muncul di samping bidang `Relay Address` untuk mengisi alamat dan kunci otomatis, lalu `Apply`;
3. Selama vterm-server Windows berjalan, klien melihat terminal, berkas, dan Git server WSL di bawah relay lokal.

> Di bawah jaringan NAT bawaan WSL2, WSL menjangkau host Windows lewat alamat host dari sisi WSL (gateway bawaan — nameserver di `cat /etc/resolv.conf`); dengan jaringan mirror `localhost` bekerja. Lihat bab Relay untuk detail relay.
