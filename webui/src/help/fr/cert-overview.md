Le serveur protège tous les points de terminaison TLS qu’il expose — l’interface Web, les connexions des clients et les ports redirigés — avec un certificat émis par sa propre **AC racine privée**. La page `Certificats et TLS` de l’interface Web est l’endroit où vous gérez cette AC, contrôlez pour quels noms le certificat est valide, et remettez l’AC aux clients qui veulent vérifier le serveur strictement.

## Fonctionnement

- Le serveur génère une AC racine privée une seule fois et la conserve pendant toute la vie de l’installation. Tout est signé par elle ;
- L’AC signe un **certificat feuille** pour le serveur avec une durée de validité de 90 jours, couvrant les noms d’hôte et les adresses IP que les clients utilisent réellement pour l’atteindre ;
- Lorsque les noms d’hôte, les interfaces réseau ou les entrées SAN de la machine changent, le certificat feuille est réémis automatiquement — l’AC racine ne change jamais, les clients qui lui font déjà confiance continuent donc de faire confiance au serveur ;
- Le nouveau certificat feuille est échangé via un résolveur de certificats partagé et rechargé à chaud : les connexions existantes ne sont jamais interrompues, et les nouvelles négociations TLS utilisent immédiatement le dernier certificat. Si un renouvellement échoue, le certificat précédent reste actif.

![La page Certificats et TLS : en haut la carte AC racine privée, au milieu la gestion des SAN, en bas les détails du certificat feuille du serveur](images/certificates-page.jpg)

## Ce que vous trouverez sur la page

1. `AC racine privée` — téléchargez ou copiez le certificat public de l’AC pour le distribuer aux clients (et à leurs navigateurs). Le téléchargement ne contient jamais de clé privée ;
2. `Domaines et adresses IP (SAN)` — la liste des noms pour lesquels le certificat feuille est valide. Décrite à la page suivante, Domaines et gestion des SAN ;
3. `Certificat feuille du serveur` — le certificat actif : état, sujet, empreinte SHA-256, période de validité et entrées SAN.

## Certificats TLS personnalisés

Si un certificat TLS personnalisé est configuré pour le serveur, c’est lui qui reste en charge : le certificat géré de cette page continue d’être généré et maintenu à jour, mais il ne remplace pas l’identité personnalisée configurée. Tant qu’une identité personnalisée est active, la page affiche un avis.

## Pourquoi faire confiance à l’AC racine

Les clients qui n’ont besoin que du chiffrement fonctionnent sans configuration. Les clients qui veulent *vérifier* le serveur — le mode strict décrit à la dernière page, Vérification stricte côté clients — ont besoin du certificat public de l’AC racine, et il doit transiter par un canal de confiance :

> Téléchargez l’AC racine uniquement depuis l’interface Web locale de ce serveur ou via un tunnel de confiance, et vérifiez son empreinte SHA-256 (affichée sur cette page) avant d’activer la vérification stricte où que ce soit.

Le même AC signe aussi les certificats utilisés par la redirection de ports (voir la page Redirection de ports). Une fois qu’un appareil fait confiance à l’AC racine, les URL redirigées s’ouvrent sans l’avertissement du navigateur *Votre connexion n’est pas privée*.
