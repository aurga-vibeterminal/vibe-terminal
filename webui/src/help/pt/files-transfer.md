Transferências de arquivos entre sua máquina e o servidor são arrastar e soltar, e a direção é o significado: **arrastar para dentro = enviar ao servidor, arrastar para fora = baixar para esta máquina**. As transferências viajam pela conexão criptografada cliente-servidor; servidores remotos e compartilhados comportam-se como locais.

## Enviando: arrastar para dentro

Arraste arquivos (ou pastas inteiras) do gerenciador de arquivos do seu sistema operacional para a lista de arquivos e solte:

- no modo árvore, **soltar numa linha de pasta** envia para aquela pasta; **soltar na raiz** (a linha de inserção entre linhas) envia para o diretório raiz atual;
- uma linha de transferência aparece no fundo da lista mostrando `Uploading`, o nome do arquivo e uma barra de progresso;
- a conclusão mostra `Uploaded n file(s)`; falhas incluem o motivo (p. ex. permissão negada) e a lista atualiza revelando os novos arquivos.

## Baixando: arrastar para fora

Pressione um arquivo na lista, arraste-o para fora da janela do cliente e solte no gerenciador de arquivos do seu sistema operacional (ou na área de trabalho):

- o arquivo flui do servidor para onde você o soltou;
- linhas de transferência mostram progresso do mesmo jeito; **uma linha de download concluída é clicável** e abre a pasta local de downloads;
- seleção múltipla e arraste vários arquivos de uma vez.

![Progresso da transferência — as linhas de envio/download](images/files-transfer.jpg)

> Arrastar e soltar move arquivos individuais entre esta máquina e um servidor; para mover dados entre dois servidores, combine Publicar com servidor HTTP ou comandos de terminal (scp/rsync).
