Os addons estendem o servidor com novas capacidades sem tocar no núcleo: provedores LLM, backends de reconhecimento de voz, atualizadores de DNS dinâmico, superfícies de ferramentas MCP extras e a detecção de apps de agentes de código vivem todos como addons. A seção `Addons` da Web UI os gerencia.

## Categorias

Cada addon declara um tipo, e a seção agrupa os cartões de acordo:

- **Applications** — ensinam o observador de terminais a reconhecer uma CLI de agente de código (Claude Code, Codex, Kimi Code, Pi e OpenCode vêm embutidos): detecção de processo, estado ocupado/ocioso, ganchos de automação;
- **LLM Providers** — backends de conclusão de chat que os recursos de IA escolhem (um provedor compatível com OpenAI vem embutido);
- **Voice / ASR** — backends de reconhecimento de voz (o addon embutido `vibe-voice` aponta para o servidor ASR local supervisionado);
- **Dynamic DNS** — mantêm um hostname apontando para o endereço atual (Cloudflare vem embutido);
- **MCP** — superfícies de ferramentas MCP extras servidas no host MCP (EasyEDA Pro vem embutido);
- qualquer outra coisa cai no grupo **Other**.

## Addons embutidos e enviados

- Addons **embutidos** vêm dentro do programa e estão sempre disponíveis. Seu manifesto e código-fonte permanecem pristinos — um código sobrescrito pode ser restaurado a qualquer momento;
- Addons **enviados** são seus: um zip com um manifesto e um código-fonte JavaScript, instalado pelo botão `Upload Addon`;
- O emblema `Modified` marca um addon cujo código você sobrescreveu.

Cada cartão mostra o tipo, os emblemas e uma chave de ativação; expandido, revela a descrição, o autor, os campos de configuração e o código-fonte.

![A seção Addons — cartões agrupados com emblemas e chaves de ativação](images/addon-section.jpg)

## Configuração

O manifesto de um addon declara seus campos de configuração, e o cartão expandido os transforma em um formulário. Os valores são armazenados por addon e só surtem efeito enquanto ele estiver ativado; campos de senha permanecem mascarados. Desativar um addon mantém sua configuração — ao reativar, você continua de onde parou.
