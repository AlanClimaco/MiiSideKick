#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod comandos; 

use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
    Manager, WebviewUrl, WebviewWindowBuilder
};

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            comandos::get_system_info, 
            comandos::esconder_janela,
            comandos::esconder_popup, 
            comandos::esconder_timer,
            comandos::limpar_lixo_pc,
            comandos::pegar_dados_notificacao,
            comandos::organizar_downloads,
            comandos::iniciar_timer
        ])
        .setup(|app| {
            
            // JANELA POPUP (NOTIFICAÇÕES)

            #[cfg(debug_assertions)]
            let popup_url = WebviewUrl::External(tauri::Url::parse("http://localhost:1420/notificacao.html").unwrap());
            #[cfg(not(debug_assertions))]
            let popup_url = WebviewUrl::App("notificacao.html".into());

            let popup_window = WebviewWindowBuilder::new(
                app,
                "popup",
                popup_url
            )
            .inner_size(350.0, 180.0)
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
                let window_size = popup_window.outer_size().unwrap_or(tauri::PhysicalSize::new(350, 180));
                let x = size.width.saturating_sub(window_size.width + 20);
                let y = size.height.saturating_sub(window_size.height + 60);
                let _ = popup_window.set_position(tauri::PhysicalPosition::new(x, y));
            }

            // JANELA TIMER (FORMULÁRIO DE LEMBRETE)

            #[cfg(debug_assertions)]
            let timer_url = WebviewUrl::External(tauri::Url::parse("http://localhost:1420/timer.html").unwrap());
            #[cfg(not(debug_assertions))]
            let timer_url = WebviewUrl::App("timer.html".into());

            let timer_window = WebviewWindowBuilder::new(
                app,
                "timer", // ID da janela do timer
                timer_url
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
                let window_size = timer_window.outer_size().unwrap_or(tauri::PhysicalSize::new(360, 420)); 
                let x = size.width.saturating_sub(window_size.width + 20);
                let y = size.height.saturating_sub(window_size.height + 60);
                let _ = timer_window.set_position(tauri::PhysicalPosition::new(x, y));
            }

            // CONFIGURAÇÃO DA BANDEJA DO SISTEMA

            let show_i = MenuItem::with_id(app, "show", "⚙️ Abrir Setup", true, None::<&str>)?;
            let timer_i = MenuItem::with_id(app, "timer", "⏱️ Novo Lembrete", true, None::<&str>)?;
            let organiza_i = MenuItem::with_id(app, "organiza", "📁 Organizar Arquivos", true, None::<&str>)?;
            let faxina_i = MenuItem::with_id(app, "faxina", "🧹 Limpar Lixo do PC", true, None::<&str>)?;
            let quit_i = MenuItem::with_id(app, "quit", "❌ Sair", true, None::<&str>)?;

            let menu = Menu::with_items(app, &[&show_i, &timer_i, &organiza_i, &faxina_i, &quit_i])?;

            let _tray = TrayIconBuilder::new()
                .menu(&menu)
                .show_menu_on_left_click(true)
                .icon(app.default_window_icon().unwrap().clone()) 
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => std::process::exit(0),
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            window.show().unwrap();
                            window.set_focus().unwrap();
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
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click { button: MouseButton::Left, .. } = event {
                        if let Some(window) = tray.app_handle().get_webview_window("main") {
                            window.show().unwrap();
                            window.set_focus().unwrap();
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("Erro ao iniciar o Tauri");
}