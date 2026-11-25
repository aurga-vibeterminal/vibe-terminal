Vibe Terminal ialah hab terminal yang diurus dari jauh untuk pembangunan terbenam dan pelayan. Satu **pelayan** berjalan pada setiap mesin yang anda kerjakan — mesin pembangunan, pelayan binaan, papan terbenam — dan **klien** bersambung kepadanya daripada aplikasi desktop atau baris perintah. Panduan ini meliputi kedua-dua belah pihak: Web UI yang mengkonfigurasi pelayan, dan klien yang bekerja dengannya.

## Bagaimana ia berpadu

- Semuanya berjalan pada **pelayan** — sesi terminal berada di situ, jadi menutup klien, kehilangan rangkaian atau menyusun semula tetingkap tidak pernah mengganggu perintah yang sedang berjalan; sambung semula dan teruskan dari tempat anda berhenti;
- Klien mendail satu sambungan WebSocket tersulit (`wss://host:port`; `wss://localhost:5153` untuk pelayan tempatan) dan mengesahkan diri dengan **kunci rahsia** pelayan; pemasangan menyerahkan kunci itu melalui kod QR atau rentetan perkongsian tersulit (lihat Menyambungkan Klien);
- **Web UI** ialah halaman konfigurasi pelayan itu sendiri — profil, addon, penyedia LLM, relay, pemajuan port dan banyak lagi. Ia bukan terminal: kerja harian berlaku dalam klien desktop atau `vtermc`.

## Komponen

- **Pelayan** — berjalan pada mesin sasaran; menempatkan sesi terminal serta menyediakan fail, Git, desktop jauh, ciri suara dan AI kepada klien yang bersambung;
- **Klien desktop** — ruang kerja natif: terminal dengan anak tetingkap terbahagi, kumpulan tab dan tetingkap terpisah, penjelajah dan penyunting fail, alat Git, input suara, sejarah perbualan ejen kod, desktop jauh dan penyahpepijatan jauh;
- **CLI `vtermc`** — lancarkan dan sambung ke sesi daripada mana-mana terminal biasa, melalui SSH atau pada mesin headless; berkongsi senarai pelayannya dengan klien desktop (lihat Klien Baris Perintah);
- **Web UI** — tetapan berasaskan pelayar untuk pelayan itu sendiri, tersusun kepada bahagian dalam bar sisi kiri (lihat Menggunakan Web UI).

## Ikon dulang

Pada mesin dengan sesi desktop, pelayan memaparkan ikon dulang semasa ia berjalan:

![Menu ikon dulang — Open mengemuka klien yang sedang berjalan atau melancarkannya, Settings membuka Web UI, dan item suara memulakan atau menghentikan pelayan suara](images/start-tray-menu.jpg)

- **Open** — mengemuka klien desktop yang sedang berjalan, atau melancarkannya apabila tiada yang berjalan (klik dua kali ikon melakukan perkara yang sama);
- **Settings** — membuka Web UI pelayan dalam pelayar (lihat Menggunakan Web UI);
- **Start/Stop Vibe Voice** — memulakan atau menghentikan pelayan suara tanpa membuka Web UI, mengikut butang Start/Stop dalam bahagian `Suara`; ia muncul apabila pengecaman suara diaktifkan, dan labelnya mengikut keadaannya (cth. `Stop Vibe Voice (model on device)` semasa model dimuatkan);
- **Exit** — menutup pelayan.

Semasa direktori dihoskan melalui HTTP, entri `HTTP Servers` menawarkan penghentian satu klik untuk setiap perkongsian (lihat Terbitkan dengan Pelayan HTTP).

## Peta ciri

Selebihnya panduan ini mengikut bab-bab dalam bar sisi:

- **Terminal** — mencipta, memecahkan dan mengurus terminal; profil, WSL dan Siri UART; perintah pantas dan pemboleh ubah persekitaran; memandu terminal daripada ejen luaran dengan MCP; menyemak perbualan ejen kod; pengesahan automatik gesaan interaktif;
- **Fail** — projek terkini, penjelajah fail pihak pelayan, muat naik & muat turun, dan menerbitkan direktori melalui HTTP (lihat Penjelajah Fail, Muat Naik & Muat Turun dan Terbitkan dengan Pelayan HTTP);
- **Git** — staging, commit berbantu AI, worktree, rebase interaktif, penyelesaian konflik dan push;
- **Desktop Jauh & Simulator** — menonton dan mengawal skrin pelayan, peranti dan emulator Android, serta simulator iOS;
- **Input Suara** — pengecaman pertuturan pihak pelayan dengan dikte push-to-talk dan polesan LLM;
- **Relay** — mencapai mesin di sebalik NAT atau IP dinamik melalui satu titik masuk awam;
- **Akses Jauh** — domain DDNS dan pemajuan port TLS untuk perkhidmatan yang berjalan pada pelayan;
- **Ciri AI** — penyedia LLM sebagai addon, ditugaskan mengikut ciri, serta integrasi MCP (lihat LLM dan MCP).

## Di mana untuk bermula

1. Buka bahagian `Server` Web UI, tetapkan `Secret Key`, dan pasangkan klien — Menyambungkan Klien membimbing melalui persediaan kod QR dan manual;
2. Lungsuri bahagian Web UI untuk mengkonfigurasi profil, addon dan ciri — lihat Menggunakan Web UI;
3. Buka terminal pada pelayan dan selesaikan bab-bab di atas mengikut keperluan.
