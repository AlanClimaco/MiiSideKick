use tauri::{AppHandle, Manager};
use std::ffi::c_void;
use std::sync::Mutex;

#[link(name = "shell32")]
extern "system" { 
    fn SHEmptyRecycleBinW(hwnd: *mut c_void, pszRootPath: *const u16, dwFlags: u32) -> i32; 
}

#[derive(serde::Serialize)]
pub struct UserInfo { 
    pub name: String, 
    pub downloads: String 
}

#[derive(serde::Serialize, Clone)]
pub struct NotificacaoPayload { 
    pub texto: String, 
    pub tipo: String 
}

static ULTIMA_NOTIFICACAO: Mutex<Option<NotificacaoPayload>> = Mutex::new(None);

#[tauri::command]
pub fn get_system_info() -> UserInfo {
    let name = std::env::var("USERNAME").unwrap_or_else(|_| "Usuário".to_string());
    let user_profile = std::env::var("USERPROFILE").unwrap_or_else(|_| "C:\\".to_string());
    let downloads = format!("{}\\Downloads", user_profile);
    UserInfo { name, downloads }
}

#[tauri::command]
pub fn esconder_janela(app: tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        window.hide().unwrap();
    }
}

// FAZ O POP UP SUMIR
#[tauri::command]
pub fn esconder_popup(app: tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("popup") {
        let _ = window.hide();
    }
}

#[tauri::command]
pub fn pegar_dados_notificacao() -> Option<NotificacaoPayload> {
    ULTIMA_NOTIFICACAO.lock().unwrap().clone()
}

pub fn disparar_notificacao(app: &AppHandle, texto: &str, tipo: &str) {
    let payload = NotificacaoPayload {
        texto: texto.to_string(),
        tipo: tipo.to_string(),
    };

    if let Ok(mut mem) = ULTIMA_NOTIFICACAO.lock() {
        *mem = Some(payload.clone());
    }

    if let Some(janela) = app.get_webview_window("popup") {
        let _ = janela.show();
        let _ = janela.unminimize();
        
        let texto_seguro = texto.replace("'", "\\'").replace("\n", "\\n");
        let tipo_seguro = tipo.replace("'", "\\'");
        
        let script = format!("if (window.atualizarInterface) {{ window.atualizarInterface('{}', '{}'); }}", texto_seguro, tipo_seguro);
        let _ = janela.eval(&script);
    }
}

#[tauri::command]
pub fn limpar_lixo_pc(app: AppHandle) {

    // AVISA IMEDIATAMENTE QUANDO COMEÇA
    disparar_notificacao(&app, "Iniciando a faxina do sistema...", "normal");

    std::thread::spawn(move || {
        unsafe { SHEmptyRecycleBinW(std::ptr::null_mut(), std::ptr::null(), 7); }
        let temp_dir = std::env::temp_dir();
        let mut apagados = 0;

        if let Ok(entradas) = std::fs::read_dir(temp_dir) {
            for entrada in entradas.flatten() {
                let path = entrada.path();
                if path.is_file() {
                    if std::fs::remove_file(&path).is_ok() { apagados += 1; }
                } else if path.is_dir() {
                    if std::fs::remove_dir_all(&path).is_ok() { apagados += 1; }
                }
            }
        }
        
        // Dá um pequeno tempo de 1,5s para o usuário conseguir ler a primeira mensagem "Iniciando..."
        std::thread::sleep(std::time::Duration::from_millis(1500));

        // ATUALIZA A TELA COM O RESULTADO
        if apagados == 0 {
            disparar_notificacao(&app, "Lixeira já estava vazia. Tudo limpo!", "frustrado");
        } else {
            let msg = format!("Faxina feita!\n{} arquivos inúteis removidos.", apagados);
            disparar_notificacao(&app, &msg, "tranquilo");
        }
    });
}

use std::fs;

