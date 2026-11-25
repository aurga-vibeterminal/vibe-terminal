O relay resolve um problema: a maioria das máquinas fica atrás de NAT ou IPs dinâmicos, então clientes não podem discar para cada uma diretamente. Deixe uma máquina publicamente alcançável agir como ponto de entrada do relay; todas as outras discam para ela. Um cliente mantém então uma única conexão ao relay e alcança por ele os terminais de cada servidor compartilhado.

## Arquitetura

![Arquitetura do relay — cada cliente mantém uma conexão ao relay; servidores compartilhados discam ao relay a montante](diagram/relay)

- **Servidor relay** — o ponto de entrada público ao qual clientes e servidores compartilhados se conectam. Deixe seu `Relay Address` vazio.
- **Servidor compartilhado** — uma máquina cujo `Relay Address` aponta ao relay. Ele discam a montante e compartilha seus terminais por ele. A faixa colorida em cada nó compartilhado no diagrama espelha a faixa que o cliente de desktop pinta nas abas daquele servidor — uma cor por servidor.
- Conexões sempre se originam de dentro: servidores compartilhados discam ao relay, então nenhum encaminhamento de porta ou IP pública é necessário neles.

## O que o cliente vê

Conectado ao relay, cada servidor compartilhado se comporta como uma máquina local:

- abas de terminal carregam uma faixa colorida e o ícone do SO remoto, identificando a máquina num relance (veja o Guia de configuração);
- o menu de perfis agrupa terminais por servidor — abra um terminal em qualquer máquina diretamente;
- grupos de abas podem ser destacados para a própria janela, dedicando uma janela a uma máquina.

## Modelo de segurança

Clientes e servidores compartilhados pareiam com a mesma chave secreta; conexões são tuneladas através do relay. A chave viaja no QR code de pareamento / string de compartilhamento, nunca em texto plano.

Instruções passo a passo estão na próxima página, Guia de configuração.
