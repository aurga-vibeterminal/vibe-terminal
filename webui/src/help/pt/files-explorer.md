O explorador de arquivos opera no sistema de arquivos do lado servidor. A barra de ferramentas no topo do painel esquerdo alterna entre Recent / Explorer / Git / Debug / Search; Explorer é o navegador de arquivos.

## O botão Home: saltar para o cwd do terminal

Clique no botão `Home` (casa) da barra de ferramentas:

- Muda para a visão `Explorer`;
- Salta para o **diretório de trabalho do terminal ativo** (seu cwd);
- Fixa aquele diretório como árvore e limpa a caixa de busca — você sempre parte de «onde o terminal está é o que você vê».

![O explorador de arquivos em modo normal — Home, trilha de navegação e a lista plana](images/files-explorer.jpg)

## Modo árvore e busca

- **Fixar como árvore**: o botão de alfinete na extremidade direita da trilha alterna entre a lista plana e o modo árvore. No modo árvore pastas expandem numa árvore de diretórios recolhível (o salto do Home fixa automaticamente);
- **Busca**: a caixa de busca no topo filtra a lista atual por nome enquanto você digita; `Search in Folder` do menu de contexto do Recent executa uma busca recursiva sobre um diretório inteiro.

![O explorador de arquivos em modo árvore — a raiz fixada expandindo numa árvore de diretórios recolhível](images/files-explorer-tree.jpg)

## Operações básicas

Botão direito num arquivo ou pasta:

- **Criar**: `New Folder` / `New File`;
- **Editar**: `Rename`; `Copy` / `Cut` / `Paste` (funciona entre diretórios);
- **Excluir**: `Delete`, com diálogo de confirmação;
- **Ligação com terminal**: `Open in Terminal` abre uma nova aba de terminal naquele diretório; `cd to Directory` faz cd do terminal atual para lá;
- **Ligação com o sistema**: `Reveal in File Explorer` mostra um diretório local no gerenciador de arquivos do sistema;
- **Caminhos**: `Copy Full Path` / `Copy Relative Path`;
- **Publicar**: `Host with HTTP server` publica o diretório como serviço de arquivos HTTP (veja Publicar com servidor HTTP); `Open in App` o abre com um aplicativo de agente detectado no servidor.

Cada operação age no sistema de arquivos do lado servidor; sessões remotas comportam-se exatamente como locais.
