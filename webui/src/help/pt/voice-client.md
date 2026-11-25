Pressione `Ctrl` duas vezes em rápida sucessão (dentro de ~300 ms) em qualquer lugar do cliente de desktop para abrir o diálogo de entrada de voz.

## Gravando

O diálogo começa a gravar imediatamente: a forma de onda líquida segue sua voz, o seletor do topo troca microfones (a escolha é lembrada), e um cronômetro mostra o tempo decorrido. `Stop` encerra o clipe e o envia para transcrição (`Reset` reinicia).

![A fase de gravação — forma de onda, seletor de microfone e cronômetro](images/voice-client-record.jpg)

## Processamento

Uma animação de fluxo de partículas roda enquanto o servidor transcreve o áudio.

## Edição

A transcrição aterra numa caixa de texto editável:

- a pílula de modo alterna entre `original` e `polished`; `Polish` roda o LLM uma vez, e `Auto polish` faz isso logo após cada transcrição;
- `Re-record` descarta o texto e recomeça; `Insert` grava de novo e insere a nova transcrição no texto existente no cursor (substituindo qualquer seleção), para que você estenda ou corrija o já ditado — o cursor fica logo depois do trecho inserido;
- passe o cursor sobre a caixa de texto para revelar um botão de copiar no canto superior direito: um clique copia o texto e ele vira um visto; editar o texto ou trocar de modo o rearma para outra cópia;
- `Submit` (ou `Ctrl+Enter`) digita o texto no terminal ativo; `Esc` fecha o diálogo.

![A fase de edição — pílula de modo, polimento automático, a linha de ações com Insert, e o botão de copiar ao passar o cursor](images/voice-client-editing.jpg)

## Configuração do polimento

`Polish` precisa de um LLM de polimento de voz: na seção `LLM` da interface Web, habilite `Enable voice text polish` na lista por recurso (um provedor comum mais esta alternância é tudo). Sem ele o botão `Polish` permanece acinzentado e a opção de polimento automático fica oculta.
