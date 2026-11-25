A tradução transforma o texto que você encontra no cliente no idioma da sua escolha: respostas do chat de IA, mensagens de commit, comentários de código ou qualquer seleção no editor de código ou num diff. O trabalho é feito por um modelo de tradução configurado no servidor — o cliente só envia o texto, renderiza o resultado e o armazena em cache para você alternar entre o original e a tradução.

## Onde a tradução está disponível

- **Painel do Copilot** — todo cabeçalho de mensagem (as suas, as respostas do modelo e os blocos `Pensando...`) tem um ícone de tradução. Um clique substitui o corpo da mensagem pela tradução; outro clique restaura o original;
- **Rascunho da mensagem de commit** — a caixa de commit do painel Git ganha um botão de tradução assim que o rascunho ocupa várias linhas. Ele traduz o rascunho inteiro no lugar, e mais um clique traz sua redação original de volta;
- **Popup de detalhes do commit** — passar o mouse sobre um commit no histórico Git abre um popup de detalhes; o ícone de tradução ao lado do hash troca a mensagem de commit para o seu idioma;
- **Editor de código e visualização de diff** — selecione qualquer coisa (seleções em bloco também contam), clique com o botão direito e escolha `Traduzir`. Uma pequena insígnia de globo marca as linhas traduzidas na medianiz, e um cartão com a tradução abre ao lado da seleção.

## Como funciona

1. O cliente envia o texto ao servidor — junto com o nome do arquivo quando o texto foi selecionado no editor ou num diff, para o modelo saber qual sintaxe de comentário o arquivo usa;
2. O LLM de tradução do servidor o traduz para o idioma de destino configurado na interface Web. A formatação Markdown (títulos, listas, tabelas, blocos de código) é preservada; em seleções de arquivos, marcadores de comentário como `//` ou `#` são reconhecidos como decoração e removidos, enquanto o código e os identificadores ficam intactos;
3. O cliente renderiza o resultado como Markdown de onde veio a solicitação — no lugar para mensagens de chat, ou num cartão rolável para seleções do editor e de diffs.

## O que você precisa

- Um servidor com o recurso de tradução habilitado — a configuração única está em *Configuração do servidor* (seção `LLM` da interface Web);
- Um cliente desktop conectado a esse servidor. Nada é configurado no cliente: todas as superfícies usam automaticamente as mesmas configurações do servidor.

Se a tradução não estiver configurada, o cliente mostra o erro do servidor (por exemplo no cartão de tradução) em vez de um resultado.
