#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod comandos;
mod gatilhos;
mod sistema;
mod clipboard;

use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Manager, WebviewUrl, WebviewWindowBuilder,
};

pub fn construir_menu_bandeja(app: &tauri::AppHandle) -> Result<Menu<tauri::Wry>, tauri::Error> {
    let cfg = comandos::carregar_configuracoes();
    let is_en = cfg.idioma == "en";

    let show_i = MenuItem::with_id(
        app,
        "show",
        if is_en { "⚙️ Open Setup" } else { "⚙️ Abrir Setup" },
        true,
        None::<&str>,
    )?;
    let pergunta_i = MenuItem::with_id(
        app,
        "pergunta",
        if is_en { "💬 Chat with Mii" } else { "💬 Conversar com o Mii" },
        true,
        None::<&str>,
    )?;

    let historico_i = MenuItem::with_id(
        app,
        "historico",
        if is_en { "💡 Recent AI Messages" } else { "💡 Últimas Mensagens da IA" },
        true,
        None::<&str>,
    )?;

    let clipboard_i = MenuItem::with_id(
        app,
        "clipboard",
        if is_en { "📋 Clipboard History" } else { "📋 Histórico de Copiar" },
        true,
        None::<&str>,
    )?;
    let timer_i = MenuItem::with_id(
        app,
        "timer",
        if is_en { "⏱️ New Reminder" } else { "⏱️ Novo Lembrete" },
        true,
        None::<&str>,
    )?;
    let organiza_i = MenuItem::with_id(
        app,
        "organiza",
        if is_en { "📁 Organize Files" } else { "📁 Organizar Arquivos" },
        true,
        None::<&str>,
    )?;
    let faxina_i = MenuItem::with_id(
        app,
        "faxina",
        if is_en { "🧹 Clean Junk Files" } else { "🧹 Limpar Lixo do PC" },
        true,
        None::<&str>,
    )?;
    let otimizar_ram_i = MenuItem::with_id(
        app,
        "otimizar_ram",
        if is_en { "⚡ Optimize RAM" } else { "⚡ Otimizar RAM" },
        true,
        None::<&str>,
    )?;
    let quit_i = MenuItem::with_id(
        app,
        "quit",
        if is_en { "❌ Quit" } else { "❌ Sair" },
        true,
        None::<&str>,
    )?;

    Menu::with_items(
        app,
        &[
            &show_i,
            &pergunta_i,
            &historico_i,
            &clipboard_i,
            &timer_i,
            &organiza_i,
            &faxina_i,
            &otimizar_ram_i,
            &quit_i,
        ],
    )
}

pub fn atualizar_menu_bandeja(app: &tauri::AppHandle) {
    if let Some(tray) = app.tray_by_id("main-tray") {
        if let Ok(menu) = construir_menu_bandeja(app) {
            let _ = tray.set_menu(Some(menu));
        }
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
                .with_shortcut("ctrl+shift+m")
                .expect("Erro ao registrar atalho global Ctrl+Shift+M")
                .with_handler(|app, _shortcut, event| {
                    if event.state() == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                        comandos::limpar_historico_conversa();
                        if let Some(window) = app.get_webview_window("pergunta") {
                            let _ = window.show();
                            let _ = window.unminimize();
                            let _ = window.set_focus();
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
            comandos::esconder_pergunta,
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
            comandos::verificar_atualizacoes_manual
        ])
        .setup(|app| {
            // Inicializa o motor de gatilhos proativos e checagem de updates
            gatilhos::iniciar_motor_proativo(app.handle().clone());

            // Inicializa o monitoramento contínuo do clipboard
            clipboard::iniciar_monitor_clipboard();

            // JANELA POPUP (NOTIFICAÇÕES)

            #[cfg(debug_assertions)]
            let popup_url = WebviewUrl::External(
                tauri::Url::parse("http://localhost:1420/notificacao.html").unwrap(),
            );
            #[cfg(not(debug_assertions))]
            let popup_url = WebviewUrl::App("notificacao.html".into());

            let popup_window = WebviewWindowBuilder::new(app, "popup", popup_url)
                .inner_size(420.0, 240.0)
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
                    .unwrap_or(tauri::PhysicalSize::new(420, 240));
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
                        _ => {}
                    }
                })
                .build(app)?;

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
