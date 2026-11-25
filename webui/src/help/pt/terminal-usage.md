Terminais são o núcleo do Vibe Terminal: cada aba hospeda uma ou mais sessões de terminal, pode ser dividida em painéis ou arrastada para a própria janela. Os terminais alcançam o servidor local assim como servidores remotos e compartilhados através de um relay.

## Criar terminais

- Clique no botão `+` da barra de título para abrir uma aba com o perfil padrão;
- Clique na seta ao lado de `+` para o menu de perfis: perfis de terminal agrupados por servidor (local, conectados, compartilhados) — expanda um grupo e escolha um perfil para abrir um terminal naquele servidor;
- Arraste uma pasta local para a barra de título e solte para abrir uma nova aba com aquele diretório de trabalho (somente servidor local; o item solto deve ser uma pasta).

![Barra de título — o botão de nova aba e o menu de perfis](images/terminal-usage-creation.jpg)

## Captura do mouse e menu de contexto

- Clique com o botão direito dentro de um terminal abre seu menu de contexto: colar, dividir painéis, comandos rápidos, coleções de ambiente, copiar todo o texto, linhas de configuração MCP e mais, dependendo da sessão;
- Aplicativos que capturam o mouse (opencode, pi, vim, tmux, htop, …) recebem os eventos do mouse em seu lugar — segure `Shift` para realizar mesmo assim as operações habituais enquanto esse app roda:
  - **Copiar texto**: segure `Shift` e selecione o texto, depois clique em outro lugar — `Shift` ainda pressionado — para desmarcar; a seleção é copiada para a área de transferência;
  - **Menu de contexto**: `Shift` + clique com o botão direito ou `Ctrl` + clique com o botão direito abre o menu de contexto; um clique com o botão direito enquanto o botão esquerdo está pressionado também o abre (qualquer seleção é limpa antes);
  - **Links**: `Shift` + clique abre links normalmente.

## Divisões e organização por arrastar

- **Dividir**: clique com o botão direito dentro de um terminal → `Split` → `Left` / `Right` / `Top` / `Bottom` divide a aba em painéis, cada um um terminal independente;

  ![Dividindo uma aba — o menu Split do clique direito e os painéis resultantes](images/terminal-usage-split.gif)

- **Mesclar arrastando**: arraste uma aba sobre outro terminal e zonas de encaixe aparecem (esquerda/direita/topo/baixo); solte para mesclar a aba na aba alvo como um novo painel;

  ![Mesclagem por arrastar — uma aba solta sobre outro terminal torna-se um novo painel](images/terminal-usage-merge.mp4)

- **Desanexar para uma janela**: arraste uma aba para fora da faixa de abas (ou sobre outra janela) e a aba inteira deixa sua janela como uma nova. As sessões sempre rodam no servidor — divisões, mesclagens e arrastos nunca interrompem um comando em execução.

## Abas e grupos de abas

- **Aba** — uma aba de terminal, possivelmente com vários painéis divididos;
- **Grupo de abas** — um contêiner que organiza abas. Por padrão os grupos seguem os servidores: as abas do servidor local ficam em `Ungrouped`, e cada servidor compartilhado recebe um grupo `Name (Server)`; grupos personalizados são suportados;
- **Grupos e janelas** — cada janela mostra um grupo ativo pelo seletor de grupos da barra de título (ou `All`); `Move Group to New Window` destaca um grupo inteiro para a própria janela, dedicando uma janela ou monitor a uma máquina;
- **Grupos e servidores** — um grupo lembra seu servidor, então trocar de grupo refocaliza o espaço de trabalho de um servidor; o menu de perfis segue o grupo ativo e mostra apenas perfis de servidores relevantes.

![O seletor de grupos — lista de grupos e mover para nova janela](images/terminal-usage-groups.jpg)
