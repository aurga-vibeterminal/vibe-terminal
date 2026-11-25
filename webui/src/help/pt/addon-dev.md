Há dois níveis de personalização: corrigir o código de um addon existente, ou escrever um novo modeledo num embutido e enviá-lo.

## Patch a quente do código

O botão de lápis em qualquer cartão de addon abre seu código JavaScript para edição. Salvar armazena um instantâneo que sobrescreve o original — para addons embutidos e enviados alike — e a mudança vale imediatamente, sem reiniciar. Use para ajustar um prompt, corrigir uma URL ou prototipar uma ideia no sistema ao vivo.

`Reset` descarta o instantâneo e restaura o código pristino (somente embutidos — um addon enviado já é seu).

![Editando o código de um addon — o instantâneo salvo sobrescreve o original](images/addon-edit-source.jpg)

## Escrevendo seu próprio addon

Um addon são dois arquivos:

- `<id>.json` — o manifesto: `id`, `name`, `description`, `author`, `email`, `date`, `type` e `config_fields` (key, label, tipo de campo, padrão, descrição) que a Web UI transforma em formulário;
- `<id>.js` — o código: um arquivo JavaScript comum (executado pelo motor QuickJS embutido, com uma ponte `addon.get_config()` para os campos configurados) que exporta as funções que seu tipo chama — p. ex. `chat_completion()` para um provedor LLM, listagens de ferramentas e chamadas de ponte para um addon MCP.

O começo mais rápido é exportar um addon existente: o botão `Export` no cartão baixa um zip com o manifesto e o código juntos. Escolha o embutido mais próximo do seu objetivo (eles também vivem em `crates/vterm-server/src/addon/builtins/`), ajuste o id, name, type e as funções exportadas, e instale o resultado com `Upload Addon`. Addons enviados podem ser apagados de novo com o botão de lixeira.

![Exportar um addon — manifesto e código JavaScript num único zip](images/addon-export.jpg)

## Armazenamento

Addons enviados e instantâneos de código vivem na pasta `addons/` do diretório de configuração como `<id>.json` + `<id>.js`, então sobrevivem a reinícios. Para um id embutido, apenas a sobrescrita `.js` é honrada — o embutido mantém o próprio manifesto.
