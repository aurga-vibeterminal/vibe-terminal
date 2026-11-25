A área `CHANGES` do painel Git cobre o ciclo diário de commit: revisar mudanças, staging, descartar, stash e commit com mensagem.

## Stage e unstage

- mudanças não staged são listadas sob `CHANGES`, cada arquivo marcado com uma insígnia de estado — `M` modificado, `A` adicionado, `D` excluído, `?` não rastreado, `!` conflito (linhas de submódulo carregam uma etiqueta `SUB` extra);
- passar o cursor sobre uma linha revela botões de ação inline: `+` faz stage dum arquivo não staged, `−` unstage dum staged, e `×` numa linha não staged o descarta; os botões `Stage All Changes` / `Unstage All Changes` nos cabeçalhos de seção cuidam de tudo de uma vez;
- `Ctrl`-clique alterna arquivos para uma seleção múltipla e `Shift`-clique seleciona um intervalo — um botão de ação então aplica a cada arquivo selecionado;
- clicar num nome de arquivo abre seu diff para revisar a mudança antes de decidir;
- o botão direito num arquivo oferece `View File History`, `Stash This File`, `Copy Full Path` / `Copy Relative Path`, `Reveal in File Explorer` e um atalho `cd` que leva o terminal à pasta do arquivo.

![A seção STAGED CHANGES — arquivos staged com botões Unstage por linha](images/git-staging-staged.jpg)

## Commit

1. Faça stage do que deve entrar;
2. digite uma mensagem na caixa de commit (`Ctrl+Enter` faz commit) ou clique no botão IA para gerar uma (veja Assistência LLM);
3. clique em `Commit`.

![A caixa de commit — entrada de mensagem com o botão de geração IA](images/git-staging-commit.jpg)

Após o commit, passe o cursor sobre o cabeçalho do histórico — a barra com o nome da branch acima do histórico de commits — e os botões `Fetch`, `Pull` e `Push` surgem à direita. `Push` envia a branch atual ao seu upstream imediatamente; para opções como `Force with lease`, use o diálogo de push da lista de branches (veja Push, checkout e mais).

![Passar o cursor sobre o cabeçalho do histórico revela os botões Fetch, Pull e Push](images/git-staging-push.jpg)

## Stash

- o botão de arquivo no cabeçalho `CHANGES` abre o diálogo `Stash All Changes`, onde uma mensagem opcional descreve o stash;
- o botão direito em arquivos não staged selecionados oferece `Stash This File` (ou `Stash <n> Selected Files` para seleção múltipla) para fazer stash só daqueles caminhos — linhas de submódulo são excluídas;
- stashes se reúnem sob `STASHES` como entradas `WIP on <branch>`; expandir uma lista os arquivos capturados, e clicar num arquivo mostra seu diff;
- passar o cursor sobre uma linha de stash mostra `Apply` (restaurar e manter), `Pop` (aplicar e remover) e `Drop` (remover sem aplicar) — `Pop` e `Drop` pedem confirmação primeiro, e o menu de contexto adiciona `View Diff`;
- a seção `STASHES` fica oculta enquanto a lista de stashes está vazia.

![O diálogo Stash All Changes com sua mensagem opcional](images/git-staging-stash-all.jpg)

![A seção STASHES — um stash expandido para seus arquivos, com botões Apply, Pop e Drop](images/git-staging-stashes.jpg)

![Pop aplica o stash e o remove da lista após uma confirmação](images/git-staging-stash-pop.jpg)

## Descartar

`Discard Changes` num arquivo não staged joga fora todas as mudanças não commitadas nele — há um diálogo de confirmação e a ação **não pode ser desfeita**.

> Um commit inclui apenas conteúdo staged; mudanças não staged permanecem na árvore de trabalho.
