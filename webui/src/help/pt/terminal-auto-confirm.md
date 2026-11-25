O analisador de terminal do lado servidor vigia a saída do terminal: detecta prompts de comandos, reconhece a conclusão de tarefas e, com um LLM, classifica o que o terminal está mostrando. `Auto Confirm` é seu recurso mais usado — quando o terminal para num prompt de confirmação comum (como `Proceed? [Y/n]`), Enter é enviado por você, de modo que tarefas longas não mais travam em perguntas.

## Configuração

A seção `Terminal Analysis` da interface Web:

- `Terminal observer` — a chave principal da análise do lado servidor;
- `Auto confirm` — envia Enter apenas para prompts classificados como **confirmações comuns**; outras perguntas nunca são respondidas às cegas;
- `Analysis delay (seconds)` — quanto esperar após um agente ficar ocioso antes de analisar, evitando chamadas LLM redundantes entre tarefas;
- `Confirmation grace (seconds)` — a janela durante a qual você pode assumir o controle após uma confirmação ser detectada;
- `Notify on completion` — emite um evento quando a saída é classificada como tarefa concluída (alimente notificações com isso).

> A nota da página: quando o modo automático do seu próprio agente está ligado, prefira desligar o `Auto Confirm` no cliente para que as duas automações não respondam uma sobre a outra.

## Dependência de LLM

A classificação é feita por um LLM: escolha um `Provider` e preencha suas credenciais na seção `LLM` (compartilhada pela classificação de terminal, mensagens de commit do Git, o assistente de chat e outros recursos de IA). Sem LLM o analisador ainda funciona com precisão reduzida — mantenha a confirmação automática desligada nesse caso.

![A página de ajustes Terminal Analysis](images/terminal-auto-confirm.jpg)

![A página de ajustes LLM — escolha um provedor e preencha credenciais](images/terminal-auto-confirm-llm.jpg)
