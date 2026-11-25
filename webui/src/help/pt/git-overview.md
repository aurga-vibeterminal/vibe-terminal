O painel Git embute controle de versão no cliente: staging, commits, gerenciar branches, worktrees, rebase interativo, resolução de conflitos e push — sem sair da sua janela de terminal. Tudo opera no repositório Git da sessão de terminal atual: no diretório em que a sessão roda está o repositório que o painel mostra.

## Abrindo

Clique na aba `Git` na barra de atividades esquerda (dados do repositório só aparecem quando a sessão ativa está dentro de um repositório Git). O repositório de uma aba segue sua sessão ativa; a trilha do repositório permite saltar entre repositórios pai e filho, e submódulos são listados à parte.

![O painel Git — mudanças, histórico e branches](images/git-overview-panel.jpg)

## O que ele faz

- **Staging e commits** — stage/unstage por arquivo ou em massa, descartar mudanças, commit com mensagem, mensagens de commit geradas por IA (veja Assistência LLM);
- **Histórico** — histórico de commits e por arquivo, comparar e restaurar entre revisões;
- **Branches** — checkout, criar e excluir branches locais/remotas e tags;
- **Worktrees** — extrair uma branch para uma árvore de trabalho isolada (uma nova aba de terminal abre automaticamente), mesclá-la de volta ao terminar (veja Worktrees);
- **Rebase interativo** — reordenar/squash/editar visualmente a fila de commits com resolução de conflitos (veja Rebase interativo);
- **Resolução de conflitos** — um editor de mesclagem com quatro modos, incluindo um agente de IA que resolve hunks um a um (veja Resolver conflitos);
- **Push e vários** — um diálogo de push completo, stashes, tratamento de HEAD desanexado (veja Push, checkout e mais).

As páginas seguintes cobrem cada área nessa ordem.
