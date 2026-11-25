A interface Web é a página de configuração do próprio servidor: cada recurso do servidor está organizado em uma seção, listada na barra lateral esquerda. Ela não é um terminal — o trabalho do dia a dia acontece no cliente de desktop ou no `vtermc` (veja Visão geral). Esta página explica a disposição e o comportamento comuns; os capítulos de cada recurso cobrem as configurações de cada seção em detalhes.

## Disposição

- **Barra lateral** — a lista de seções (abaixo), o ícone de ajuda ao lado do nome do servidor e a versão do app no rodapé; em telas estreitas ela desliza a partir da esquerda e o botão hambúrguer do cabeçalho a abre e fecha;
- **Cabeçalho** — o título da seção atual mais, no canto superior direito, o seletor de idioma e o alternador de tema claro/escuro (em telas estreitas o ícone de ajuda também fica ali);
- **Conteúdo** — o formulário da seção selecionada, centralizado e com rolagem.

## Seções

Em ordem na barra lateral:

- `Servidor` — porta, certificado TLS e configurações de autenticação: a `Secret Key`, o `QR Code` de pareamento, o endereço do relay e o alternador de `Auto-start on Login` ficam todos aqui (veja Conectar um cliente e Guia de configuração). No Linux, a dica do `Auto-start on Login` também explica o login automático do gerenciador de display para alcançar a área de trabalho antes de um login local (veja Configuração do servidor em Área de trabalho remota e simuladores);
- `Terminal` — análise de terminal: detecção de prompt, confirmação automática e classificação por LLM (veja Confirmação automática);
- `UART` — perfis de terminal serial baseados em portas UART locais (veja Serial UART);
- `Complementos` — complementos instalados: ative, edite ou envie-os (provedores de LLM são complementos — veja LLM);
- `LLM` — atribui um provedor de LLM a cada recurso de IA (veja LLM);
- `Voz` — provedor de reconhecimento de fala, modelo e configurações de transcrição (veja Configuração do servidor em Entrada de voz);
- `Ambiente` — coleções de variáveis de ambiente em uma árvore de grupos (veja Variáveis de ambiente);
- `MCP` — o servidor MCP de terminal exposto a agentes de código (veja Terminal MCP);
- `DDNS` — provedor DDNS e configurações de sincronização (veja DDNS);
- `Encaminhamento de portas` — o conjunto de portas encaminhadas e os encaminhamentos TLS ativos (veja Encaminhamento de portas);
- `Static HTTP` — pastas hospedadas por HTTP e os compartilhamentos de arquivos ativos (veja Publicar com servidor HTTP);
- `Área de trabalho remota` — configurações de captura, entrada e transmissão da área de trabalho remota (veja Configuração do servidor em Área de trabalho remota e simuladores);
- `Estatísticas de LLM` — uso de tokens, contagens de solicitações e métricas de desempenho em todos os recursos de LLM.

## Salvamento e dicas

- A maioria das seções salva automaticamente pouco depois de uma edição — um indicador de salvamento e qualquer erro aparecem na parte inferior da seção;
- `MCP`, `DDNS` e `Área de trabalho remota` retêm as edições até que `Salvar alterações` seja pressionado: enquanto houver edições pendentes, uma dica de `Alterações não salvas` aparece ao lado dos botões, e `Redefinir` as descarta;
- Aberta pelo cliente móvel, a interface Web segue o idioma e o tema do app e oculta seus próprios controles de idioma e tema.
