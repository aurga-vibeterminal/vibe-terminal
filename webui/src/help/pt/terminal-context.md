Agentes de código (Claude Code, Codex, Kimi, Pi, OpenCode…) só sabem o que alcança a entrada deles. `Adicionar contexto` é como entregar caminhos de arquivos e trechos de código sem redigitar: arraste arquivos do `Explorer`, ou arraste texto selecionado do editor de código e da visão de diff do Git, direto para o terminal do agente ou a entrada de chat do Copilot.

## Arrastando arquivos

Pressione um arquivo — ou uma seleção múltipla — na lista de arquivos do `Explorer` e arraste-o:

- **Sobre um terminal**: os caminhos completos são digitados no ponto de soltura, separados por espaço, prontos para virar parte do seu próximo prompt ao agente;
- **Sobre a entrada de chat do Copilot**: os caminhos são inseridos no rascunho da mensagem, um por linha, para você montar um prompt multi-arquivos antes de enviar.

Caminhos arrastados do gerenciador de arquivos do seu sistema operacional e soltos num terminal são automaticamente entre aspas, de modo que nomes com espaços sobrevivem intactos.

![Arrastar um arquivo da lista do Explorer para um terminal digita seu caminho completo](images/terminal-context-drag-file.gif)

## Arrastando código selecionado como contexto

Selecione texto no editor de código ou num diff do Git, depois pressione a própria seleção e arraste-a para um terminal:

- quando o terminal **está rodando um agente de código**, a soltura insere um *bloco de contexto* — o caminho do arquivo, o intervalo de linhas e o trecho embrulhado numa cerca de código:

````text
File: crates/vterm/src/ui/terminal_view.rs
Lines: 558-566
```
fn can_accept_native_drop(&self, info: &DndInfo) -> bool { … }
```
````

- quando o terminal está num **prompt de shell simples**, a soltura digita o texto selecionado bruto, para que o mesmo gesto sirva também para comandos comuns;
- quando o diff é visto num commit específico, o bloco carrega adicionalmente uma linha `Commit:` fixando a revisão exata.

![Arrastar uma seleção do editor para um terminal de agente solta um bloco de contexto File/Lines](images/terminal-context-drag-selection.mp4)

## Copiar contexto

O mesmo bloco vive no menu de contexto: `Copy Context` no editor de código e na visão de diff o copia para a área de transferência. Cole na entrada de chat do Copilot — ou em qualquer outro agente — quando arrastar for inconveniente. A animação acima mostra o bloco resultante na entrada do agente.

## Dicas

- Seleção múltipla no `Explorer` para soltar vários caminhos de uma vez;
- a entrada de chat do Copilot envia direto para o terminal como entrada digitada (veja `Histórico do agente de código`), então contexto adicionado lá alcança o agente exatamente como se você o tivesse digitado;
- combine com `Confirmação automática` para deixar o agente agir sobre o contexto solto sem mais prompts.
