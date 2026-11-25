`Host with HTTP server` publie n’importe quel répertoire du serveur comme service de fichiers HTTP en un clic : les autres appareils du même réseau (ou via le relais) — un téléphone, un autre ordinateur, un banc de test — peuvent parcourir et télécharger ce répertoire depuis un navigateur. Idéal pour partager des artefacts de build et des lots de journaux, ou remettre des fichiers à des appareils sans client installé.

## Comment l’utiliser

Trois entrées, toutes sur un répertoire : le menu contextuel de la liste de fichiers, le menu contextuel d’une entrée Recent, et le menu contextuel du fil d’Ariane → `Host with HTTP server`.

Un clic dessus :

1. le serveur démarre un service HTTP statique enraciné dans ce répertoire, le port tiré de la plage de ports Static HTTP ;
2. **Serveur local** : l’URL `http://127.0.0.1:port/` est copiée directement dans le presse-papiers, avec un message d’état « HTTP server : … (copié dans le presse-papiers) » ;
3. **Serveur distant** (connecté via un relais) : le client demande automatiquement un mappage de proxy inverse et copie l’URL de partage atteignable de l’extérieur — les restrictions LAN ne sont pas un obstacle.

Ouvrez cette URL dans le navigateur de n’importe quel appareil et le répertoire se parcourt et se télécharge comme une liste web normale.

## Gérer les services en cours

Chaque service HTTP continue de tourner jusqu’à l’arrêt :

- ouvrez la section `Static HTTP` de l’interface Web ; la liste `Services` montre chaque service en cours et sa racine de documents ;
- chaque entrée a un bouton d’arrêt ; la plage de ports se configure sur la même page.

> Le service est une liste de fichiers en lecture seule — pas de points d’écriture ; le contenu du répertoire est visible en direct à mesure qu’il change, ce qui convient aux journaux et artefacts produits en continu.
