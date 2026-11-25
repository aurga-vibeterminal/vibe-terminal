L’explorateur de fichiers opère sur le système de fichiers côté serveur. La barre d’outils en haut du panneau gauche bascule entre Recent / Explorer / Git / Debug / Search ; Explorer est le navigateur de fichiers.

## Le bouton Home : sauter vers le cwd du terminal

Cliquez sur le bouton `Home` (maison) de la barre d’outils :

- bascule vers la vue `Explorer` ;
- saute au **répertoire de travail du terminal actif** (son cwd) ;
- épingle ce répertoire en arbre et vide la boîte de recherche — vous partez toujours de « là où est le terminal, c’est ce que vous voyez ».

![L’explorateur de fichiers en mode normal — Home, le fil d’Ariane et la liste plate](images/files-explorer.jpg)

## Mode arbre et recherche

- **Épingler en arbre** : le bouton épingle à l’extrémité droite du fil d’Ariane bascule entre liste plate et mode arbre. En mode arbre les dossiers se déploient en arborescence repliable (le saut Home épingle automatiquement) ;
- **Recherche** : la boîte de recherche en haut filtre la liste courante par nom à la frappe ; `Search in Folder` du menu contextuel de Recent lance une recherche récursive sur tout un répertoire.

![L’explorateur de fichiers en mode arbre — la racine épinglée se déploie en arborescence repliable](images/files-explorer-tree.jpg)

## Opérations de base

Clic droit sur un fichier ou dossier :

- **Créer** : `New Folder` / `New File` ;
- **Éditer** : `Rename` ; `Copy` / `Cut` / `Paste` (fonctionne entre répertoires) ;
- **Supprimer** : `Delete`, avec une boîte de confirmation ;
- **Lien terminal** : `Open in Terminal` ouvre un nouvel onglet de terminal dans ce répertoire ; `cd to Directory` fait cd du terminal courant ;
- **Lien système** : `Reveal in File Explorer` montre un répertoire local dans le gestionnaire de fichiers du système ;
- **Chemins** : `Copy Full Path` / `Copy Relative Path` ;
- **Publier** : `Host with HTTP server` publie le répertoire comme service de fichiers HTTP (voir Publier avec un serveur HTTP) ; `Open in App` l’ouvre avec une application d’agent détectée sur le serveur.

Chaque opération agit sur le système de fichiers côté serveur ; les sessions distantes se comportent exactement comme les locales.
