# MiiSideKick

> Assistente de desktop inteligente, leve e expressivo com IA nativa (Google Gemini), automação de rotinas e estética nostálgica Frutiger Aero.

O MiiSideKick é uma aplicação nativa para Windows e Linux (Ubuntu) desenvolvida para operar de forma silenciosa e não-intrusiva na bandeja do sistema (System Tray). Combinando a velocidade e o baixo consumo de recursos do Rust com uma interface desacoplada e elegante, ele automatiza a manutenção do sistema e traz um companheiro de desktop reativo com expressões emocionais dinâmicas.

---

## Principais Funcionalidades

### Cérebro com IA e Personalidades (Google Gemini)
* Conversação Nativa no Desktop: Envie dúvidas e interaja rapidamente com o modelo gemini-3.8-flash por meio de uma janela flutuante e minimalista.
* Modos de Resposta Configuráveis: Escolha na tela de Setup como o Mii deve se comportar:
  * Espelho (Padrão): Gentileza com quem é educado; deboche afiado e sarcasmo na mesma moeda com quem tenta zoá-lo.
  * Sarcástico: Irônico, debochado e ácido em todas as respostas.
  * Educado: Polido, paciente e formal como um mordomo.
  * Engraçado: Trocadilhos, piadinhas leves e bom humor.
  * Sério e Direto: 100% objetivo, técnico e sem rodeios.
* Fluxo Contínuo de Resposta: Botão "Responder" integrado que mantém a memória de curto prazo (contexto da conversa) e fecha o balão automaticamente para evitar sobreposição de janelas.
* Links e Referências Externas: Para dúvidas complexas ou quando perguntado sobre sua autoria (Alan Clímaco), o Mii exibe botões discretos com links diretos para documentações e portfólio.
* Limite Calibrável de Caracteres: Escolha entre presets rápidos (Curto: 70, Médio: 140, Longo: 250) ou personalize livremente o teto máximo entre 30 e 500 caracteres.

### Reações Visuais Dinâmicas
O avatar do Mii reage em tempo real de acordo com o sentimento da conversa ou o status das tarefas do sistema através de 7 sprites oficiais:
* normal: Respostas cotidianas e status padrão.
* feliz: Elogios, bom humor e faxina concluída.
* frustrado: Respostas ríspidas a insultos ou avisos de lixeira já vazia.
* assustado: Perguntas chocantes e alerta de fim do timer.
* explicando: Tutoriais e explicações de conceitos técnicos.
* bocejo: Tédio, perguntas repetitivas ou lembretes de descanso.
* lambendo: Deboche triunfante ou referências gastronômicas.

### Estética Frutiger Aero (Windows 7 Glassmorphism)
* Interface nostálgica com translucidez vítrea, reflexos luminosos suaves (glass glare), cantos arredondados em relevo (squircles) e ícones vetoriais SVG limpos (sem emojis).
* Pausa automática do temporizador do balão de fala ao passar o mouse por cima (hover).

### Super Organizador de Arquivos (Lazy Creation)
* Triagem inteligente de arquivos soltos em pastas categorizadas (Documentos, Instaladores, Mídias, Projetos, E-books, Emulação de consoles como PS1, PS2, NDS, 3DS, etc.).
* Criação sob Demanda: Pastas de destino só são criadas no disco caso de fato exista algum arquivo daquela categoria para ser movido.

### Manutenção e Produtividade
* Limpeza de Lixo do PC: Esvaziamento assíncrono e seguro da lixeira nativa (SHEmptyRecycleBinW no Windows e FreeDesktop Trash no Linux).
* Lembretes e Timer: Agendamento rápido de notificações com foco não-intrusivo.

---

## Tecnologias Utilizadas

* Backend: Rust - Performance extrema, segurança de memória e integração nativa com o sistema operacional.
* Shell e Core: Tauri v2 - Orquestração de janelas, system tray, menus nativos e comunicação IPC assíncrona.
* Frontend: TypeScript, HTML5 e CSS3 nativo (Vanilla MPA - sem sobrecarga de frameworks).
* Inteligência Artificial: Google Gemini API (Endpoint v1beta / modelo gemini-3.8-flash).
* Empacotador: Vite - Compilação rápida e gestão otimizada de assets.

---

## Estrutura do Projeto

```text
MiiSideKick/
├── src-tauri/                 # Backend nativo em Rust
│   ├── src/
│   │   ├── main.rs            # Ciclo de vida, pré-carregamento de janelas e System Tray
│   │   └── comandos.rs        # Chamadas da API Gemini, organizador, limpeza e I/O
│   ├── tauri.conf.json        # Configuração de permissões, janelas e bundles
│   └── Cargo.toml             # Dependências do ecossistema Rust
├── src/                       # Frontend da aplicação
│   ├── assets/                # Recursos visuais (sprites de reações, mii.png e tutoriais)
│   ├── css/                   # Folhas de estilo (setup.css, notificacao.css, global.css)
│   ├── notificacao/           # Balão flutuante de alertas e fala do avatar
│   ├── pergunta/              # Janela rápida de envio de perguntas ao Mii
│   ├── timer/                 # Janela de configuração de lembretes
│   ├── index.html             # Janela de Setup e Onboarding (Frutiger Aero)
│   └── main.ts                # Lógica e persistência da tela de configurações
└── vite.config.ts             # Configuração de bundling Multi-Page Application (MPA)
```

---

## Como Executar e Compilar

### Pré-requisitos
* Node.js (v18+) e npm
* Rust e cargo
* Dependências de sistema:
  * Windows: WebView2 Runtime (nativo no Windows 10/11) e C++ Build Tools.
  * Linux (Ubuntu/Debian): libwebkit2gtk-4.1-dev, build-essential, curl, wget, file, libssl-dev, libgtk-3-dev, libayatana-appindicator3-dev, librsvg2-dev.

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
* No Windows: Gera o instalador .exe (NSIS) e .msi em src-tauri/target/release/bundle/.
* No Linux: Gera os pacotes .deb e .AppImage.

---

## Configuração da IA

1. Obtenha uma chave de API gratuita no Google AI Studio (https://aistudio.google.com/app/apikey).
2. Abra as Configurações do MiiSideKick pelo ícone da bandeja.
3. Cole o token no campo Chave de API do Gemini, ajuste a sua personalidade favorita e clique em Salvar e Ocultar.
4. Clique com o botão direito na bandeja e escolha Conversar com o Mii.

---

## Autor

Desenvolvido por Alan Clímaco.
* Portfólio: https://alanclimaco.github.io/Portfolio/
* GitHub: https://github.com/AlanClimaco

---

## Licença

Este projeto é distribuído sob a licença MIT. Consulte o arquivo LICENSE para mais informações.
