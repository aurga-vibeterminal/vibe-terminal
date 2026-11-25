Bestandsoverdrachten tussen jouw machine en de server zijn slepen & loslaten, en de richting is de betekenis: **naar binnen slepen = uploaden naar de server, naar buiten slepen = downloaden naar deze machine**. Overdrachten rijden mee op de versleutelde client-serververbinding; externe en gedeelde servers gedragen zich als lokale.

## Uploaden: naar binnen slepen

Sleep bestanden (of hele mappen) vanuit de bestandsbeheerder van je besturingssysteem op de bestandslijst en laat los:

- In boommodus uploadt **loslaten op een mapregel** naar die map; **loslaten op de wortel** (de invoeglijn tussen regels) uploadt naar de huidige wortelmap;
- Onderaan de lijst verschijnt een overdrachtsregel met `Uploading`, de bestandsnaam en een voortgangsbalk;
- Voltooiing toont `Uploaded n file(s)`; mislukkingen bevatten de reden (bijv. toegang geweigerd) en de lijst ververst om de nieuwe bestanden te onthullen.

## Downloaden: naar buiten slepen

Druk op een bestand in de lijst, sleep het uit het clientvenster en laat het vallen in de bestandsbeheerder van je besturingssysteem (of op het bureaublad):

- Het bestand streamt van de server naar waar je het liet vallen;
- Overdrachtsregels tonen voortgang op dezelfde manier; **een voltooide downloadregel is klikbaar** en opent de lokale downloadmap;
- Meervoudig selecteren en meerdere bestanden tegelijk slepen.

![Overdrachtsvoortgang — de upload-/downloadregels](images/files-transfer.jpg)

> Slepen & loslaten verplaatst losse bestanden tussen deze machine en één server; om gegevens tussen twee servers te verplaatsen, combineer Publiceren met HTTP-server of terminalopdrachten (scp/rsync).
