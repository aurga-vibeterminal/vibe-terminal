Quando um rebase ou mesclagem encontra conflitos, o painel muda para a visão de conflitos: a lista de arquivos em conflito à esquerda, o editor de mesclagem à direita. Arquivos em conflito também aparecem na lista `CHANGES` com uma insígnia `!` — clicar num salta direto para esta visão em vez de um diff simples. O editor oferece quatro modos de resolução e você pode alternar entre eles a qualquer momento.

![A visão de conflitos — lista de arquivos à esquerda, editor de mesclagem à direita](images/git-conflicts-view.jpg)

## A lista de arquivos em conflito

Arquivos são agrupados por estado: os automesclados com sucesso (staged automaticamente) e os que ainda precisam de atenção manual. Clique num arquivo para abri-lo no editor; a lista atualiza ao vivo conforme você resolve, até ficar vazia.

## Os quatro modos de resolução

- **Merge** (padrão) — conflitos são apresentados hunk a hunk; por hunk escolha `take ours` / `take theirs` ou edite o resultado mesclado inline;
- **Split** — um diff é quebrado em segmentos mais finos a nível de linha, para adotar os dois lados pedaço a pedaço dentro de um hunk;
- **Raw** — edição simples com marcadores de conflito `diff3` (`<<<<<<<` / `=======` / `>>>>>>>`) para ajuste manual;
- **AI** — um agente resolve o arquivo hunk a hunk: o LLM analisa cada hunk em conflito e produz cartões de solução com explicação e confiança. Você pode digitar instruções para guiar a estratégia (p. ex. «prefira theirs, mantenha os logs»), pausar, ajustar noutro modo e retomar — hunks já resolvidos são pulados.

Em cada modo a resolução é escrita no arquivo da sua sessão e staged, exatamente como uma edição manual.

## Continuando o rebase

Assim que todo conflito é resolvido, o rebase segue para o próximo commit; você também pode abortar a qualquer momento para voltar ao estado pré-rebase.
