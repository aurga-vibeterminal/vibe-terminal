## Le navigateur avertit « Votre connexion n’est pas privée » — est-ce normal ?

Oui. Les ports redirigés, l’interface Web et les connexions clients partagent le même certificat auto-signé du serveur ; le chiffrement lui-même n’est pas compromis. Importez le certificat du serveur dans votre magasin de confiance pour faire disparaître l’avertissement définitivement.

## Vibe Terminal est-il gratuit ?

Oui — le client/serveur desktop et le serveur headless sont gratuits. Téléchargez et installez sur autant de machines que vous le souhaitez.

## Où vont mes données ?

Nulle part où vous ne les avez pas mises. Les sessions, les identifiants (enc-v2:) et la reconnaissance vocale restent sur vos machines ; les clés d’appairage ne voyagent jamais en clair.

## Puis-je exposer un service redirigé en toute sécurité ?

Oui. Les partages intégrés exigent un jeton : les liens générés par le client pour un partage `Host with HTTP server` ou la page Settings contiennent un jeton d’accès, et une connexion sans jeton valide ni session valide est interrompue avant la moindre réponse. Lors de la première visite, le jeton est échangé contre un cookie de session sécurisé propre à ce service et disparaît de l’URL — un ticket Settings fonctionne une seule fois et expire au bout d’une minute, et arrêter ou redémarrer un partage invalide ses liens et sessions précédents. Considérez le lien complet comme un identifiant et ne le remettez qu’à des personnes de confiance. Une redirection vers votre propre service ajoute le TLS du certificat du serveur mais aucune authentification — gardez une authentification devant les services sensibles et démontez une redirection instantanément depuis le panneau de redirection de ports une fois terminé.
