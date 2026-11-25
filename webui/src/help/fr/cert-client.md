Par défaut, un client chiffre chaque connexion mais ne vérifie pas qui se trouve à l’autre bout — la même posture de confiance à la première utilisation que l’acceptation d’un certificat auto-signé dans un navigateur. La **vérification stricte** change cela : le client vérifie le certificat du serveur auprès d’une AC racine que vous fournissez et refuse de se connecter tant que la chaîne, la validité et les SAN ne sont pas tous conformes. Cela rend les attaques de l’homme du milieu sur la connexion pratiquement impossibles, au prix d’une étape manuelle : remettre au client l’AC racine du serveur.

## Où le configurer

La vérification stricte se configure par serveur dans le panneau d’édition de serveur du client, à côté de l’adresse et de la clé secrète :

1. `Certificat de l’AC racine (PEM)` — un champ multiligne pour le certificat public de l’AC racine du serveur ;
2. `Vérification stricte de l’AC (chaîne, validité et SAN)` — sélectionnable uniquement une fois une AC saisie.

![Le panneau d’édition de serveur dans le client de bureau : champ multiligne Certificat de l’AC racine (PEM) avec la case à cocher de vérification stricte en dessous](images/client-ca-fields.jpg)

## Amener l’AC racine sur le client

Deux moyens, depuis la même source (la page `Certificats et TLS` du serveur — voir la page Domaines et gestion des SAN) :

- **Copier-coller** — appuyez sur `Copier l’AC racine` dans l’interface Web et collez le PEM dans le champ `Certificat de l’AC racine (PEM)` du client ;
- **Importer une chaîne de partage** — les chaînes de partage produites par le client intègrent l’AC racine en même temps que l’adresse et la clé. Importer un tel partage préremplit le champ de l’AC ; le destinataire n’a plus qu’à cocher la case de vérification stricte.

L’AC est la même pour tous les clients et ne change jamais au fil des renouvellements de certificat — une seule saisie par serveur suffit.

## Ce que vérifie le mode strict

Lorsque la case est cochée, chaque connexion à ce serveur doit présenter un certificat qui :

1. Se chaîne jusqu’à l’AC racine configurée — un certificat signé par quoi que ce soit d’autre est rejeté ;
2. Se trouve dans sa période de validité — les certificats expirés ou pas encore valides sont rejetés ;
3. Porte un SAN correspondant à l’adresse à laquelle vous vous connectez — un certificat émis pour d’autres noms est rejeté.

Si l’un des contrôles échoue, la connexion est refusée avec une erreur indiquant lequel a échoué. Sans la case cochée (ou sans AC), les connexions restent chiffrées mais ces contrôles sont ignorés.

## Vérifier l’empreinte

Avant de faire confiance à l’AC, confirmez que vous avez la bonne : la page `Certificats et TLS` affiche l’empreinte SHA-256 de l’AC racine. Comparez-la au PEM que vous vous apprêtez à coller (ou au PEM intégré à un partage reçu). Si elles diffèrent, n’activez pas la vérification stricte — obtenez plutôt l’AC via un canal de confiance.

## Échecs courants

- `Non-concordance de SAN` — vous vous connectez par un nom ou une IP qui ne figure pas dans le certificat. Ajoutez-le sur le serveur (voir la page Domaines et gestion des SAN) et réémettez ;
- `Certificat expiré ou pas encore valide` — l’horloge du serveur est déréglée ou le renouvellement du certificat feuille a échoué. Vérifiez l’état du certificat dans l’interface Web ;
- `Non signé par l’AC configurée` — l’AC du champ appartient à un autre serveur, ou la connexion est interceptée. Comparez les empreintes avant de continuer.