#[tauri::command]
pub fn organizar_downloads(app: AppHandle) {
    // Avisa que começou
    disparar_notificacao(&app, "Iniciando organização de arquivos...", "normal");

    // Joga pra background thread para não travar a bandeja!
    std::thread::spawn(move || {
        let mut movidos = 0;
        
        // Pega os caminhos padrão do usuário logado no Windows
        if let (Some(downloads_dir), Some(docs_dir), Some(musics_dir), Some(pics_dir), Some(videos_dir)) = (
            dirs::download_dir(),
            dirs::document_dir(),
            dirs::audio_dir(),
            dirs::picture_dir(),
            dirs::video_dir(),
        ) {
            let instaladores_dir = downloads_dir.join("Instaladores");
            let compactados_dir = downloads_dir.join("Compactados");

            // Cria as pastas base se não existirem
            let _ = fs::create_dir_all(&instaladores_dir);
            let _ = fs::create_dir_all(&compactados_dir);

            // Lê a pasta de downloads
            if let Ok(entradas) = fs::read_dir(&downloads_dir) {
                for entrada in entradas.flatten() {
                    let path = entrada.path();
                    
                    if path.is_file() {
                        // Pega a extensão do arquivo em minúsculo
                        if let Some(ext) = path.extension().and_then(|e| e.to_str()).map(|e| e.to_lowercase()) {
                            let file_name = path.file_name().unwrap();
                            
                            let destino = match ext.as_str() {
                                "exe" | "msi" => Some(instaladores_dir.join(file_name)),
                                "zip" | "rar" | "7z" | "tar" | "gz" => Some(compactados_dir.join(file_name)),
                                "pdf" | "docx" | "doc" | "txt" | "xlsx" | "pptx" => Some(docs_dir.join(file_name)),
                                "mp3" | "wav" | "flac" | "aac" | "ogg" => Some(musics_dir.join(file_name)),
                                "jpg" | "jpeg" | "png" | "gif" | "bmp" | "tiff" | "webp" => Some(pics_dir.join(file_name)),
                                "mp4" | "mkv" | "avi" | "mov" | "wmv" | "flv" => Some(videos_dir.join(file_name)),
                                _ => None, // Se não for nenhuma dessas, ignora o arquivo
                            };

                            // Se encontrou uma pasta destino, move o arquivo
                            if let Some(dest) = destino {
                                if fs::rename(&path, &dest).is_ok() {
                                    movidos += 1;
                                }
                            }
                        }
                    }
                }
            }
        }

        std::thread::sleep(std::time::Duration::from_millis(1500));

        if movidos == 0 {
            disparar_notificacao(&app, "Tudo limpo! Não havia o que organizar.", "frustrado");
        } else {
            let msg = format!("Mágica feita!\n{} arquivos guardados.", movidos);
            disparar_notificacao(&app, &msg, "tranquilo");
        }
    });
}

#[tauri::command]
pub fn iniciar_timer(app: AppHandle, minutos: u64, mensagem: String) {
    let msg_inicio = if minutos == 1 {
        format!("Ok! Te aviso em 1 minuto.")
    } else {
        format!("Ok! Te aviso em {} minutos.", minutos)
    };
    
    disparar_notificacao(&app, &msg_inicio, "tranquilo");

    let app_clone = app.clone();
    
    std::thread::spawn(move || {
        let metade_segundos = (minutos * 60) / 2;
        let resto_segundos = (minutos * 60) - metade_segundos;

        std::thread::sleep(std::time::Duration::from_secs(metade_segundos));

        let msg_metade = if minutos == 1 {
            format!("Atenção! Falta meio minuto para:\n{}", mensagem)
        } else if minutos == 3 {
            format!("Atenção! Falta 1 minuto e meio para:\n{}", mensagem)
        } else if minutos % 2 != 0 {
            let inteira = minutos / 2;
            format!("Atenção! Faltam {} minutos e meio para:\n{}", inteira, mensagem)
        } else {
            let metade = minutos / 2;
            if metade == 1 {
                format!("Atenção! Falta 1 minuto para:\n{}", mensagem)
            } else {
                format!("Atenção! Faltam {} minutos para:\n{}", metade, mensagem)
            }
        };

        disparar_notificacao(&app_clone, &msg_metade, "normal");

        std::thread::sleep(std::time::Duration::from_secs(resto_segundos));

        let msg_final = format!("ACABOU O TEMPO!\n{}", mensagem);
        disparar_notificacao(&app_clone, &msg_final, "assustado");
    });
}

#[tauri::command]
pub fn esconder_timer(app: tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("timer") {
        let _ = window.hide();
    }
}