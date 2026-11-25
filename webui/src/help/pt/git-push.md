O menu de contexto da lista de branches concentra push, checkout e as operações cotidianas de branches.

## Push

Botão direito numa branch → `Push '{branch}'...` abre o diálogo de push:

![Botão direito na branch — a entrada Push](images/git-push-contextmenu.jpg)

- ele mostra o mapeamento de alvos `local → remote`; qualquer um dos nomes pode ser mudado (empurrar para uma branch remota existente sob outro nome pede confirmação);
- opções: `Push all branches`, `Set upstream` (vincular upstream no primeiro push), `Push tags`, `Dry run`;
- o `Force` reescritor de histórico e o mais seguro `Force with lease` (recusa quando outra pessoa enviou — preferido). Operações que reescrevem histórico, como push após um rebase, exigem um destes;
- um aviso de risco sobre pushes forçados é mostrado no diálogo.

![O diálogo de push — alvos local → remote e as caixas de opções](images/git-push-dialog.jpg)

## Checkout

Botão direito numa branch → `Checkout` para trocar de branch — branches locais fazem checkout diretamente, uma branch remota ou tag abre o diálogo de checkout:

![Botão direito na branch — a entrada Checkout](images/git-checkout-contextmenu.jpg)

- `New branch name` — a branch local a criar, pré-preenchida a partir da referência;
- `Create new local branch` (`-B`), `Track remote as upstream` (`--track`, só branches remotas), `Force` (`-f`), `Merge` (`--merge`).

![O diálogo de checkout — nome da branch e opções](images/git-checkout-dialog.jpg)

O diretório de trabalho da sessão segue a troca; o terminal e o painel Git refletem a nova branch imediatamente.

## HEAD desanexado

Fazer checkout de um commit puro (p. ex. dar checkout num commit ao navegar o histórico) desanexa o HEAD. O próximo commit então abre o diálogo `Detached HEAD`: escolha uma branch existente para commitar, `+ Create new branch…` no lugar, ou `Skip` por enquanto.

## Mais operações

- **Stash** — faça stash de todas as mudanças para a pilha (mensagem opcional) do cabeçalho de mudanças; a lista `STASHES` suporta `Pop` (aplicar e remover) e `Drop` (descartar, irreversível);
- **Excluir branches/worktrees** — pelo menu de contexto da entrada, ambas com confirmações de ação irreversível;
- **Histórico e restauração** — `HISTORY` lista commits; botão direito para histórico de arquivo, diff contra a árvore de trabalho ou restaurar para uma revisão;
- **Copiar** — nomes de branches e hashes de commit copiam com um clique, útil para colar a um agente ou colega.
