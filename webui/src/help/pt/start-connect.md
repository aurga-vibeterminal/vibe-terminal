Conecte um cliente a este servidor com o QR code de pareamento da interface Web — ou digitando os dados à mão. Tudo o que um cliente precisa (endereço, porta, chave secreta) é mostrado na seção Server.

## Preparar o servidor

- Abra a seção `Server` e certifique-se de que uma `Secret Key` está definida; clique no botão de dado para gerar uma. Regenerar a chave invalida todos os clientes pareados, que precisam parear de novo.
- Expanda o bloco `QR Code` e escolha a `Network Interface` e a `IP Version` (IPv4 / IPv6) que as outras máquinas devem usar para alcançar este servidor.
- Se um domínio DDNS estiver configurado, você pode mudar `Connection Type` para `DDNS Domain` para que o QR code incorpore o domínio em vez de um endereço IP.
- A `Pairing Preview` mostra a URL de conexão exata (`wss://host:port`) que o cliente discará.
- Clicar no `QR code` copia uma string de compartilhamento criptografada para a área de transferência — os clientes a usam para preenchimento automático com um toque.

## Cliente de desktop

- Clique no botão do nome do servidor na barra de título e escolha `Servers` no menu para abrir o diálogo Server Manager.

![Cliente de desktop — o menu de servidor aberto ao clicar no botão do nome do servidor](images/start-connect-server-menu.jpg)

- Clique no botão `+` ao lado da lista de servidores para iniciar o formulário Add Server.
- Se a string de compartilhamento está na área de transferência, um ícone de área de transferência aparece no canto superior direito do formulário — clique nele para descriptografar a string e preencher automaticamente `Title`, `Address` e `Secret Key`.
- Caso contrário, preencha os campos à mão: um nome de exibição, a URL de conexão (`wss://host:port`, endereços IPv6 entre colchetes) e a chave secreta.
- Clique em `Save`, depois escolha o novo servidor para abrir uma sessão.

![Cliente de desktop — Server Manager com o formulário Add Server](images/start-connect-add-server.jpg)
