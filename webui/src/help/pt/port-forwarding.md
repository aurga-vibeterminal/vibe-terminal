O encaminhamento de portas publica um serviço HTTP que roda na máquina do servidor — um servidor de desenvolvimento, um painel de dispositivo, qualquer coisa ouvindo em um endereço simples `http://localhost:port` — em sua própria porta TLS, de forma que qualquer dispositivo que alcance o servidor o abre num navegador como `https://<endereço-do-servidor>:<porta>/`.

## Como funciona

- Um encaminhamento mapeia uma **porta de destino** — o serviço HTTP nesta máquina — para uma **porta encaminhada** alocada da faixa de portas configurada (padrão `62000`–`62999`);
- A porta encaminhada fala TLS: todo o tráfego é criptografado com o **certificado autoassinado** do servidor — nada trafega em texto claro, mesmo por uma rede não confiável;
- Qualquer dispositivo que alcance o servidor — na LAN, ou pela internet uma vez que a faixa de portas esteja encaminhada no roteador — abre `https://<endereço-do-servidor>:<portaEncaminhada>/` e cai no serviço local.

Os encaminhamentos são criados sob demanda: o compartilhamento de arquivos `Host with HTTP server` num servidor remoto (via relay) solicita um automaticamente e entrega a URL pronta para abrir. Nenhuma configuração manual é necessária.

## Gerenciar encaminhamentos

![Interface Web — a seção Port Forwarding: configuração da faixa de portas e a lista de encaminhamentos ativos](images/port-forwarding-webui.jpg)

1. Abra a seção `Port Forwarding` da interface Web do servidor.
2. O `Port Range Configuration` delimita o conjunto do qual as portas encaminhadas são alocadas; as alterações são salvas automaticamente. Estreite a faixa quando o firewall ou roteador só encaminha poucas portas.
3. A lista ao vivo mostra cada encaminhamento como `:portaEncaminhada → :portaDestino` com sua contagem de conexões ativas.
4. O botão de parar de um encaminhamento derruba o ouvinte imediatamente e desconecta suas conexões ativas; o próximo compartilhamento aloca uma porta nova. O `Refresh` relê a lista.

## O aviso de certificado

Os encaminhamentos são criptografados com o próprio certificado autoassinado do servidor — o mesmo que protege a interface Web e as conexões dos clientes. Os navegadores não conseguem verificá-lo contra uma autoridade certificadora, então a primeira visita a uma URL encaminhada mostra um aviso de privacidade (*Sua conexão não é privada*). Isso é esperado e seguro de ignorar: a criptografia em si não está comprometida, apenas a origem do certificado não pode ser verificada por máquina. Para silenciar o aviso permanentemente, importe o certificado do servidor para o armazenamento de confiança do seu dispositivo.

![Navegador — o aviso de certificado autoassinado não confiável; prossiga por Advanced → Proceed para abrir a página](images/port-forwarding-cert-warning.jpg)

## O Chrome no macOS mostra `ERR_ADDRESS_UNREACHABLE`

No macOS 15 (Sequoia) e posteriores, apps de terceiros precisam de permissão explícita para alcançar dispositivos na rede local — qualquer endereço `192.168.x.x`, `10.x.x.x` ou semelhante. O Safari é isento por ser um app do sistema, então uma URL encaminhada pode abrir normalmente no Safari enquanto o Chrome reporta `ERR_ADDRESS_UNREACHABLE` antes mesmo de o TLS começar. Isso não é um problema de certificado nem um problema com o encaminhamento em si.

Para conceder acesso ao Chrome:

1. Abra `Ajustes do Sistema` → `Privacidade e Segurança` → `Rede Local`.
2. Encontre o `Google Chrome` na lista e **ative** o interruptor.
3. Encerre o Chrome por completo (`Cmd+Q`, não apenas fechando janelas) e abra-o de novo.

Se o interruptor já estiver ativado, desative-o e ative-o de novo — ele pode travar após uma atualização do Chrome ou do macOS. Se o erro persistir, verifique se há uma extensão de VPN ou proxy no Chrome que não roteia faixas de endereços privados.

## Controle de acesso

O que protege uma porta encaminhada depende do que está atrás dela:

- **Os compartilhamentos integrados exigem token.** Os links de compartilhamento gerados pelo cliente — um compartilhamento `Host with HTTP server` (`?share_token=…`) e a página de Settings (`?ticket=…`) — incluem um token de acesso. Uma conexão sem token válido e sem sessão válida é derrubada antes de qualquer resposta. Na primeira visita, o navegador o troca por um cookie de sessão seguro exclusivo daquele serviço e o token some da URL — não fica na barra de endereços nem no histórico.

- **Os tokens têm uso único e vida curta.** Um ticket de Settings funciona exatamente uma vez e expira em um minuto; quando um compartilhamento para ou reinicia, os links e sessões anteriores deixam de funcionar. Trate o link completo como uma credencial — quem o tiver pode acessar o serviço — e compartilhe apenas com pessoas de confiança.

- **Seus próprios serviços levam a própria autenticação.** Um encaminhamento para qualquer outro serviço local criptografa o tráfego com o certificado autoassinado do servidor, mas não adiciona autenticação: tudo o que o serviço servir — páginas e APIs — fica acessível a todos que alcançarem a porta. Coloque autenticação em serviços sensíveis antes de expô-los.
