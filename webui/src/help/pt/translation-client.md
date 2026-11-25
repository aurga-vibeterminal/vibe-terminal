Com o servidor configurado, todas as superfícies do cliente oferecem tradução pelo mesmo ícone de globo. Os resultados são armazenados em cache por item, então alternar de um lado para o outro nunca repete uma solicitação.

## Mensagens do painel de chat

Todo cabeçalho de mensagem tem um ícone de tradução — ao lado da hora nas suas mensagens, à direita do nome do remetente e da hora nas respostas do modelo, e à direita do rótulo `Pensando...` nos blocos de raciocínio:

- **Primeiro clique** envia o Markdown bruto da mensagem ao servidor e troca o corpo da mensagem pela tradução. O ícone acende enquanto a tradução é exibida;
- **Outro clique** restaura o texto original;
- A tradução acompanha a mensagem enquanto a conversa permanece aberta, incluindo blocos `Pensando...` depois de expandidos.

Mensagens traduzidas renderizam como Markdown normal, então listas, tabelas e blocos de código mantêm a forma.

![Uma mensagem de chat com o ícone de tradução no cabeçalho e o corpo traduzido abaixo](images/translation-chat-message.jpg)

## Rascunho da mensagem de commit

Na caixa de commit do painel Git, um botão de tradução aparece ao lado do botão de IA assim que o rascunho passa de uma linha:

- **Clique** para traduzir o rascunho inteiro — o texto traduzido substitui o rascunho na caixa como pré-visualização somente leitura;
- **Clique de novo** para voltar à sua redação original (o rascunho em si nunca foi modificado);
- A digitação fica desabilitada enquanto a pré-visualização é exibida; volte primeiro e continue editando.

![A caixa de commit com o botão de tradução e a pré-visualização traduzida](images/translation-commit-draft.jpg)

## Popup de detalhes do commit

Passe o mouse sobre um commit no histórico Git para abrir o popup de detalhes. Ao lado do botão de copiar o hash fica um ícone de tradução: clique para traduzir a mensagem de commit dentro do popup; clique de novo para ver o original. A mensagem traduzida é renderizada como a original — linha de assunto, corpo em Markdown e blocos de código inclusos.

![O popup de detalhes do commit — botões copiar e traduzir na linha do hash](images/translation-git-tip.jpg)

## Seleções no editor de código

Selecione texto no editor de código — seleções normais e em bloco (retangulares) funcionam — clique com o botão direito e escolha `Traduzir` (a primeira entrada do menu):

1. Uma insígnia de globo sólida aparece sobre o número da linha da primeira linha selecionada;
2. Um cartão de tradução abre imediatamente com um marcador *Translating…* e se preenche quando o servidor responde;
3. O cartão é grudento: fica no lugar enquanto você rola, e só o botão de fechar, `Esc` ou um clique fora o descartam. Passar o mouse sobre a insígnia o traz de volta.

Cada seleção traduzida ganha sua própria insígnia, e traduzir as mesmas linhas de novo simplesmente reexibe o cartão em cache. As insígnias desaparecem quando você abre outro arquivo.

![Uma seleção no editor de código com Traduzir no menu de contexto](images/translation-editor-menu.jpg)

## Seleções na visualização de diff

A visualização de diff funciona da mesma forma: selecione linhas no corpo do diff, botão direito, `Traduzir`. A insígnia substitui o novo número de linha da primeira linha selecionada, e o cartão do resultado ancora abaixo dela. Seleções no modo de edição também traduzem; trocar para outro arquivo ou commit limpa as insígnias.

![Um diff com uma seleção traduzida — insígnia na medianiz e cartão de tradução](images/translation-diff-card.jpg)

## O que é traduzido

O servidor diz ao modelo de onde veio o texto:

- **Prosa** (mensagens de chat, mensagens de commit) — a formatação Markdown é preservada e apenas a linguagem natural é traduzida;
- **Seleções de arquivo** (editor, diff) — o nome do arquivo viaja com a solicitação, então o modelo conhece os marcadores de comentário do arquivo (`//`, `#`, `--`, …). Os marcadores de comentário são removidos e o texto do comentário é traduzido como prosa; o Markdown dentro do comentário (títulos, listas, blocos de código) sobrevive. Se a seleção é código comum, apenas comentários e literais de string são traduzidos — identificadores e estrutura ficam intocados.
