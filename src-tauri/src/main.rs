#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod comandos;
mod gatilhos;
mod sistema;
mod clipboard;
mod organizador;
mod avatar;
mod captura;
mod busca;

use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Manager, WebviewUrl, WebviewWindowBuilder,
};

pub fn construir_menu_bandeja(app: &tauri::AppHandle) -> Result<Menu<tauri::Wry>, tauri::Error> {
    let cfg = comandos::carregar_configuracoes();
    let is_en = cfg.idioma == "en";
    let visiveis = &cfg.tray_itens_visiveis;

    let show_i = MenuItem::with_id(
        app,
        "show",
        if is_en { "Open Setup" } else { "Abrir Setup" },
        true,
        None::<&str>,
    )?;

    let mut items: Vec<Box<dyn tauri::menu::IsMenuItem<tauri::Wry>>> = Vec::new();
    items.push(Box::new(show_i));

    if visiveis.contains(&"pergunta".to_string()) {
        let pergunta_i = MenuItem::with_id(
            app,
            "pergunta",
            if is_en { "Chat with Mii" } else { "Conversar com o Mii" },
            true,
            None::<&str>,
        )?;
        items.push(Box::new(pergunta_i));
    }

    if visiveis.contains(&"historico".to_string()) {
        let historico_i = MenuItem::with_id(
            app,
            "historico",
            if is_en { "Recent AI Messages" } else { "Últimas Mensagens da IA" },
            true,
            None::<&str>,
        )?;
        items.push(Box::new(historico_i));
    }

    if visiveis.contains(&"clipboard".to_string()) {
        let clipboard_i = MenuItem::with_id(
            app,
            "clipboard",
            if is_en { "Clipboard History" } else { "Histórico de Copiar" },
            true,
            None::<&str>,
        )?;
        items.push(Box::new(clipboard_i));
    }

    if visiveis.contains(&"timer".to_string()) {
        let timer_i = MenuItem::with_id(
            app,
            "timer",
            if is_en { "New Reminder" } else { "Novo Lembrete" },
            true,
            None::<&str>,
        )?;
        items.push(Box::new(timer_i));
    }

    if visiveis.contains(&"organiza".to_string()) {
        let organiza_i = MenuItem::with_id(
            app,
            "organiza",
            if is_en { "Organize Files" } else { "Organizar Arquivos" },
            true,
            None::<&str>,
        )?;
        items.push(Box::new(organiza_i));
    }

    if visiveis.contains(&"faxina".to_string()) {
        let faxina_i = MenuItem::with_id(
            app,
            "faxina",
            if is_en { "Clean Junk Files" } else { "Limpar Lixo do PC" },
            true,
            None::<&str>,
        )?;
        items.push(Box::new(faxina_i));
    }

    if visiveis.contains(&"otimizar_ram".to_string()) {
        let otimizar_ram_i = MenuItem::with_id(
            app,
            "otimizar_ram",
            if is_en { "Optimize RAM" } else { "Otimizar RAM" },
            true,
            None::<&str>,
        )?;
        items.push(Box::new(otimizar_ram_i));
    }

    if visiveis.contains(&"busca".to_string()) {
        let busca_i = MenuItem::with_id(
            app,
            "busca",
            if is_en { "Search Files" } else { "Buscar Arquivos" },
            true,
            None::<&str>,
        )?;
        items.push(Box::new(busca_i));
    }

    let quit_i = MenuItem::with_id(
        app,
        "quit",
        if is_en { "Quit" } else { "Sair" },
        true,
        None::<&str>,
    )?;
    items.push(Box::new(quit_i));

    let refs: Vec<&dyn tauri::menu::IsMenuItem<tauri::Wry>> = items.iter().map(|i| i.as_ref()).collect();
    Menu::with_items(app, &refs)
}

pub fn atualizar_menu_bandeja(app: &tauri::AppHandle) {
    if let Some(tray) = app.tray_by_id("main-tray") {
        if let Ok(menu) = construir_menu_bandeja(app) {
            let _ = tray.set_menu(Some(menu));
        }
    }
}

