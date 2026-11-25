## Pelayar memaparkan amaran “Sambungan anda bukan peribadi” — adakah ini normal?

Normal. Port yang dimajukan, Web UI dan sambungan klien berkongsi sijil tandatangan sendiri pelayan yang sama; penyulitan itu sendiri tidak dikompromi. Import sijil pelayan ke stor kepercayaan peranti anda untuk menyenyapkan amaran itu secara kekal.

## Adakah Vibe Terminal percuma?

Ya — klien/pelayan desktop dan pelayan headless adalah percuma. Muat turun dan pasang pada seberapa banyak mesin yang anda suka.

## Ke mana data saya pergi?

Tidak ke mana-mana selain tempat anda meletakkannya. Sesi, kredensial (enc-v2:) dan pengecaman suara semuanya berada di mesin anda; kunci pemasangan tidak pernah bergerak dalam teks biasa.

## Bolehkah saya mendedahkan perkhidmatan yang dimajukan dengan selamat?

Boleh. Perkongsian terbina dalam memerlukan token: pautan yang dijana oleh klien untuk perkongsian `Host with HTTP server` dan halaman Settings menyertakan token akses, dan sambungan tanpa token yang sah dan tanpa sesi yang sah diputuskan sebelum apa-apa pun disajikan. Pada lawatan pertama, token ditukar dengan kuki sesi selamat yang hanya berkesan untuk perkhidmatan tersebut dan hilang daripada URL — ticket Settings hanya berfungsi sekali dan tamat tempoh dalam masa seminit, manakala menghentikan atau memulakan semula perkhidmatan mematikan pautan dan sesi sebelumnya. Anggap pautan penuh sebagai kredensial dan berikan hanya kepada orang yang anda percayai. Pemajuan ke perkhidmatan anda sendiri menambah TLS dengan sijil pelayan tetapi tanpa pengesahan — kekalkan pengesahan di hadapan perkhidmatan sensitif, dan bongkarkan pemajuan serta-merta dari panel pemajuan port apabila selesai.
