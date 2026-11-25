A entrada de voz transforma fala em entrada de terminal: fale em vez de digitar, com polimento LLM opcional da transcrição antes do envio. O reconhecimento roda como servidor ASR local com modelos no dispositivo — áudio e texto ficam na sua máquina. Esta página cobre o lado do servidor; a experiência de desktop vive em Usar o cliente.

## O cartão Local ASR Server

Abra a seção `Voice` da interface Web e habilite o reconhecimento de voz. O cartão `Local ASR Server` gerencia o processo vterm-voice supervisionado:

- `Load on Startup` — iniciar o servidor ASR junto com o backend;
- `Bind Address` (`127.0.0.1` somente localhost / `0.0.0.0` todas as interfaces) e `Port` (padrão `8398`) — onde o endpoint ASR escuta;
- `Model Directory` + `Local Model` — escolha do catálogo embutido: famílias Qwen3-ASR, Fun-ASR e Parakeet, cada uma em variantes full / 8 bits / 4 bits. `Download` traz o modelo do Hugging Face ou ModelScope para o diretório com uma barra de progresso ao vivo; comandos `git clone` manuais são mostrados como alternativa, e modelos já no disco são marcados disponíveis;
- `Device` — o backend de inferência (`Auto`, `CPU`, `CUDA`, `Vulkan`, `Metal`), conforme reportado pelo servidor;
- `Access Token` — segredo compartilhado do endpoint; o botão de dado gera um;
- `Save` persiste o cartão (edições ficam um rascunho local até então), e `Start Server` / `Stop Server` controlam o processo. O ícone de status no título do cartão reflete alcançabilidade e estado do modelo (carregado / carregando / erro).

![A seção Voice — o cartão Local ASR Server com modelo, dispositivo e token](images/voice-server-local-asr.jpg)

> **Disputa CUDA**: quando outro programa de inferência CUDA já ocupa a GPU (p. ex. llama.cpp rodando um modelo local), deixar o vterm-voice usar `CUDA` também pode desacelerar drasticamente ambos. Nesse caso ponha `Device` em `CPU` — esses modelos ASR compactos são rápidos o bastante em CPU.

## Provedor e testes

O menu suspenso de provedores lista addons ASR — o addon `vibe-voice` embutido aponta para este servidor supervisionado, então escolhê-lo é como o painel usa seus modelos locais.

A área `Test` experimenta tudo sem cliente: `Record` (microfone → forma de onda → transcrição) e `Live` (reconhecimento em fluxo), cada um com tempos e botão de copiar.

![A área Test — as abas Record e Live](images/voice-server-test.jpg)
