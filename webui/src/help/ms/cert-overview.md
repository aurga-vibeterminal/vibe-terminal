Pelayan melindungi semua titik akhir TLS yang didedahkannya — antara muka web, sambungan klien dan port yang diteruskan — dengan sijil yang dikeluarkan oleh **CA akar peribadinya** sendiri. Halaman `Sijil & TLS` dalam antara muka web ialah tempat anda mengurus CA itu, mengawal nama-nama yang sah untuk sijil tersebut, dan menyerahkan CA kepada klien yang mahu mengesahkan pelayan secara ketat.

## Cara ia berfungsi

- Pelayan menjana CA akar peribadi sekali sahaja dan menyimpannya sepanjang hayat pemasangan. Semuanya ditandatangani olehnya;
- CA menandatangani **sijil daun** untuk pelayan dengan tempoh sah 90 hari, meliputi nama hos dan alamat IP yang benar-benar digunakan klien untuk menghubunginya;
- Apabila nama hos, antara muka rangkaian atau entri SAN mesin berubah, sijil daun dikeluarkan semula secara automatik — CA akar tidak pernah berubah, jadi klien yang sudah mempercayainya terus mempercayai pelayan;
- Sijil daun baharu ditukar melalui penyelesai sijil kongsi dan dimuatkan semula secara terus (hot-reload): sambungan sedia ada tidak pernah diputus, dan jabat tangan baharu serta-merta menggunakan sijil terkini. Jika pembaharuan gagal, sijil terdahulu kekal aktif.

![Halaman Sijil & TLS: di bahagian atas kad CA Akar Peribadi, di tengah pengurusan SAN, di bawah butiran sijil daun pelayan](images/certificates-page.jpg)

## Apa yang terdapat pada halaman ini

1. `CA Akar Peribadi` — muat turun atau salin sijil awam CA untuk diedarkan kepada klien (dan pelayar web mereka). Muat turun tidak pernah mengandungi kunci peribadi;
2. `Domain & Alamat IP (SAN)` — senarai nama yang sah untuk sijil daun. Diterangkan pada halaman seterusnya, Pengurusan Domain & SAN;
3. `Sijil Daun Pelayan` — sijil yang sedang aktif: status, subjek, cap jari SHA-256, tempoh sah dan entri SAN.

## Sijil TLS tersuai

Jika sijil TLS tersuai dikonfigurasikan untuk pelayan, sijil itu kekal berkuatkuasa: sijil terurus pada halaman ini terus dijana dan dikemas kini, tetapi tidak menggantikan identiti tersuai yang dikonfigurasikan. Selagi identiti tersuai aktif, halaman menunjukkan notis.

## Mengapa mempercayai CA akar

Klien yang hanya memerlukan penyulitan berfungsi tanpa sebarang persediaan. Klien yang mahu *mengesahkan* pelayan — mod ketat yang diterangkan pada halaman terakhir, Pengesahan Ketat pada Klien — memerlukan sijil awam CA akar, dan ia mesti dihantar melalui saluran yang anda percayai:

> Muat turun CA akar hanya dari antara muka web tempatan pelayan ini atau melalui tunnel yang dipercayai, dan sahkan cap jari SHA-256 (ditunjukkan pada halaman ini) sebelum mendayakan pengesahan ketat di mana-mana.

CA yang sama turut menandatangani sijil yang digunakan oleh penerusan port (lihat halaman Penerusan Port). Sebaik sahaja peranti mempercayai CA akar, URL yang diteruskan dibuka tanpa amaran pelayar web *sambungan anda tidak bersifat peribadi*.
