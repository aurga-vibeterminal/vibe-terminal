Conectar uma máquina ao relay leva dois campos: o endereço do relay e sua chave. Veja a página anterior, Visão geral, para o que é o recurso e como as peças se encaixam.

## Configurar o servidor relay

1. Abra a seção `Server` na máquina que atuará como relay.
2. Certifique-se de que uma `Secret Key` está definida — o botão de dado gera uma. A mesma chave pareia clientes e servidores compartilhados.
3. Expanda o bloco `QR Code` e escolha a `Network Interface` / `IP Version` que clientes e servidores compartilhados devem usar para alcançá-lo.

## Conectar um servidor compartilhado

Os dados de conexão do relay viajam na área de transferência — nada de redigitar:

1. Na interface Web do servidor relay, clique no `QR code`. Isto copia uma string de compartilhamento criptografada (endereço, porta, chave secreta) para a área de transferência.
2. Abra a interface Web na máquina a ser compartilhada, seção `Server`, e clique no campo `Relay Address`.
3. O navegador exibe um diálogo perguntando se permite que este site acesse a área de transferência — clique em `Allow`:

![Navegador — o diálogo de permissão de acesso à área de transferência mostrado ao focar o campo](images/relay-allow-paste.jpg)

4. Com o acesso concedido, a área de transferência é verificada: quando contém a string de compartilhamento, um botão de colar aparece ao lado do campo — clique nele para descriptografar e preencher automaticamente `Relay Address` e `Relay Secret Key`:

![Interface Web — endereço do relay e chave secreta pré-preenchidos da string colada](images/relay-paste-share.jpg)

5. Revise os valores e clique em `Apply` — salvar reconecta o relay a montante.

![Interface Web — os campos de relay preenchidos com o botão Apply](images/relay-apply-share.jpg)

> Ler a área de transferência exige um contexto seguro (HTTPS ou localhost). Em HTTP simples o botão de colar fica oculto; digite o endereço e a chave do relay à mão.

Por padrão, a conexão a montante é criptografada mas não verificada. Para fixar também o certificado do relay, veja Verificação estrita no relay em Certificados e TLS.

## Cliente de desktop

### Abas de servidores compartilhados

Abas de terminais rodando num servidor compartilhado carregam dois marcadores: uma faixa colorida na borda esquerda da aba — uma cor por servidor — e o ícone do SO remoto (Windows / Apple / Linux) ao lado do ponto de status. Abas locais do próprio servidor relay não têm marcadores.

![Cliente de desktop — marcadores de aba de terminais de servidores compartilhados](images/relay-tab-indicator.jpg)

### Perfis de terminal por servidor

O menu de perfis (barra de título) agrupa perfis de terminal por servidor: primeiro os do próprio servidor relay conectado, depois um grupo recolhível por servidor compartilhado, nomeado pela máquina. Expanda um grupo e escolha um perfil para abrir um terminal naquele servidor. O menu segue o grupo de abas selecionado na barra de título.

![Cliente de desktop — menu de perfis com grupos por servidor](images/relay-profile-menu.jpg)

### Agrupamento e janelas separadas

O seletor de grupos da barra de título lista os grupos de abas de todos os servidores; o grupo padrão dum servidor compartilhado aparece como `Ungrouped (Nome do servidor)`. O submenu de cada grupo lista suas abas para troca rápida e oferece `Move Group to New Window`, que destaca o grupo inteiro para a própria janela — útil para dedicar uma janela ou monitor a uma máquina.

![Cliente de desktop — menu do seletor de grupos com a ação mover para nova janela](images/relay-group-selector.jpg)
