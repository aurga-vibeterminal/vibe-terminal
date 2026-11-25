O rebase interativo transforma `git rebase -i` num fluxo visual: arraste para reordenar a fila de commits, aplique os commits um a um, resolva conflitos visualmente — sem linha de comando.

## Começando

Abra o painel de branches e digite na caixa `Filter branches...` para achar a branch alvo, depois botão direito nela → `Rebase Current Branch onto '{branch}'` para rebasear a branch atual sobre o alvo. O painel muda para a visão de rebase, que avança por quatro fases:

![Busca de branch no painel, depois iniciando o rebase sobre o alvo](images/git-rebase-search.jpg)

## 1. Editar a fila de commits

Os commits da sua branch que faltam ao alvo formam uma fila:

- **arraste** para reordená-los;
- cada commit tem um menu de ação: `pick` (manter) / `squash` (dobrar no anterior, mesclar mensagens) / `fixup` (dobrar, descartar mensagem) / `edit` (parar para corrigir) / `drop` (descartar);
- commits squashed e droppados dobram visualmente.

![A fila de commits do rebase — reordenar arrastando, ações por commit](images/git-rebase-queue.jpg)

## 2. Aplicar

`Start Rebase` reproduz a fila sobre o alvo commit a commit com uma barra de progresso ao vivo; sem conflitos corre direto até terminar. `Abort` recua a qualquer momento.

## 3. Conflitos (se houver)

Um conflito durante a aplicação muda para a visão de conflitos — uma lista de arquivos mais o editor de mesclagem com seus quatro modos (`Merge` / `Split` / `Raw` / `AI`), onde cada hunk em conflito ganha sua própria decisão (`Ours` / `Theirs` / `Both`…; veja Resolver conflitos). Assim que cada arquivo é resolvido, `Continue` retoma o rebase.

![A visão de conflitos — decisões de mesclagem por hunk](images/git-rebase-resolve-conflict.jpg)

## 4. Concluído

Na conclusão o painel confirma `Branch '{branch}' was rebased onto '{target}'` e mostra o cartão de push — o rebase reescreveu o histórico, então a branch remota precisa de atualização forçada:

- `Force Push with Lease` — recomendado: sobrescreve somente quando ninguém mais enviou e nunca usa um `--force` simples;
- `Not now` — pular o push; o rebase local já está completo.

![A página de conclusão — Force Push with Lease ou pular o push](images/git-rebase-done.jpg)

> A sessão continua utilizável durante o rebase; fechar o painel não interrompe um em execução.
