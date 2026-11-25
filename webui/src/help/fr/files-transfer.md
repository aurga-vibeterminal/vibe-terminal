Les transferts de fichiers entre votre machine et le serveur sont du glisser-déposer, et la direction est le sens : **glisser vers l’intérieur = téléverser vers le serveur, glisser vers l’extérieur = télécharger vers cette machine**. Les transferts empruntent la connexion chiffrée client-serveur ; les serveurs distants et partagés se comportent comme les locaux.

## Téléverser : glisser vers l’intérieur

Faites glisser des fichiers (ou des dossiers entiers) depuis le gestionnaire de fichiers de votre système sur la liste de fichiers et relâchez :

- en mode arbre, **déposer sur une ligne de dossier** téléverse dans ce dossier ; **déposer à la racine** (la ligne d’insertion entre les lignes) téléverse dans le répertoire racine courant ;
- une ligne de transfert apparaît en bas de la liste affichant `Uploading`, le nom du fichier et une barre de progression ;
- l’achèvement affiche `Uploaded n file(s)` ; les échecs incluent la raison (p. ex. permission refusée) et la liste se rafraîchit pour révéler les nouveaux fichiers.

## Télécharger : glisser vers l’extérieur

Pressez un fichier dans la liste, faites-le glisser hors de la fenêtre du client, et déposez-le dans le gestionnaire de fichiers de votre système (ou sur le bureau) :

- le fichier descend en flux du serveur vers l’endroit du dépôt ;
- les lignes de transfert affichent la progression pareillement ; **une ligne de téléchargement terminée est cliquable** et ouvre le dossier de téléchargement local ;
- sélection multiple et glisser plusieurs fichiers d’un coup.

![Progression des transferts — les lignes de téléversement/téléchargement](images/files-transfer.jpg)

> Le glisser-déposer déplace des fichiers individuels entre cette machine et un serveur ; pour déplacer des données entre deux serveurs, combinez Publier avec un serveur HTTP ou des commandes de terminal (scp/rsync).
