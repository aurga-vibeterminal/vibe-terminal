Variáveis de ambiente permitem manter conjuntos agrupados de variáveis por projeto (`ANDROID_HOME`, chaves de API, ajustes de proxy) e aplicá-las ou removê-las em qualquer sessão de terminal com um clique. Vivem na configuração do servidor; os valores são criptografados em repouso.

## Manutenção na interface Web

A seção `Environment Variables` as organiza como árvore:

- **Grupos** — contêineres puros, aninháveis (botão direito → `New Group`), para organizar por projeto ou cenário;
- **Coleções** — as folhas da árvore e as verdadeiras portadoras de variáveis (`New Leaf`); apenas coleções podem ser aplicadas a um terminal;
- Os pares chave/valor de cada coleção são editados via `Edit Variables` (`Add Variable`); os valores ficam ocultos por padrão;
- Coleções podem ser duplicadas para derivar variantes (o mesmo conjunto com uma chave trocada).

![A página Environment Variables — a árvore grupos/coleções](images/terminal-env-tree.jpg)

## Aplicar a um terminal

Botão direito dentro de um terminal → `Envs`:

- o submenu espelha a árvore da interface Web; expanda uma coleção e escolha `Apply` para injetar suas variáveis na sessão atual, ou `Clear` para removê-las de novo;
- processos recém-iniciados veem as variáveis (programas em primeiro plano já em execução não);
- coleções aparecem dobradas sob seus grupos; o item `Envs` inteiro se esconde quando a árvore não tem coleções.

![Botão direito do terminal — aplicar e remover no submenu Envs](images/terminal-env-menu.mp4)
