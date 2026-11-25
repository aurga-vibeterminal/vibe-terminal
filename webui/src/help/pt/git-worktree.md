Worktrees extraem várias branches de um repositório para diretórios separados: desenvolva em paralelo sem tocar a árvore de trabalho principal, depois mescle a branch de volta e limpe. O Vibe Terminal transforma todo o ciclo em apontar e clicar.

## Criar um worktree

1. Na lista de branches do painel Git, botão direito na branch alvo (local ou remota) → `New Worktree`;
2. o diálogo nomeia o novo worktree pela branch — edite se quiser; confirme, e o servidor cria o diretório do worktree e extrai a branch lá;
3. **uma nova aba de terminal abre automaticamente** com seu diretório de trabalho no novo worktree — comece a desenvolver isolado imediatamente enquanto a árvore principal fica na sua branch.

![O diálogo New Worktree — o nome é por padrão a branch](images/git-worktree-create.jpg)

![A nova aba aberta para o worktree recém-criado](images/git-worktree-terminal.jpg)

## Trabalhando em paralelo

Cada worktree é um diretório de trabalho totalmente independente: builds e testes da branch principal e da branch de funcionalidade rodam lado a lado em abas diferentes sem interferência. Branches extraídas num worktree carregam um marcador de worktree mostrando seu caminho na lista de branches, e commit e push funcionam direto da aba do worktree como em qualquer branch.

O botão direito numa branch de worktree adiciona duas entradas exclusivas: `Reveal in File Explorer` (sessões locais) abre seu diretório no gerenciador de arquivos do sistema, e `Open in Terminal` abre outra aba lá.

## Mesclar de volta e limpar

1. Faça commit e push dentro da aba do worktree como de costume;
2. mude a aba principal para a branch alvo (p. ex. `main`), botão direito na branch de funcionalidade → `Merge into Current Branch` para trazer o trabalho;

   ![Mesclando a branch do worktree de volta para a branch principal](images/git-worktree-merge.jpg)

3. mesclado, botão direito na branch → `Delete Worktree`: isto remove o diretório do worktree e sua branch juntos. Certifique-se antes de que tudo esteja mesclado ou enviado — o diálogo avisa de novo e a ação não pode ser desfeita.

   ![A confirmação Delete Worktree — diretório e branch são removidos juntos](images/git-worktree-delete.jpg)

