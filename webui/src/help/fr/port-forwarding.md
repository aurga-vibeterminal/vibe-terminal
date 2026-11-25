La redirection de ports publie un service HTTP qui tourne sur la machine serveur — un serveur de développement, un tableau de bord d’appareil, tout ce qui écoute sur une simple adresse `http://localhost:port` — sur son propre port TLS, de sorte que tout appareil capable de joindre le serveur l’ouvre dans un navigateur via `https://<adresse-du-serveur>:<port>/`.

## Comment ça marche

- Une redirection associe un **port cible** — le service HTTP de cette machine — à un **port redirigé** tiré de la plage de ports configurée (par défaut `62000`–`62999`) ;
- le port redirigé parle TLS : tout le trafic est chiffré avec le **certificat auto-signé** du serveur — rien ne circule en clair, même sur un réseau non fiable ;
- tout appareil capable de joindre le serveur — sur le réseau local, ou depuis Internet une fois la plage de ports redirigée sur le routeur — ouvre `https://<adresse-du-serveur>:<portRedirigé>/` et atterrit sur le service local.

Les redirections sont créées à la demande : le partage de fichiers `Host with HTTP server` sur un serveur distant (relais) en demande automatiquement une et vous remet l’URL prête à ouvrir. Aucune configuration manuelle n’est nécessaire.

## Gérer les redirections

![Interface Web — la section Port Forwarding : configuration de la plage de ports et liste des redirections actives](images/port-forwarding-webui.jpg)

1. Ouvrez la section `Port Forwarding` de l’interface Web du serveur.
2. `Port Range Configuration` borne le pool dans lequel les ports redirigés sont tirés ; les changements s’enregistrent automatiquement. Réduisez-la quand votre pare-feu ou routeur ne redirige que quelques ports.
3. La liste en direct montre chaque redirection sous la forme `:portRedirigé → :portCible` avec son nombre de connexions actives.
4. Le bouton d’arrêt d’une redirection supprime immédiatement l’écouteur et déconnecte ses connexions actives ; le partage suivant tire un port neuf. `Refresh` relit la liste.

## L’avertissement de certificat

Les redirections sont chiffrées avec le certificat auto-signé du serveur lui-même — le même qui protège l’interface Web et les connexions clients. Les navigateurs ne peuvent pas le vérifier auprès d’une autorité de certification, donc la première visite d’une URL redirigée affiche un avertissement de confidentialité (*Votre connexion n’est pas privée*). C’est attendu et sans risque de poursuivre : le chiffrement lui-même n’est pas compromis, seule l’origine du certificat ne peut pas être vérifiée automatiquement. Pour faire taire l’avertissement définitivement, importez le certificat du serveur dans le magasin de certificats de confiance de votre appareil.

![Navigateur — l’avertissement de certificat auto-signé non fiable ; poursuivez via Paramètres avancés → Continuer pour ouvrir la page](images/port-forwarding-cert-warning.jpg)

## Chrome sur macOS affiche `ERR_ADDRESS_UNREACHABLE`

Sous macOS 15 (Sequoia) et ultérieur, les applications tierces ont besoin d’une autorisation explicite pour joindre les appareils du réseau local — toute adresse `192.168.x.x`, `10.x.x.x` ou similaire. Safari en est exempté en tant qu’application système, donc une URL redirigée peut s’ouvrir normalement dans Safari tandis que Chrome signale `ERR_ADDRESS_UNREACHABLE` avant même le début du TLS. Ce n’est ni un problème de certificat ni un problème de la redirection elle-même.

Pour autoriser Chrome :

1. Ouvrez `Réglages Système` → `Confidentialité et sécurité` → `Réseau local`.
2. Trouvez `Google Chrome` dans la liste et activez-le.
3. Quittez complètement Chrome (`Cmd+Q`, pas seulement en fermant les fenêtres) et relancez-le.

Si l’interrupteur est déjà activé, désactivez-le puis réactivez-le — il peut rester bloqué après une mise à jour de Chrome ou de macOS. Si l’erreur persiste, vérifiez dans Chrome la présence d’un VPN ou d’une extension proxy qui ne routerait pas les plages d’adresses privées.

## Contrôle d'accès

Ce qui protège un port redirigé dépend de ce qui se trouve derrière :

- **Les partages intégrés exigent un jeton.** Les liens de partage générés par le client — un partage `Host with HTTP server` (`?share_token=…`) et la page Settings (`?ticket=…`) — contiennent un jeton d'accès. Une connexion sans jeton valide ni session valide est interrompue avant la moindre réponse. Lors de la première visite, le navigateur l'échange contre un cookie de session sécurisé propre à ce service et le jeton disparaît de l'URL — il ne reste ni dans la barre d'adresse ni dans l'historique.

- **Les jetons sont à usage unique et éphémères.** Un ticket Settings fonctionne une seule fois et expire au bout d'une minute ; dès qu'un partage s'arrête ou redémarre, les liens et sessions précédents cessent de fonctionner. Considérez le lien complet comme un identifiant — quiconque le détient peut accéder au service — et ne le partagez qu'avec des personnes de confiance.

- **Vos propres services gardent leur propre authentification.** Une redirection vers un autre service local chiffre le trafic avec le certificat auto-signé du serveur mais n'ajoute aucune authentification : tout ce que le service sert — pages comme API — est accessible à quiconque peut joindre le port. Ajoutez une authentification aux services sensibles avant de les exposer.
