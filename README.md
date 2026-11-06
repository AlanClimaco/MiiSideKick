# MiiSideKick

> Assistente de desktop inteligente, leve e expressivo com IA nativa (Google Gemini), automação de rotinas e utilitários de sistema e estética nostálgica Frutiger Aero.

O MiiSideKick é uma aplicação nativa para Windows e Linux (Ubuntu) desenvolvida para operar de forma silenciosa e não-intrusiva na bandeja do sistema (System Tray). Combinando a velocidade e o baixo consumo de recursos do Rust com uma interface desacoplada e elegante, ele automatiza a manutenção do sistema e traz um companheiro de desktop reativo com expressões emocionais dinâmicas e suporte multinacional.

---

## Principais Funcionalidades

### Cérebro com IA, Personalidades e Suporte Multinacional (Google Gemini)
*  **Suporte Bilíngue (PT/EN)**: Chaveador dinâmico no Setup para alternar entre Português (🇧🇷) e Inglês (🇺🇸), adaptando a interface, as respostas da IA e os alertas em segundo plano.
*  **Conversação Nativa no Desktop**: Envie dúvidas e interaja rapidamente com o modelo gemini-3.8-flash por meio de uma janela flutuante acionada pelo menu da bandeja ou pelo atalho global (`Ctrl + Shift + M` / `Cmd + Shift + M`).
*  **Perfil de Gostos e Interesses do Usuário**: Cadastre preferências e hobbies (ex: animes, xadrez, café, programação) que o Mii injeta de maneira sutil e probabilística nas conversas, gerando analogias e puxadas de assunto naturais.
*  **Modos de Resposta Configuráveis**: Escolha na tela de Setup como o Mii deve se comportar:
    *  *Espelho (Padrão)*: Adapta o tom de conversa conforme a interação do usuário, variando entre educado, engraçado e direto (sem xingamentos pesados nem encarnar personagens).
    *  *Educado*: Polido, paciente e formal como um mordomo.
    *  *Engraçado*: Trocadilhos, piadinhas leves e bom humor.
    *  *Sério e Direto*: 100% objetivo, técnico e sem rodeios.
    *  *Hostil*: Agressivo, sem filtro e direto ao ponto quando provocado ou ao responder perguntas óbvias (reclamando do gasto de cota da API, sem usar bordões robóticos de IA e reativo a pegadinhas e trocadilhos).
*  **Uso Moderado do Nome**: A IA utiliza o nome do usuário de forma natural e ocasional, sem repetições artificiais a cada frase.
*  **Fluxo Contínuo de Resposta**: Botão "Responder" integrado que mantém a memória de curto prazo (contexto da conversa) e fecha o balão automaticamente para evitar sobreposição de janelas.
*  **Links e Referências Externas**: Para dúvidas complexas ou quando perguntado sobre sua autoria (Alan Clímaco), o Mii exibe botões discretos com links diretos para documentações e portfólio.
*  **Limite Calibrável de Caracteres**: Escolha entre presets rápidos (Curto: 70, Médio: 140, Longo: 250) ou personalize livremente o teto máximo entre 30 e 500 caracteres.

### Janela Dedicada para o Histórico de Mensagens da IA
*  **Acesso Rápido na Bandeja**: Acione a opção *"💡 Últimas Mensagens da IA"* para abrir uma janela flutuante dedicada (`historico.html`) com visual translúcido Frutiger Aero.
*  **Fila Circular em Memória**: Mantém até 5 mensagens e curiosidades recentes geradas pela IA, ignorando alertas utilitários estáticos do sistema.
*  **Cards com Ações Imediatas**: Cada card exibe o horário de geração, mini avatar com a reação exata do Mii, link clicável (quando presente), botão para copiar o texto com um clique e botão *"Ler no Balão"* para reexibir a notificação no balão do Mii sem fechar a listagem.

### Gerenciador de Clipboard (Histórico da Área de Transferência)
*  **Monitoramento Contínuo**: Captura automaticamente novos textos copiados em segundo plano (`clipboard.html`), ignorando duplicatas imediatas e espaços vazios.
*  **Interface Glassmorphism**: Cards com horário, contagem de caracteres, botão de cópia com feedback instantâneo (*"Copiado! ✓"*) e botão de limpeza rápida de histórico.

### Estética Frutiger Aero & Navegação por Abas (Estilo Windows 7)
*  **Navegação por Abas Horizontais no Setup**: Interface desenhada no formato clássico de folha de propriedades (*Property Sheet*) do Windows 7, eliminando a rolagem vertical e organizando as opções em 4 abas limpas:
    *  *Geral & Perfil*: Idioma (PT/EN), Nome do Usuário, Gostos/Interesses e Tutorial da Bandeja.
    *  *Personalidade & IA*: Chave de API do Gemini (com botão de alternar visibilidade e link oficial), seletor de Personalidades e Tamanho da Resposta.
    *  *Alertas & Sistema*: Alertas Proativos (Manutenção e Bem-Estar), Frequência dos disparos e Inicialização com o Windows.
    *  *Ferramentas & Feedback*: Card dos Atalhos Globais (`Ctrl + Shift + M`) e Formulário de Feedback assíncrono (conectado ao Formspree).
