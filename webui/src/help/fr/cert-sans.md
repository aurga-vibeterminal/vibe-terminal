Le certificat feuille n’est valide que pour les noms listés dans ses **Autres noms de sujet (SAN)**. Lorsqu’un client se connecte, le nom d’hôte ou l’IP de son URL doit correspondre exactement à l’une de ces entrées — ni les caractères génériques ni la correspondance par préfixe ne sont pris en charge. Cette page couvre la gestion de cette liste côté serveur ; la vérification côté client se trouve à la dernière page, Vérification stricte côté clients.

## Découverte automatique

Le serveur découvre en continu les noms par lesquels il est joignable — les noms d’hôte actuels et les adresses des interfaces réseau actives — et les liste sous `Découverts automatiquement`. Ces entrées suivent la machine et ne peuvent pas être modifiées à la main : ajoutez une interface réseau ou changez le nom d’hôte, et la liste suivra au prochain renouvellement.

## Domaines et adresses IP supplémentaires

La découverte ne sait pas tout : les noms DDNS, les entrées DNS à horizon scindé ou une adresse derrière une NAT doivent être listés à la main. Ajoutez-les sous `Domaines et adresses IP supplémentaires` :

1. Saisissez un nom d’hôte ou une adresse IP (par exemple `server.home.arpa` ou `10.0.0.8`) et appuyez sur `Ajouter` ;
2. Répétez pour chaque nom que les clients utiliseront. Supprimez une entrée avec le bouton corbeille ;
3. Vérifiez l’`Aperçu final des SAN` — la vue fusionnée des entrées découvertes et manuelles que portera le prochain certificat ;
4. Appuyez sur `Enregistrer et réémettre`. L’enregistrement signe un nouveau certificat feuille de 90 jours avec la nouvelle liste de SAN et le recharge à chaud — les connexions existantes restent établies et les nouvelles négociations TLS utilisent immédiatement le nouveau certificat.

## Réémission manuelle et état

Le panneau `Certificat feuille du serveur` affiche le certificat actif : un badge d’état, le sujet, l’empreinte SHA-256, la période de validité et les entrées SAN. Le badge d’état signifie :

- `Disponible` — un certificat feuille valide est chargé et utilisé ;
- `Échec de la mise à jour` — la dernière tentative de réémission a échoué ; le certificat précédent reste actif et l’erreur est affichée sur la page ;
- `Non généré` — ni l’AC ni le certificat feuille n’existent encore ; utilisez `Générer l’AC et le certificat` pour créer les deux.

`Réémettre le certificat feuille` signe immédiatement un nouveau certificat feuille sans attendre un déclencheur automatique — utile juste après un changement de nom d’hôte ou d’interfaces, ou pour prendre en compte des modifications de SAN faites hors de l’interface Web.

## Distribuer l’AC racine

Les clients qui utiliseront la vérification stricte ont besoin du certificat public de l’AC racine. Obtenez-le depuis la carte `AC racine privée` :

- `Télécharger l’AC racine` enregistre le fichier PEM ;
- `Copier l’AC racine` place le PEM dans le presse-papiers, prêt à être collé dans le champ AC d’un client.

L’AC ne change jamais au fil des renouvellements du certificat feuille ni des modifications de SAN — une seule distribution suffit. Avant d’activer la vérification stricte où que ce soit, comparez l’empreinte SHA-256 du certificat affichée sur cette page avec ce que le client a reçu (voir la dernière page, Vérification stricte côté clients).

> Le téléchargement et le presse-papiers ne contiennent que le certificat public de l’AC — jamais de clé privée. Ne la remettez qu’aux personnes que vous voulez pouvoir vérifier votre serveur.
