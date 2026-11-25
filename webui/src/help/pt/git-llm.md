A inteligência do painel Git é movida por LLM: mensagens de commit com um clique, leitura de diff assistida por IA e resolução de conflitos por agente, hunk a hunk. Tudo depende de um provedor LLM configurado na interface Web.

## Configurar o provedor

Abra a seção `LLM` da interface Web:

- escolha um `Provider` e preencha suas credenciais (os campos são definidos pelo manifesto do provedor); a configuração é global — classificação de terminal, mensagens de commit do Git, o assistente de chat e outros recursos de IA compartilham o mesmo provedor;
- provedores são addons do tipo LLM: os comuns vêm embutidos, e provedores personalizados podem ser enviados como zip na seção `Addons` (o manifesto declara os campos de configuração que a interface Web transforma num formulário).

![A seção LLM da interface Web — escolha um provedor e preencha suas credenciais](images/git-llm-provider-config.jpg)

## Recursos LLM no Git

- **Geração de mensagem de commit** — clique no botão IA ao lado da caixa de commit; o LLM lê o diff staged e escreve uma mensagem, digitada na caixa com uma animação. A mensagem continua editável — faça commit só quando estiver satisfeito;
- **Análise de diff** — o botão brilho no cabeçalho da visão de diff analisa o diff atual;
- **Agente IA de conflitos** — o modo IA do editor de mesclagem do rebase resolve hunks um a um com explicações e confiança (veja Resolver conflitos).

![A mensagem de commit gerada por IA digitada na caixa de commit](images/git-llm-commit-message.jpg)

> O LLM só lê conteúdo do repositório para produzir texto; commits e mesclagens sempre acontecem quando você os confirma.
