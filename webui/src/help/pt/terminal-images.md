Capturas de tela e terminal se encontram num mesmo fluxo: um único atalho global agarra qualquer região da área de trabalho, você a anota e copia ou salva — depois cola a imagem da área de transferência direto num terminal, onde ela aterrissa como caminho de arquivo que o programa em execução (um agente de código inclusive) pode abrir e usar. Ambas as metades vivem no cliente desktop.

## Fazendo uma captura de tela

- Pressione `Shift`+`Alt`+`A` em qualquer lugar: o atalho é global, dispara não importa qual aplicativo tenha o foco enquanto o cliente desktop estiver rodando (no macOS, pressione `Shift`+`Command`+`A`);
- Cada tela escurece sob uma sobreposição de captura. Arraste para desenhar a seleção — ela pode atravessar várias telas — depois arraste dentro dela para mover, ou puxe suas alças para redimensionar; um crachá acima da seleção mostra o tamanho em pixels;
- Pressione `Esc` a qualquer momento para cancelar.

![A sobreposição de captura — uma seleção com seu crachá de tamanho e a barra de anotação](images/terminal-images-capture.jpg)

## Anotando

A barra sob a seleção carrega à esquerda as ferramentas de anotação (retângulo e elipse), no meio seis cores, e à direita três botões de ação — cancelar (`✕`), salvar (seta de download) e copiar (`✓`).

- Escolha uma ferramenta e uma cor, depois arraste dentro da seleção para desenhar; os traços são recortados à seleção;
- `Ctrl`+`Z` desfaz a última anotação, `Ctrl`+`R` a refaz;
- Arrastar uma área nova fora da seleção atual a descarta — anotações incluídas — e começa outra.

## Copiar ou salvar

- `Copiar` (`✓`) põe a captura anotada na área de transferência — pronta para colar num terminal (abaixo), num chat ou em qualquer outro aplicativo;
- `Salvar` (seta de download) grava um PNG por meio de um diálogo de salvar; o nome de arquivo por padrão é `vterm-screenshot-<timestamp>.png`.

## Colando uma imagem num terminal

- Copie uma imagem para a área de transferência — da ferramenta de captura acima, de um navegador ou da captura própria do sistema operacional;
- Clique com o botão direito no terminal e escolha `Paste Image`; a entrada só aparece quando a área de transferência realmente contém uma imagem;
- A imagem é enviada ao servidor da sessão e guardada lá como arquivo temporário; seu caminho entre aspas é então digitado no cursor do terminal exatamente como se você o tivesse digitado — embrulhado em marcadores de colagem entre colchetes quando o aplicativo em execução os suporta, para que shells, editores e agentes o recebam limpo;
- Para o programa é só texto: um agente de código como Claude Code ou opencode abre a imagem pelo caminho, e comandos comuns de shell podem usá-lo como argumento.

![Clique direito no terminal — a entrada Paste Image](images/terminal-images-paste.jpg)

## Notas

- O caminho inserido vive no servidor da sessão: num servidor remoto ou WSL o arquivo temporário é criado naquela máquina, bem onde os programas rodando no terminal podem alcançá-lo;
- Combinado com `Adicionar contexto aos agentes`, é a maneira mais rápida de mostrar a um agente algo da sua tela — capture, cole, e o agente segura a imagem como arquivo;
- No Linux a captura funciona sob X11; sessões Wayland nativas não suportam nem o atalho global nem a sobreposição interativa, e se outro aplicativo X11 já agarrou `Shift`+`Alt`+`A`, o atalho não pode ser registrado.
