Pemajuan port menerbitkan perkhidmatan HTTP yang berjalan pada mesin pelayan — pelayan pembangunan, papan pemuka peranti, apa sahaja yang mendengar pada alamat `http://localhost:port` biasa — pada port TLS tersendiri, supaya mana-mana peranti yang dapat mencapai pelayan boleh membukanya dalam pelayar sebagai `https://<alamat-pelayan>:<port>/`.

## Cara ia berfungsi

- Pemajuan memetakan **port sasaran** — perkhidmatan HTTP pada mesin ini — kepada **port yang dimajukan** yang diperuntukkan daripada julat port yang dikonfigurasikan (lalai `62000`–`62999`);
- Port yang dimajukan bertutur TLS: semua trafik disulitkan dengan **sijil tandatangan sendiri** pelayan — tiada apa bergerak dalam teks biasa, walaupun melalui rangkaian yang tidak dipercayai;
- Mana-mana peranti yang dapat mencapai pelayan — pada LAN, atau dari internet setelah julat port dimajukan pada penghala — membuka `https://<alamat-pelayan>:<portDimajukan>/` dan tiba pada perkhidmatan tempatan.

Pemajuan dicipta atas permintaan: perkongsian fail `Host with HTTP server` pada pelayan jauh (relay) secara automatik meminta satu dan menyerahkan URL sedia dibuka kepada anda. Tiada persediaan manual diperlukan.

## Mengurus pemajuan

![Web UI — bahagian Port Forwarding: konfigurasi julat port dan senarai pemajuan aktif](images/port-forwarding-webui.jpg)

1. Buka bahagian `Port Forwarding` pada Web UI pelayan.
2. `Port Range Configuration` menghadkan kumpulan tempat port yang dimajukan diperuntukkan; perubahan disimpan secara automatik. Kecilkannya apabila tembok api atau penghala anda hanya memajukan beberapa port.
3. Senarai langsung menunjukkan setiap pemajuan sebagai `:portDimajukan → :portSasaran` berserta bilangan sambungan aktifnya.
4. Butang henti pemajuan menghentikan pendengar dengan serta-merta dan memutuskan sambungan aktifnya; perkongsian seterusnya memperuntukkan port baharu. `Refresh` membaca semula senarai itu.

## Amaran sijil

Pemajuan disulitkan dengan sijil tandatangan sendiri milik pelayan — sijil yang sama yang melindungi Web UI dan sambungan klien. Pelayar tidak dapat mengesahkannya terhadap pihak berkuasa persijilan, jadi lawatan pertama ke URL yang dimajukan memaparkan amaran privasi (*Sambungan anda bukan peribadi*). Ini memang dijangka dan selamat untuk diteruskan: penyulitan itu sendiri tidak dikompromi, hanya asal usul sijil tidak dapat disahkan secara mesin. Untuk menyenyapkan amaran itu secara kekal, import sijil pelayan ke stor kepercayaan peranti anda.

![Pelayar — amaran sijil tandatangan sendiri yang tidak dipercayai; teruskan melalui Advanced → Proceed untuk membuka halaman](images/port-forwarding-cert-warning.jpg)

## Chrome di macOS memaparkan `ERR_ADDRESS_UNREACHABLE`

Pada macOS 15 (Sequoia) dan lebih baharu, apl pihak ketiga memerlukan kebenaran eksplisit untuk mencapai peranti pada rangkaian tempatan — mana-mana alamat `192.168.x.x`, `10.x.x.x` atau seumpamanya. Safari dikecualikan sebagai apl sistem, jadi URL yang dimajukan boleh dibuka dengan baik dalam Safari manakala Chrome melaporkan `ERR_ADDRESS_UNREACHABLE` sebelum TLS pun bermula. Ini bukan masalah sijil dan bukan masalah dengan pemajuan itu sendiri.

Untuk memberikan akses kepada Chrome:

1. Buka `Tetapan Sistem` → `Privasi & Keselamatan` → `Rangkaian Tempatan`.
2. Cari `Google Chrome` dalam senarai dan **hidupkannya**.
3. Tutup Chrome sepenuhnya (`Cmd+Q`, bukan sekadar menutup tetingkap) dan lancarkannya semula.

Jika togol itu sudah hidup, matikannya dan hidupkan semula — ia boleh tersangkut selepas kemas kini Chrome atau macOS. Jika ralat berterusan, semak sambungan VPN atau proksi dalam Chrome yang tidak menghalakan julat alamat peribadi.

## Kawalan akses

Apa yang melindungi port yang dimajukan bergantung pada apa yang berada di sebaliknya:

- **Perkongsian terbina dalam memerlukan token.** Pautan perkongsian yang dijana oleh klien — perkongsian `Host with HTTP server` (`?share_token=…`) dan halaman Settings (`?ticket=…`) — menyertakan token akses. Sambungan tanpa token yang sah dan tanpa sesi yang sah diputuskan sebelum apa-apa pun disajikan. Pada lawatan pertama, pelayar menukarnya dengan kuki sesi selamat yang hanya berkesan untuk perkhidmatan tersebut, dan token hilang daripada URL — tidak kekal dalam bar alamat mahupun sejarah.

- **Token adalah sekali guna dan berumur pendek.** Ticket Settings hanya berfungsi sekali sahaja dan tamat tempoh dalam masa seminit; apabila perkhidmatan berhenti atau dimulakan semula, pautan dan sesi sebelumnya berhenti berfungsi. Anggap pautan penuh sebagai kredensial — sesiapa yang memilikinya boleh mengakses perkhidmatan — dan kongsi hanya dengan orang yang anda percayai.

- **Perkhidmatan anda sendiri membawa pengesahannya sendiri.** Pemajuan ke perkhidmatan tempatan lain menyulitkan trafik dengan sijil yang ditandatangani sendiri oleh pelayan tetapi tidak menambah pengesahan: apa sahaja yang disajikan perkhidmatan itu — halaman dan API — boleh dicapai sesiapa yang dapat menjangkau port tersebut. Letakkan pengesahan pada perkhidmatan sensitif sebelum mendedahkannya.