pub fn atualizar_atalhos_globais(app: &tauri::AppHandle, config: &comandos::AppConfig) {
    use tauri_plugin_global_shortcut::GlobalShortcutExt;

    // Desregistra todos os atalhos antes de registrar os ativos
    let _ = app.global_shortcut().unregister_all();

    if config.atalho_radial {
        let _ = app.global_shortcut().register("ctrl+shift+space");
    }
    if config.atalho_chat {
        let _ = app.global_shortcut().register("ctrl+shift+m");
    }
    if config.atalho_busca {
        let _ = app.global_shortcut().register("ctrl+shift+f");
    }
    if config.atalho_clipboard {
        let _ = app.global_shortcut().register("ctrl+shift+v");
    }
    if config.atalho_olhar_tela {
        let _ = app.global_shortcut().register("ctrl+shift+s");
    }
}

fn main() {
    #[cfg(target_os = "linux")]
    {
        // Previne crash de EGL_NOT_INITIALIZED com WebKitGTK no Linux
        std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--autostart"]),
        ))
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, shortcut, event| {
                    if event.state() == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                        let sc_str = shortcut.to_string().to_lowercase();
                        if sc_str.contains("space") {
                            comandos::abrir_radial(app.clone());
                        } else if sc_str.contains("keym") {
                            comandos::limpar_historico_conversa();
                            if let Some(window) = app.get_webview_window("pergunta") {
                                let _ = window.show();
                                let _ = window.unminimize();
                                let _ = window.set_focus();
                            }
                        } else if sc_str.contains("keyf") {
                            busca::abrir_busca(app.clone());
                        } else if sc_str.contains("keyv") {
                            clipboard::abrir_clipboard(app.clone());
                        } else if sc_str.contains("keys") {
                            comandos::analisar_tela_com_ia(app.clone(), None);
                        }
                    }
                })
                .build(),
        )
        .invoke_handler(tauri::generate_handler![
            comandos::get_system_info,
            comandos::esconder_janela,
            comandos::esconder_popup,
            comandos::esconder_timer,
            comandos::abrir_timer,
            comandos::abrir_configuracoes,
            comandos::esconder_pergunta,
            comandos::esconder_radial,
            comandos::abrir_radial,
            comandos::limpar_lixo_pc,
            comandos::pegar_dados_notificacao,
            comandos::organizar_downloads,
            comandos::iniciar_timer,
            comandos::carregar_configuracoes,
            comandos::salvar_configuracoes,
            comandos::perguntar_ao_mii,
            comandos::abrir_pergunta,
            comandos::limpar_historico_conversa,
            comandos::limpar_cache_ram,
            clipboard::obter_historico_clipboard,
            clipboard::copiar_item_clipboard,
            clipboard::limpar_historico_clipboard,
            clipboard::abrir_clipboard,
            clipboard::esconder_clipboard,
            comandos::obter_historico_ia,
            comandos::limpar_historico_ia,
            comandos::reexibir_notificacao_ia,
            comandos::abrir_historico,
            comandos::esconder_historico,
            comandos::verificar_atualizacoes_manual,
            organizador::obter_regras_organizador,
            organizador::salvar_regras_organizador,
            organizador::restaurar_regras_padrao,
            avatar::obter_mapa_avatares,
            avatar::obter_status_avatar_customizado,
            avatar::abrir_pasta_avatar_customizado,
            comandos::analisar_tela_com_ia,
            comandos::copiar_detalhes_ia,
            busca::procurar_arquivos_locais,
            busca::abrir_pasta_com_arquivo,
            busca::abrir_busca,
            busca::abrir_busca_com_termo,
            busca::obter_termo_pre_pesquisa,
            busca::esconder_busca
        ])
        .setup(|app| {
            // Inicializa pasta e README de avatar customizado
            avatar::inicializar_diretorio_avatar();

            // Inicializa o motor de gatilhos proativos e checagem de updates
            gatilhos::iniciar_motor_proativo(app.handle().clone());

            // Inicializa o monitoramento contínuo do clipboard
            clipboard::iniciar_monitor_clipboard();

            // Registra os atalhos globais ativos do usuário
            let cfg = comandos::carregar_configuracoes();
            atualizar_atalhos_globais(&app.handle(), &cfg);

            // JANELA POPUP (NOTIFICAÇÕES)

            #[cfg(debug_assertions)]
            let popup_url = WebviewUrl::External(
                tauri::Url::parse("http://localhost:1420/notificacao.html").unwrap(),
            );
            #[cfg(not(debug_assertions))]
            let popup_url = WebviewUrl::App("notificacao.html".into());

            let popup_window = WebviewWindowBuilder::new(app, "popup", popup_url)
                .inner_size(430.0, 360.0)
                .decorations(false)
                .transparent(true)
                .resizable(false)
                .shadow(false)
                .always_on_top(true)
                .skip_taskbar(true)
                .visible(false)
                .build()?;

            #[cfg(debug_assertions)]
            popup_window.open_devtools();

            if let Ok(Some(monitor)) = popup_window.primary_monitor() {
                let size = monitor.size();
                let window_size = popup_window
                    .outer_size()
                    .unwrap_or(tauri::PhysicalSize::new(430, 360));
                let x = size.width.saturating_sub(window_size.width + 20);
                let y = size.height.saturating_sub(window_size.height + 60);
                let _ = popup_window.set_position(tauri::PhysicalPosition::new(x, y));
            }

            // JANELA TIMER (FORMULÁRIO DE LEMBRETE)

            #[cfg(debug_assertions)]
            let timer_url = WebviewUrl::External(
                tauri::Url::parse("http://localhost:1420/timer.html").unwrap(),
            );
            #[cfg(not(debug_assertions))]
            let timer_url = WebviewUrl::App("timer.html".into());

            let timer_window = WebviewWindowBuilder::new(
                app, "timer", // ID da janela do timer
                timer_url,
            )
            .inner_size(360.0, 420.0)
            .decorations(false)
            .transparent(true)
            .resizable(false)
            .shadow(false)
            .always_on_top(true)
            .skip_taskbar(true)
            .visible(false)
            .build()?;

            if let Ok(Some(monitor)) = timer_window.primary_monitor() {
                let size = monitor.size();
                let window_size = timer_window
                    .outer_size()
                    .unwrap_or(tauri::PhysicalSize::new(360, 420));
                let x = size.width.saturating_sub(window_size.width + 20);
                let y = size.height.saturating_sub(window_size.height + 60);
                let _ = timer_window.set_position(tauri::PhysicalPosition::new(x, y));
            }

            // JANELA PERGUNTA (CONVERSAR COM O MII)

            #[cfg(debug_assertions)]
            let pergunta_url = WebviewUrl::External(
                tauri::Url::parse("http://localhost:1420/pergunta.html").unwrap(),
            );
            #[cfg(not(debug_assertions))]
            let pergunta_url = WebviewUrl::App("pergunta.html".into());

            let pergunta_window = WebviewWindowBuilder::new(
                app, "pergunta", // ID da janela de pergunta
                pergunta_url,
            )
            .inner_size(360.0, 180.0)
            .decorations(false)
            .transparent(true)
            .resizable(false)
            .shadow(false)
            .always_on_top(true)
            .skip_taskbar(true)
            .visible(false)
            .build()?;

            if let Ok(Some(monitor)) = pergunta_window.primary_monitor() {
                let size = monitor.size();
                let window_size = pergunta_window
                    .outer_size()
                    .unwrap_or(tauri::PhysicalSize::new(360, 180));
                let x = size.width.saturating_sub(window_size.width + 20);
                let y = size.height.saturating_sub(window_size.height + 60);
                let _ = pergunta_window.set_position(tauri::PhysicalPosition::new(x, y));
            }

            // JANELA CLIPBOARD (HISTÓRICO DA ÁREA DE TRANSFERÊNCIA)

            #[cfg(debug_assertions)]
            let clipboard_url = WebviewUrl::External(
                tauri::Url::parse("http://localhost:1420/clipboard.html").unwrap(),
            );
            #[cfg(not(debug_assertions))]
            let clipboard_url = WebviewUrl::App("clipboard.html".into());

            let clipboard_window = WebviewWindowBuilder::new(
                app, "clipboard",
                clipboard_url,
            )
            .inner_size(360.0, 420.0)
            .decorations(false)
            .transparent(true)
            .resizable(false)
            .shadow(false)
            .always_on_top(true)
            .skip_taskbar(true)
            .visible(false)
            .build()?;

            if let Ok(Some(monitor)) = clipboard_window.primary_monitor() {
                let size = monitor.size();
                let window_size = clipboard_window
                    .outer_size()
                    .unwrap_or(tauri::PhysicalSize::new(360, 420));
                let x = size.width.saturating_sub(window_size.width + 20);
                let y = size.height.saturating_sub(window_size.height + 60);
                let _ = clipboard_window.set_position(tauri::PhysicalPosition::new(x, y));
            }

            // JANELA HISTÓRICO (ÚLTIMAS MENSAGENS DA IA)

            #[cfg(debug_assertions)]
            let historico_url = WebviewUrl::External(
                tauri::Url::parse("http://localhost:1420/historico.html").unwrap(),
            );
            #[cfg(not(debug_assertions))]
            let historico_url = WebviewUrl::App("historico.html".into());

            let historico_window = WebviewWindowBuilder::new(
                app, "historico",
                historico_url,
            )
            .inner_size(360.0, 420.0)
            .decorations(false)
            .transparent(true)
            .resizable(false)
            .shadow(false)
            .always_on_top(true)
            .skip_taskbar(true)
            .visible(false)
            .build()?;

            if let Ok(Some(monitor)) = historico_window.primary_monitor() {
                let size = monitor.size();
                let window_size = historico_window
                    .outer_size()
                    .unwrap_or(tauri::PhysicalSize::new(360, 420));
                let x = size.width.saturating_sub(window_size.width + 20);
                let y = size.height.saturating_sub(window_size.height + 60);
                let _ = historico_window.set_position(tauri::PhysicalPosition::new(x, y));
            }

            // JANELA RADIAL (PIE MENU FLUTUANTE)

            #[cfg(debug_assertions)]
            let radial_url = WebviewUrl::External(
                tauri::Url::parse("http://localhost:1420/radial.html").unwrap(),
            );
            #[cfg(not(debug_assertions))]
            let radial_url = WebviewUrl::App("radial.html".into());

            let _radial_window = WebviewWindowBuilder::new(
                app, "radial",
                radial_url,
            )
            .inner_size(380.0, 380.0)
            .decorations(false)
            .transparent(true)
            .resizable(false)
            .shadow(false)
            .always_on_top(true)
            .skip_taskbar(true)
            .visible(false)
            .build()?;

            // CONFIGURAÇÃO DA BANDEJA DO SISTEMA
            let menu = construir_menu_bandeja(app.handle())?;

            let _tray = TrayIconBuilder::with_id("main-tray")
                .menu(&menu)
                .show_menu_on_left_click(true)
                .icon(app.default_window_icon().unwrap().clone())
                .on_menu_event(|app, event| {
                    let id_str = event.id.as_ref();
                    match id_str {
                        "quit" => std::process::exit(0),
                        "show" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.center();
                                window.show().unwrap();
                                window.set_focus().unwrap();
                            }
                        }
                        "pergunta" => {
                            comandos::limpar_historico_conversa();
                            if let Some(window) = app.get_webview_window("pergunta") {
                                window.show().unwrap();
                                window.set_focus().unwrap();
                            }
                        }
                        "historico" => {
                            comandos::abrir_historico(app.clone());
                        }
                        "clipboard" => {
                            if let Some(window) = app.get_webview_window("clipboard") {
                                let _ = window.show();
                                let _ = window.unminimize();
                                let _ = window.set_focus();
                            }
                        }
                        "timer" => {
                            if let Some(window) = app.get_webview_window("timer") {
                                window.show().unwrap();
                                window.set_focus().unwrap();
                            }
                        }
                        "faxina" => comandos::limpar_lixo_pc(app.clone()),
                        "organiza" => comandos::organizar_downloads(app.clone()),
                        "otimizar_ram" => comandos::limpar_cache_ram(app.clone()),
                        "busca" => busca::abrir_busca(app.clone()),
                        _ => {}
                    }
                })
                .build(app)?;

            if let Some(busca_window) = app.get_webview_window("busca") {
                if let Ok(Some(monitor)) = busca_window.primary_monitor() {
                    let size = monitor.size();
                    let window_size = busca_window
                        .outer_size()
                        .unwrap_or(tauri::PhysicalSize::new(380, 460));
                    let x = size.width.saturating_sub(window_size.width + 20);
                    let y = size.height.saturating_sub(window_size.height + 60);
                    let _ = busca_window.set_position(tauri::PhysicalPosition::new(x, y));
                }
            }

            if let Some(main_window) = app.get_webview_window("main") {
                let args: Vec<String> = std::env::args().collect();
                
                if !args.contains(&"--autostart".to_string()) {
                    let _ = main_window.center();
                    main_window.show().unwrap();
                    main_window.set_focus().unwrap();
                }
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("Erro ao iniciar o Tauri");
}
