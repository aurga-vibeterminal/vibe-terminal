L’interface Web est la page de configuration propre au serveur : chaque fonction du serveur est organisée en section, listée dans la barre latérale gauche. Ce n’est pas un terminal — le travail quotidien s’effectue dans le client de bureau ou `vtermc` (voir Présentation). Cette page explique la disposition et le comportement communs ; les chapitres de fonction détaillent les paramètres de chaque section.

## Disposition

- **Barre latérale** — la liste des sections (ci-dessous), l’icône d’aide à côté du nom du serveur et la version de l’application en bas ; sur les écrans étroits, elle apparaît en glissant depuis la gauche et le bouton hamburger de l’en-tête l’affiche ou la masque ;
- **En-tête** — le titre de la section courante plus, en haut à droite, le sélecteur de langue et le commutateur de thème clair/sombre (sur les écrans étroits, l’icône d’aide s’y trouve aussi) ;
- **Contenu** — le formulaire de la section sélectionnée, centré et à défilement.

## Sections

Dans l’ordre de la barre latérale :

- `Serveur` — port, certificat TLS et paramètres d’authentification : la `Secret Key`, le `QR Code` d’appairage, l’adresse du relais et le commutateur `Auto-start on Login` se trouvent tous ici (voir Connecter un client et Guide d’installation). Sous Linux, l’astuce d’`Auto-start on Login` explique aussi la connexion automatique du gestionnaire d’affichage pour atteindre le bureau avant une connexion locale (voir Configuration du serveur dans Bureau à distance et simulateurs) ;
- `Terminal` — analyse du terminal : détection de l’invite, confirmation automatique et classification LLM (voir Confirmation automatique) ;
- `UART` — profils de terminal série adossés aux ports UART locaux (voir Série UART) ;
- `Addons` — addons installés : les activer, les modifier ou les téléverser (les fournisseurs LLM sont des addons — voir LLM) ;
- `LLM` — attribue un fournisseur LLM à chaque fonction IA (voir LLM) ;
- `Voix` — fournisseur de reconnaissance vocale, modèle et paramètres de transcription (voir Configuration du serveur dans Saisie vocale) ;
- `Environnement` — collections de variables d’environnement dans une arborescence de groupes (voir Variables d’environnement) ;
- `MCP` — le serveur MCP du terminal exposé aux agents de code (voir Terminal MCP) ;
- `DDNS` — fournisseur DDNS et paramètres de synchronisation (voir DDNS) ;
- `Redirection de ports` — le pool de ports redirigés et les redirections TLS actives (voir Redirection de ports) ;
- `Static HTTP` — dossiers servis en HTTP et partages de fichiers actifs (voir Publier avec un serveur HTTP) ;
- `Bureau à distance` — paramètres de capture, de saisie et de flux pour le bureau à distance (voir Configuration du serveur dans Bureau à distance et simulateurs) ;
- `Statistiques LLM` — utilisation de jetons, nombre de requêtes et métriques de performance de toutes les fonctions LLM.

## Enregistrement et astuces

- La plupart des sections s’enregistrent automatiquement peu après une modification — un indicateur d’enregistrement et toute erreur apparaissent en bas de la section ;
- `MCP`, `DDNS` et `Bureau à distance` conservent les modifications jusqu’au clic sur `Enregistrer les modifications` : tant que des modifications sont en attente, une mention `Modifications non enregistrées` apparaît à côté des boutons, et `Réinitialiser` les abandonne ;
- Ouverte depuis le client mobile, l’interface Web suit la langue et le thème de l’application et masque ses propres contrôles de langue et de thème.
