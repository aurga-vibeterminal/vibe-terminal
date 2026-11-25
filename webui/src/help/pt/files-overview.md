O gerenciamento de arquivos permite trabalhar com arquivos no servidor diretamente do cliente: a barra de atividades esquerda tem duas entradas — `Recent` (projetos recentes) e o explorador de arquivos — usadas junto com abas de terminal. Tudo opera no servidor da sessão: a máquina local, ou uma máquina remota/compartilhada por um relay.

## Recent: projetos recentes

`Recent` lista as pastas que seus agentes de código abriram mais recentemente no servidor atual, mais novas primeiro:

- **Clique** numa entrada e o explorador de arquivos muda para aquele diretório;
- **Botão direito** abre um menu de ações rápidas:
  - `Open in VS Code` — abrir o diretório no VS Code nesta máquina;
  - `Reveal in File Explorer` — mostrá-lo no gerenciador de arquivos do sistema;
  - `Copy Full Path` / `Copy Relative Path` — caminho completo, ou caminho relativo ao repositório;
  - `Search in Folder` — buscar arquivos dentro daquele diretório;
  - `Open in Terminal` — abrir uma nova aba de terminal lá; `cd Here` — fazer cd do terminal atual para ele;
  - `Host with HTTP server` — publicar o diretório como serviço de arquivos HTTP (veja Publicar com servidor HTTP);
  - `Open in App` — abri-lo com um aplicativo de agente detectado no servidor.

![O painel esquerdo — a lista de projetos recentes](images/files-overview.jpg)

## O explorador de arquivos

A aba `Explorer` navega e gerencia o sistema de arquivos do lado servidor: navegação de diretórios, visões árvore e plana, busca, criar/renomear/excluir e outras operações de arquivos, mais envio e download por arrastar e soltar entre o servidor e esta máquina. Veja Explorador de arquivos e Enviar e baixar.
