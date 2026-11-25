Par défaut, un serveur partagé chiffre sa connexion en amont vers le relais mais ne vérifie pas qui se trouve à l’autre bout — la même posture de confiance à la première utilisation que l’acceptation d’un certificat auto-signé dans un navigateur. Le **TLS strict du relais** change cela : le serveur partagé vérifie le certificat du relais auprès d’une AC racine que vous fournissez et refuse de se connecter tant que la chaîne, la validité et le nom d’hôte ne sont pas tous conformes. Cela rend les attaques de l’homme du milieu sur le lien relais pratiquement impossibles, au prix d’une étape manuelle : remettre au serveur partagé l’AC racine du relais. Il applique la même idée que la page précédente, Vérification stricte côté clients, au lien en amont du serveur partagé.

## Où le configurer

La vérification stricte se configure sur le serveur partagé — la machine dont l’`Relay Address` pointe vers le relais. Dans son interface Web, ouvrez la section `Server` : sous le `Relay Secret Key` se trouve une ligne `TLS strict du relais` affichant l’état `On` / `Off` courant. Dépliez-la pour découvrir :

1. `AC racine du relais amont` — un champ multiligne pour le certificat public de l’AC racine du relais ;
2. Un commutateur `Off` / `On` — `On` n’est sélectionnable qu’une fois le PEM lu comme certificat valide.

![Interface Web — le bloc TLS strict du relais déplié dans la section Server : champ AC racine du relais amont avec le commutateur Off/On et le bouton Apply](images/cert-relay-strict-tls.jpg)

Les réglages du relais ne sont jamais sauvegardés en cours de modification : vérifiez les valeurs puis appuyez sur `Apply`. L’enregistrement reconnecte immédiatement le relais en amont avec les nouveaux réglages.

## Amener l’AC racine sur le serveur partagé

Deux moyens, depuis la même source (l’interface Web du relais lui-même) :

- **Coller une chaîne de partage** — cliquez le `QR code` dans l’interface Web du serveur relais. La chaîne de partage intègre l’AC racine du relais en même temps que l’adresse et la clé ; la coller dans le champ `Relay Address` du serveur partagé remplit aussi le champ de l’AC (voir le Guide d’installation sous Relais). Le mode strict reste désactivé jusqu’à ce que vous vérifiez l’AC et l’activiez vous-même ;
- **Copier-coller** — appuyez sur `Copier l’AC racine` sur la page `Certificats et TLS` du relais et collez le PEM dans le champ `AC racine du relais amont`.

L’AC est la même pour tous les serveurs partagés et ne change jamais au fil des renouvellements de certificat — une seule saisie par relais suffit.

Avant de faire confiance à l’AC, confirmez que vous avez la bonne : la page `Certificats et TLS` du relais affiche l’empreinte SHA-256 de l’AC racine. Comparez-la au PEM que vous vous apprêtez à coller. Si elles diffèrent, n’activez pas la vérification stricte — obtenez plutôt l’AC via un canal de confiance.

## Ce que vérifie le mode strict

Avec le commutateur sur `On`, la connexion `RelayPrimary` en amont doit présenter un certificat qui :

1. Se chaîne jusqu’à l’AC racine configurée — un certificat signé par quoi que ce soit d’autre, y compris les racines système publiques, est rejeté ;
2. Se trouve dans sa période de validité — les certificats expirés ou pas encore valides sont rejetés ;
3. Porte un SAN correspondant à l’hôte de l’`Relay Address` à laquelle vous vous connectez — un certificat émis pour d’autres noms est rejeté.

Une limite à garder en tête : seule la connexion en amont vers le relais est vérifiée. Les connexions de retour Terminal et requêtes conservent la politique de certificats existante.

Si l’un des contrôles échoue, la connexion est refusée puis retentée en arrière-plan à intervalles croissants ; le serveur partagé reste inaccessible via le relais jusqu’à ce que le contrôle passe ou que vous désactiviez le mode strict.

## Échecs courants

- `La vérification TLS stricte exige une AC racine PEM` — le commutateur est activé mais le champ de l’AC est vide. Collez d’abord le PEM ;
- `Non-concordance de SAN` — vous appelez le relais par un nom ou une IP qui ne figure pas dans son certificat. Ajoutez-le sur la page `Certificats et TLS` du relais (voir la page Domaines et gestion des SAN) et réémettez ;
- `Certificat expiré ou pas encore valide` — l’horloge du relais est déréglée ou le renouvellement du certificat feuille a échoué. Vérifiez l’état du certificat sur le relais ;
- `Non signé par l’AC configurée` — l’AC du champ appartient à un autre relais, ou la connexion est interceptée. Comparez les empreintes avant de continuer.
