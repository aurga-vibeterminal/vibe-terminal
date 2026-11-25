Penjelajah fail mengendalikan sistem fail pihak pelayan. Bar alat di bahagian atas panel kiri bertukar antara Recent / Explorer / Git / Debug / Search; Explorer ialah pelayar failnya.

## Butang Home: lompat ke cwd terminal

Klik butang `Home` (rumah) dalam bar alat:

- Bertukar ke paparan `Explorer`;
- Lompat ke **direktori kerja terminal aktif** (cwd-nya);
- Menematkan direktori itu sebagai pokok dan mengosongkan kotak carian — anda sentiasa bermula daripada "di mana terminal berada, itulah yang anda lihat".

![Penjelajah fail dalam mod biasa — Home, serampang roti dan senarai rata](images/files-explorer.jpg)

## Mod pokok dan carian

- **Tematkan sebagai pokok**: butang pin di hujung kanan serampang roti menogol antara senarai rata dan mod pokok. Dalam mod pokok folder mengembang menjadi pokok direktori boleh lipat (lompatan Home menematkan secara automatik);
- **Carian**: kotak carian di atas menapis senarai semasa mengikut nama sambil menaip; `Search in Folder` daripada menu konteks Recent menjalankan carian rekursif ke atas keseluruhan direktori.

![Penjelajah fail dalam mod pokok — akar tersemat mengembang menjadi pokok direktori boleh lipat](images/files-explorer-tree.jpg)

## Operasi asas

Klik kanan fail atau folder:

- **Cipta**: `New Folder` / `New File`;
- **Sunting**: `Rename`; `Copy` / `Cut` / `Paste` (berfungsi merentasi direktori);
- **Padam**: `Delete`, dengan dialog pengesahan;
- **Kaitan terminal**: `Open in Terminal` membuka tab terminal baharu pada direktori itu; `cd to Directory` membuat cd terminal semasa ke sana;
- **Kaitan OS**: `Reveal in File Explorer` menunjukkan direktori tempatan dalam pengurus fail sistem;
- **Laluan**: `Copy Full Path` / `Copy Relative Path`;
- **Terbitkan**: `Host with HTTP server` menerbitkan direktori sebagai perkhidmatan fail HTTP (lihat Terbitkan dengan Pelayan HTTP); `Open in App` membukanya dengan aplikasi ejen yang dikesan pada pelayan.

Setiap operasi bertindak pada sistem fail pihak pelayan; sesi jauh berkelakuan tepat seperti tempatan.
