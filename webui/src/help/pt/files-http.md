`Host with HTTP server` publica qualquer diretório no servidor como serviço de arquivos HTTP com um clique: outros dispositivos na mesma rede (ou pelo relay) — um telefone, outro computador, uma bancada de teste — podem navegar e baixar aquele diretório num navegador. Ideal para compartilhar artefatos de build e pacotes de logs, ou entregar arquivos a dispositivos sem o cliente instalado.

## Como usar

Três entradas, todas num diretório: o menu de contexto da lista de arquivos, o menu de contexto de uma entrada do Recent, e o menu de contexto da trilha de navegação → `Host with HTTP server`.

Ao clicar:

1. o servidor inicia um serviço HTTP estático enraizado naquele diretório, com a porta tirada da faixa de portas Static HTTP;
2. **Servidor local**: a URL `http://127.0.0.1:port/` é copiada direto para a área de transferência, com uma mensagem de status «HTTP server: … (copiado para a área de transferência)»;
3. **Servidor remoto** (conectado por um relay): o cliente automaticamente solicita um mapeamento de proxy reverso e copia a URL de compartilhamento alcançável externamente — restrições de LAN não são obstáculo.

Abra aquela URL no navegador de qualquer dispositivo e o diretório navega e baixa como uma listagem web normal.

## Gerenciar serviços em execução

Cada serviço HTTP continua rodando até ser parado:

- abra a seção `Static HTTP` da interface Web; a lista `Services` mostra cada serviço rodando e sua raiz de documentos;
- cada entrada tem um botão de parar; a faixa de portas é configurada na mesma página.

> O serviço é uma listagem de arquivos somente leitura — sem pontos de escrita; o conteúdo do diretório é visível ao vivo conforme muda, o que serve para logs e artefatos produzidos continuamente.
