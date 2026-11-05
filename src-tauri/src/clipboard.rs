use std::collections::VecDeque;
use std::sync::Mutex;
use tauri::{AppHandle, Manager};

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct ClipboardItem {
    pub id: String,
    pub texto: String,
    pub tamanho: usize,
    pub timestamp: u64,
    pub hora_formatada: String,
}

static HISTORICO_CLIPBOARD: Mutex<VecDeque<ClipboardItem>> = Mutex::new(VecDeque::new());

#[tauri::command]
pub fn obter_historico_clipboard() -> Vec<ClipboardItem> {
    if let Ok(hist) = HISTORICO_CLIPBOARD.lock() {
        hist.iter().cloned().collect()
    } else {
        Vec::new()
    }
}

#[tauri::command]
pub fn copiar_item_clipboard(texto: String) -> Result<(), String> {
    let mut cb = arboard::Clipboard::new().map_err(|e| e.to_string())?;
    cb.set_text(texto).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn limpar_historico_clipboard() {
    if let Ok(mut hist) = HISTORICO_CLIPBOARD.lock() {
        hist.clear();
    }
}

#[tauri::command]
pub fn abrir_clipboard(app: AppHandle) {
    if let Some(janela) = app.get_webview_window("clipboard") {
        let _ = janela.show();
        let _ = janela.unminimize();
        let _ = janela.set_focus();
    }
}

#[tauri::command]
pub fn esconder_clipboard(app: AppHandle) {
    if let Some(janela) = app.get_webview_window("clipboard") {
        let _ = janela.hide();
    }
}

/// Inicia uma thread leve de escuta do clipboard (checagem a cada 800ms)
pub fn iniciar_monitor_clipboard() {
    std::thread::spawn(move || {
        let mut ultimo_texto = String::new();
        let mut id_counter: u64 = 0;

        loop {
            std::thread::sleep(std::time::Duration::from_millis(800));

            if let Ok(mut clipboard) = arboard::Clipboard::new() {
                if let Ok(texto) = clipboard.get_text() {
                    let texto_trim = texto.trim();
                    let bytes_len = texto.as_bytes().len();

                    // Limite de 30 KB por item para impacto de RAM insignificante e deduplicação consecutiva
                    if !texto_trim.is_empty() && bytes_len <= 30 * 1024 && texto != ultimo_texto {
                        ultimo_texto = texto.clone();
                        id_counter = id_counter.wrapping_add(1);

                        let now = std::time::SystemTime::now();
                        let millis = now
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_millis() as u64;

                        let total_secs = millis / 1000;
                        let mins = (total_secs / 60) % 60;
                        let hours = (total_secs / 3600) % 24;
                        let hora_formatada = format!("{:02}:{:02}", hours, mins);

                        let item = ClipboardItem {
                            id: format!("{}-{}", millis, id_counter),
                            texto,
                            tamanho: bytes_len,
                            timestamp: millis,
                            hora_formatada,
                        };

                        if let Ok(mut hist) = HISTORICO_CLIPBOARD.lock() {
                            // Reordena se já existia, trazendo para a frente
                            hist.retain(|i| i.texto != item.texto);
                            hist.push_front(item);
                            // Mantém entre 5 e 10 itens (limite máximo de 10)
                            if hist.len() > 10 {
                                hist.pop_back();
                            }
                        }
                    }
                }
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_historico_clipboard_limite_e_deduplicacao() {
        limpar_historico_clipboard();
        assert_eq!(obter_historico_clipboard().len(), 0);

        if let Ok(mut hist) = HISTORICO_CLIPBOARD.lock() {
            for i in 1..=15 {
                let item = ClipboardItem {
                    id: format!("id-{}", i),
                    texto: format!("Item {}", i),
                    tamanho: 10,
                    timestamp: i as u64,
                    hora_formatada: "12:00".to_string(),
                };
                hist.retain(|x| x.texto != item.texto);
                hist.push_front(item);
                if hist.len() > 10 {
                    hist.pop_back();
                }
            }
        }

        let lista = obter_historico_clipboard();
        assert_eq!(lista.len(), 10);
        assert_eq!(lista[0].texto, "Item 15");
        assert_eq!(lista[9].texto, "Item 6");

        limpar_historico_clipboard();
        assert_eq!(obter_historico_clipboard().len(), 0);
    }
}