*  **Rodapé Fixo (Sticky Footer)**: Mantém sempre visíveis o botão *"Salvar e Ocultar"*, etiqueta de versão (`v1.6.1`), botão *"Verificar Atualizações"* com checagem assíncrona da release mais recente no GitHub e créditos do desenvolvedor.
*  **Design Nostálgico**: Translucidez vítrea com efeito blur, reflexos luminosos suaves, cantos arredondados em relevo (*squircles*) e ícones vetoriais SVG limpos.
*  **Ancoragem Inteligente**: Balão posicionado automaticamente no canto inferior direito da tela ativa.
*  **Hover Inteligente**: Pausa automática do temporizador de fechamento da notificação enquanto o cursor estiver sobre o balão.

### Reações Visuais Dinâmicas
O avatar do Mii reage em tempo real de acordo com o sentimento da conversa ou o status das tarefas do sistema através de 8 sprites oficiais:
*  *normal*: Respostas cotidianas e status padrão.
*  *feliz*: Elogios, bom humor e faxina concluída.
*  *frustrado*: Decepção, tristeza, aviso de lixeira já vazia ou falhas do sistema.
*  *raiva*: Broncas, irritação, respostas com fúria ou xingamentos.
*  *assustado*: Perguntas chocantes e alerta de fim do timer.
*  *explicando*: Tutoriais e explicações de conceitos técnicos.
*  *bocejo*: Tédio, perguntas repetitivas ou lembretes de descanso.
*  *lambendo*: Deboche triunfante ou referências gastronômicas.

### Motor Proativo de Alertas & Modo Foco (Tela Cheia)
*  **Detecção Nativa de Tela Cheia / Jogos / Apresentações**: Integração com a API nativa do Windows (`SHQueryUserNotificationState`) que identifica jogos em tela cheia, apresentações ou modo foco, enfileirando silenciosamente as notificações para não interromper o usuário.
*  **Tolerância de Saída**: Período de segurança após a saída de tela cheia antes de desenfileirar as notificações pendentes de forma ordenada.
*  **Tempo de Leitura Dinâmico**: O tempo de exibição do balão de notificação é calculado proporcionalmente ao tamanho do texto gerado, com tempo estendido garantido para avisos com botões de ação e reconsultas.
*  **Monitoramento de Manutenção**: Identifica proativamente quando a lixeira está muito cheia ou se há acúmulo de arquivos soltos na pasta Downloads.
*  **Lembretes de Bem-Estar**: Dispara avisos ergonômicos de saúde (água e postura) ao longo do dia de forma 100% local e independente de APIs.
*  **Curiosidades Proativas com IA**: Pílulas de conhecimento disparadas proativamente respeitando a personalidade e gostos configurados, operando em silêncio se a API estiver inacessível (sem disparar mensagens offline repetitivas).
*  **Frequência Customizável**: Ajuste no Setup a frequência das notificações (Discreto, Equilibrado, Frequente).

### Super Organizador de Arquivos (Lazy Creation)
*  Triagem inteligente de arquivos soltos em pastas categorizadas (Documentos, Instaladores, Mídias, Projetos, E-books, Emulação de consoles como PS1, PS2, NDS, 3DS, etc.).
*  Criação sob Demanda: Pastas de destino só são criadas no disco caso de fato exista algum arquivo daquela categoria para ser movido.

### Manutenção e Produtividade
*  **Limpeza de Lixo do PC**: Esvaziamento assíncrono e seguro da lixeira nativa (`SHEmptyRecycleBinW` no Windows e FreeDesktop Trash no Linux).
*  **Otimização de Memória RAM**: Redução do conjunto de páginas de memória (*Working Set*) do sistema via acionamento direto pelo menu da bandeja ou proativamente via balão.
*  **Lembretes e Timer com Alerta Sonoro**: Agendamento de notificações com avisos visuais silenciosos e reprodução do efeito sonoro leve `notification.mp3` no término do tempo.

---

## Tecnologias Utilizadas

