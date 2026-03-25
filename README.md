# Assistente Pessoal Desktop

Repositório oficial do Assistente Pessoal Desktop. Aplicação nativa para Windows desenvolvida para operar silenciosamente na bandeja do sistema (System Tray), oferecendo automação de rotinas, limpeza de sistema e gestão de lembretes.

Projeto construído com foco em performance e baixo consumo de memória, utilizando arquitetura Multi-Page Application (MPA) com janelas pré-carregadas para evitar deadlocks no sistema operacional.

## Tecnologias

- **Rust** (Backend, processamento em background e integração com OS)
- **TypeScript**, **HTML** e **CSS** (Frontend Vanilla, sem frameworks pesados)
- **Tauri v2** (Framework para construção do binário nativo)
- **Vite** (Build tool e empacotador de assets)

## Estrutura e Componentes

Abaixo está listada a estrutura principal do projeto. Clique no nome do diretório ou arquivo para acessar diretamente o código-fonte correspondente.

- [src-tauri](./src-tauri/) — Diretório principal do backend em Rust.
  - [src/main.rs](./src-tauri/src/main.rs) — Ponto de entrada da aplicação. Configura o System Tray, gerencia o ciclo de vida e faz o pre-loading das instâncias do WebView2.
  - [src/comandos.rs](./src-tauri/src/comandos.rs) — Lógica de negócios. Contém as rotinas de I/O, limpeza da lixeira (`shell32`), movimentação de arquivos e controle de visibilidade das janelas.
  - [tauri.conf.json](./src-tauri/tauri.conf.json) — Arquivo de configuração de permissões, dimensões de janelas e regras de compilação.
- [src](./src/) — Diretório do frontend em TypeScript e HTML.
  - [assets](./src/assets/) — Recursos estáticos.
    - [reacoes](./src/assets/reacoes/) — Sprites do avatar dinâmico utilizados nas notificações.
    - [tutoriais](./src/assets/tutoriais/) — Imagens de onboarding do usuário.
  - [notificacao](./src/notificacao/) — Interface dos alertas do sistema.
    - [notificacao.html](./src/notificacao/notificacao.html) — Estrutura base da notificação flutuante com suporte a transparência (alpha channel).
    - [notificacao.ts](./src/notificacao/notificacao.ts) — Script receptor dos eventos globais do Rust e atualizador do DOM.
  - [timer](./src/timer/) — Interface de criação de lembretes.
    - [timer.html](./src/timer/timer.html) — Layout do formulário.
    - [timer.ts](./src/timer/timer.ts) — Validação de inputs e disparo de comandos assíncronos para o backend.
  - [index.html](./index.html) — Janela principal de configurações e onboarding.
  - [main.ts](./src/main.ts) — Lógica de interação da tela de configurações.
  - [styles.css](./src/styles.css) — Estilização global, incluindo o design system baseado em Glassmorphism.
- [vite.config.ts](./vite.config.ts) — Configuração de roteamento (Rollup Options) para compilação das múltiplas janelas HTML.