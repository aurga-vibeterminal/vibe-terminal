La gestion de fichiers vous laisse travailler avec les fichiers du serveur directement depuis le client : la barre d’activités de gauche a deux entrées — `Recent` (projets récents) et l’explorateur de fichiers — utilisées aux côtés des onglets de terminal. Tout opère sur le serveur de la session : la machine locale, ou une machine distante/partagée via un relais.

## Recent : projets récents

`Recent` liste les dossiers que vos agents de code ont ouverts le plus récemment sur le serveur courant, plus récents d’abord :

- **Cliquez** une entrée et l’explorateur de fichiers bascule vers ce répertoire ;
- **Clic droit** ouvre un menu d’actions rapides :
  - `Open in VS Code` — ouvrir le répertoire dans VS Code sur cette machine ;
  - `Reveal in File Explorer` — l’afficher dans le gestionnaire de fichiers du système ;
  - `Copy Full Path` / `Copy Relative Path` — chemin complet, ou chemin relatif au dépôt ;
  - `Search in Folder` — chercher des fichiers dans ce répertoire ;
  - `Open in Terminal` — ouvrir un nouvel onglet de terminal là ; `cd Here` — cd du terminal courant vers lui ;
  - `Host with HTTP server` — publier le répertoire comme service de fichiers HTTP (voir Publier avec un serveur HTTP) ;
  - `Open in App` — l’ouvrir avec une application d’agent détectée sur le serveur.

![Le panneau gauche — la liste des projets récents](images/files-overview.jpg)

## L’explorateur de fichiers

L’onglet `Explorer` parcourt et gère le système de fichiers côté serveur : navigation de répertoires, vues arbre et plate, recherche, création/renommage/suppression et autres opérations de fichiers, plus le téléversement et le téléchargement par glisser-déposer entre le serveur et cette machine. Voir Explorateur de fichiers et Téléverser et télécharger.