*  **Backend**: Rust - Performance extrema, segurança de memória, `sysinfo` para monitoramento de hardware e integração nativa com APIs do Windows/Linux.
*  **Shell e Core**: Tauri v2 - Orquestração de múltiplas janelas, system tray, atalhos globais (`tauri-plugin-global-shortcut`), autostart (`tauri-plugin-autostart`), opener (`tauri-plugin-opener`), menus nativos e comunicação IPC assíncrona.
*  **Frontend**: TypeScript, HTML5 e CSS3 nativo (Vanilla MPA - sem sobrecarga de frameworks).
*  **Inteligência Artificial**: Google Gemini API (Endpoint v1beta / modelo gemini-3.8-flash).
*  **Áudio Otimizado**: HTML5 Audio API integrado via bundling estático do Vite com áudio comprimido leve (`notification.mp3`, ~68 KB).
*  **Empacotador**: Vite - Compilação rápida e gestão otimizada de assets multi-page.

---

## Estrutura do Projeto

```text
MiiSideKick/
├── src-tauri/                 # Backend nativo em Rust
│   ├── src/
│   │   ├── main.rs            # Ciclo de vida, pré-carregamento de janelas e System Tray
│   │   ├── comandos.rs        # Chamadas da API Gemini, otimização de RAM, organizador e I/O
│   │   ├── gatilhos.rs        # Motor proativo de background, monitor de RAM e auto-updater
│   │   ├── clipboard.rs       # Monitoramento contínuo da área de transferência
│   │   └── sistema.rs         # Detecção nativa de tela cheia (modo foco) e fila FIFO
│   ├── capabilities/          # Permissões do Tauri v2 para janelas
│   ├── tauri.conf.json        # Configuração de permissões, janelas e bundles
│   └── Cargo.toml             # Dependências do ecossistema Rust
├── src/                       # Frontend da aplicação
│   ├── assets/                # Recursos visuais e áudio (sprites, mii.png, audio/notification.mp3)
│   ├── css/                   # Folhas de estilo (setup.css, historico.css, clipboard.css, notificacao.css)
│   ├── clipboard/             # Gerenciador de histórico da área de transferência
│   ├── historico/             # Janela flutuante das últimas mensagens geradas pela IA
│   ├── notificacao/           # Balão flutuante de alertas, áudio e fala do avatar
│   ├── pergunta/              # Janela rápida de envio de perguntas ao Mii
│   ├── timer/                 # Janela de configuração de lembretes
│   ├── types.ts               # Tipagens TypeScript compartilhadas
│   ├── index.html             # Janela de Setup com abas horizontais (Windows 7 Property Sheet)
│   ├── clipboard.html         # Janela dedicada do histórico do clipboard
│   ├── historico.html         # Janela dedicada do histórico da IA
│   └── main.ts                # Lógica, navegação por abas e persistência das configurações
└── vite.config.ts             # Configuração de bundling Multi-Page Application (MPA)
```

---

## Como Executar e Compilar

### Pré-requisitos
*  Node.js (v18+) e npm
*  Rust e cargo
*  Dependências de sistema:
    *  Windows: WebView2 Runtime (nativo no Windows 10/11) e C++ Build Tools.
    *  Linux (Ubuntu/Debian): libwebkit2gtk-4.1-dev, build-essential, curl, wget, file, libssl-dev, libgtk-3-dev, libayatana-appindicator3-dev, librsvg2-dev.

### Instalação das Dependências
```bash
npm install
```

### Modo de Desenvolvimento
Inicia a aplicação com hot-reload no frontend e compilação do Rust:
```bash
npm run tauri dev
```

### Compilação de Produção
Gera os instaladores nativos para o sistema operacional em que o comando for executado:
```bash
npm run tauri build
```
*  No Windows: Gera o instalador .exe (NSIS) e .msi em src-tauri/target/release/bundle/.
*  No Linux: Gera os pacotes .deb e .AppImage.

---

### Resolvendo Conflitos de Instalação no Linux
Caso a versão antiga persista após reinstalar o pacote .deb no Ubuntu/Debian:
```bash
killall assistente 2>/dev/null || pkill -f assistente
rm -f ~/.local/bin/assistente /usr/local/bin/assistente
hash -r
assistente
```

---

## Configuração da IA

1. Obtenha uma chave de API gratuita no Google AI Studio (https://aistudio.google.com/app/apikey).
2. Abra as Configurações do MiiSideKick pelo ícone da bandeja.
3. Na aba *Personalidade & IA*, cole o token no campo Chave de API do Gemini e ajuste sua personalidade favorita.
4. Na aba *Geral & Perfil*, informe seu nome e interesses para personalizar as analogias e o tom das conversas.
5. Clique em **Salvar e Ocultar**.
6. Clique no ícone da bandeja ou pressione `Ctrl + Shift + M` para conversar com o Mii a qualquer momento!

---

## Autor

Desenvolvido por Alan Clímaco.
*  Portfólio: https://alanclimaco.github.io/Portfolio/
*  GitHub: https://github.com/AlanClimaco

---

#### Licença
Distribuído sob a licença MIT. Consulte o arquivo `LICENSE` para obter mais informações.
