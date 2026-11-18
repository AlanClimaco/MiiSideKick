use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::SystemTime;
use tauri::{AppHandle, Emitter, Manager};

static TERMO_PRE_PESQUISA: Mutex<Option<String>> = Mutex::new(None);

pub fn definir_termo_pre_pesquisa(termo: &str) {
    if let Ok(mut lock) = TERMO_PRE_PESQUISA.lock() {
        *lock = Some(termo.to_string());
    }
}

#[tauri::command]
pub fn obter_termo_pre_pesquisa() -> Option<String> {
    if let Ok(mut lock) = TERMO_PRE_PESQUISA.lock() {
        lock.take()
    } else {
        None
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct ResultadoBusca {
    pub nome: String,
    pub caminho: String,
    pub tamanho_formatado: String,
    pub data_modificacao: String,
}

/// Normaliza caracteres com acentos básicos para permitir busca tolerante
pub fn normalizar_texto(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'á' | 'à' | 'ã' | 'â' | 'ä' | 'Á' | 'À' | 'Ã' | 'Â' | 'Ä' => 'a',
            'é' | 'è' | 'ê' | 'ë' | 'É' | 'È' | 'Ê' | 'Ë' => 'e',
            'í' | 'ì' | 'î' | 'ï' | 'Í' | 'Ì' | 'Î' | 'Ï' => 'i',
            'ó' | 'ò' | 'õ' | 'ô' | 'ö' | 'Ó' | 'Ò' | 'Õ' | 'Ô' | 'Ö' => 'o',
            'ú' | 'ù' | 'û' | 'ü' | 'Ú' | 'Ù' | 'Û' | 'Ü' => 'u',
            'ç' | 'Ç' => 'c',
            'ñ' | 'Ñ' => 'n',
            other => other.to_ascii_lowercase(),
        })
        .collect()
}

/// Verifica se o nome do arquivo corresponde à busca (case-insensitive, sem acentos, por partes)
pub fn corresponde_busca(nome_arquivo: &str, termo_busca: &str) -> bool {
    let nome_norm = normalizar_texto(nome_arquivo);
    let termo_norm = normalizar_texto(termo_busca.trim());
    if termo_norm.is_empty() {
        return false;
    }
    let partes: Vec<&str> = termo_norm.split_whitespace().collect();
    if partes.is_empty() {
        return false;
    }
    partes.iter().all(|parte| nome_norm.contains(parte))
}

/// Formata o tamanho em bytes para representação legível (B, KB, MB, GB)
pub fn formatar_tamanho(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Formata a data de modificação no formato DD/MM/AAAA HH:MM usando o algoritmo civil gregoriano
pub fn formatar_data_modificacao(time: SystemTime) -> String {
    let dur = match time.duration_since(std::time::UNIX_EPOCH) {
        Ok(d) => d,
        Err(_) => return "Desconhecido".to_string(),
    };
    let secs = dur.as_secs();
    let sec_in_day = (secs % 86400) as u32;
    let hours = sec_in_day / 3600;
    let mins = (sec_in_day % 3600) / 60;

    let mut days = (secs / 86400) as i64;
    days += 719468;
    let era = (if days >= 0 { days } else { days - 146096 }) / 146097;
    let doe = (days - era * 146097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = (yoe as i64) + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };

    format!("{:02}/{:02}/{:04} {:02}:{:02}", d, m, y, hours, mins)
}

struct ItemCandidato {
    resultado: ResultadoBusca,
    modificado: SystemTime,
    score: i64,
}

/// Pastas padrão do usuário onde a busca deve ser realizada (Downloads, Documentos, Desktop, Imagens)
pub fn obter_pastas_padrao() -> Vec<PathBuf> {
    let mut pastas = Vec::new();
    if let Some(d) = dirs::download_dir() {
        if d.exists() {
            pastas.push(d);
        }
    }
    if let Some(d) = dirs::document_dir() {
        if d.exists() {
            pastas.push(d);
        }
    }
    if let Some(d) = dirs::desktop_dir() {
        if d.exists() {
            pastas.push(d);
        }
    }
    if let Some(d) = dirs::picture_dir() {
        if d.exists() {
            pastas.push(d);
        }
    }
    pastas
}

/// Ignora pastas irrelevantes, ocultas ou pesadas para manter a busca instantânea
fn deve_ignorar_pasta(nome: &str) -> bool {
    let lower = nome.to_ascii_lowercase();
    lower.starts_with('.')
        || lower == "node_modules"
        || lower == "target"
        || lower == "appdata"
        || lower == "$recycle.bin"
        || lower == "windows"
        || lower == "vendor"
        || lower == ".git"
}

/// Varredura recursiva com limite de profundidade (3 níveis) e limite de itens visitados
fn varrer_diretorio(
    dir: &Path,
    termo: &str,
    profundidade_atual: u32,
    max_profundidade: u32,
    candidatos: &mut Vec<ItemCandidato>,
    itens_visitados: &mut usize,
    max_itens: usize,
) {
    if profundidade_atual > max_profundidade || *itens_visitados >= max_itens {
        return;
    }

    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    let termo_norm = normalizar_texto(termo.trim());

    for entry in entries.flatten() {
        *itens_visitados += 1;
        if *itens_visitados >= max_itens {
            break;
        }

        let path = entry.path();
        let file_type = match entry.file_type() {
            Ok(ft) => ft,
            Err(_) => continue,
        };

        let file_name = entry.file_name().to_string_lossy().to_string();

        if file_type.is_dir() {
            if !deve_ignorar_pasta(&file_name) {
                varrer_diretorio(
                    &path,
                    termo,
                    profundidade_atual + 1,
                    max_profundidade,
                    candidatos,
                    itens_visitados,
                    max_itens,
                );
            }
        } else if file_type.is_file() {
            if corresponde_busca(&file_name, termo) {
                if let Ok(metadata) = entry.metadata() {
                    let tamanho = metadata.len();
                    let modificado = metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);
                    let tamanho_fmt = formatar_tamanho(tamanho);
                    let data_fmt = formatar_data_modificacao(modificado);

                    // Cálculo de relevância
                    let nome_norm = normalizar_texto(&file_name);
                    let mut score = 0i64;

                    if nome_norm == termo_norm {
                        score += 1000;
                    } else if nome_norm.starts_with(&termo_norm) {
                        score += 500;
                    } else if nome_norm.contains(&termo_norm) {
                        score += 200;
                    }

                    candidatos.push(ItemCandidato {
                        resultado: ResultadoBusca {
                            nome: file_name,
                            caminho: path.to_string_lossy().to_string(),
                            tamanho_formatado: tamanho_fmt,
                            data_modificacao: data_fmt,
                        },
                        modificado,
                        score,
                    });
                }
            }
        }
    }
}

/// Executa a busca em segundo plano pelas 4 pastas padrão retornando os 10 mais relevantes/recentes
pub fn executar_busca_arquivos_locais(termo: &str) -> Result<Vec<ResultadoBusca>, String> {
    let termo_limpo = termo.trim();
    if termo_limpo.is_empty() {
        return Ok(Vec::new());
    }

    let pastas = obter_pastas_padrao();
    let mut candidatos = Vec::new();
    let mut itens_visitados = 0;
    const MAX_ITENS: usize = 5000;
    const MAX_PROFUNDIDADE: u32 = 3;

    for pasta in pastas {
        varrer_diretorio(
            &pasta,
            termo_limpo,
            0,
            MAX_PROFUNDIDADE,
            &mut candidatos,
            &mut itens_visitados,
            MAX_ITENS,
        );
    }

    // Ordena por relevância e recência
    candidatos.sort_by(|a, b| {
        b.score
            .cmp(&a.score)
            .then_with(|| b.modificado.cmp(&a.modificado))
    });

    let top10 = candidatos
        .into_iter()
        .take(10)
        .map(|c| c.resultado)
        .collect();

    Ok(top10)
}

#[tauri::command]
pub async fn procurar_arquivos_locais(termo: String) -> Result<Vec<ResultadoBusca>, String> {
    if termo.trim().is_empty() {
        return Ok(Vec::new());
    }

    let termo_clone = termo.clone();
    tauri::async_runtime::spawn_blocking(move || executar_busca_arquivos_locais(&termo_clone))
        .await
        .map_err(|e| format!("Erro ao executar busca: {}", e))?
}

#[tauri::command]
pub fn abrir_pasta_com_arquivo(caminho: String) -> Result<(), String> {
    let caminho_limpo = caminho.trim();
    if caminho_limpo.is_empty() {
        return Err("Caminho não fornecido".to_string());
    }

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        use std::process::Command;

        let caminho_win = caminho_limpo.replace('/', "\\");
        let arg = format!("/select,\"{}\"", caminho_win);

        Command::new("explorer.exe")
            .raw_arg(&arg)
            .spawn()
            .map_err(|e| format!("Erro ao abrir o Windows Explorer: {}", e))?;

        Ok(())
    }

    #[cfg(target_os = "linux")]
    {
        if let Some(parent) = Path::new(caminho_limpo).parent() {
            let _ = std::process::Command::new("xdg-open").arg(parent).spawn();
        }
        Ok(())
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        Ok(())
    }
}

#[tauri::command]
pub fn abrir_busca(app: AppHandle) {
    if let Some(janela) = app.get_webview_window("busca") {
        let _ = janela.show();
        let _ = janela.unminimize();
        let _ = janela.set_focus();
    }
}

#[tauri::command]
pub fn abrir_busca_com_termo(app: AppHandle, termo: Option<String>) {
    let termo_final = if let Some(t) = termo.filter(|s| !s.trim().is_empty()) {
        definir_termo_pre_pesquisa(&t);
        t
    } else if let Ok(lock) = TERMO_PRE_PESQUISA.lock() {
        lock.clone().unwrap_or_default()
    } else {
        String::new()
    };

    if let Some(janela) = app.get_webview_window("busca") {
        let _ = janela.show();
        let _ = janela.unminimize();
        let _ = janela.set_focus();
        if !termo_final.trim().is_empty() {
            let _ = janela.emit("termo_busca", termo_final);
        }
    }
}

#[tauri::command]
pub fn esconder_busca(app: AppHandle) {
    if let Some(janela) = app.get_webview_window("busca") {
        let _ = janela.hide();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalizar_texto_acentos() {
        assert_eq!(normalizar_texto("Currículo"), "curriculo");
        assert_eq!(normalizar_texto("AÇÃO"), "acao");
        assert_eq!(normalizar_texto("Relatório_2026.PDF"), "relatorio_2026.pdf");
        assert_eq!(normalizar_texto("Cômodo e Úmido"), "comodo e umido");
    }

    #[test]
    fn test_corresponde_busca_tolerante() {
        assert!(corresponde_busca("Curriculo_2026.pdf", "curricul"));
        assert!(corresponde_busca("Currículo_2026.pdf", "curriculo"));
        assert!(corresponde_busca("Documento_Importante.docx", "import"));
        assert!(corresponde_busca("Minha_Fatura_Outubro.pdf", "fatura outubro"));
        assert!(!corresponde_busca("OutroArquivo.txt", "curriculo"));
    }

    #[test]
    fn test_formatar_tamanho() {
        assert_eq!(formatar_tamanho(500), "500 B");
        assert_eq!(formatar_tamanho(1024), "1.0 KB");
        assert_eq!(formatar_tamanho(2048 * 1024), "2.0 MB");
        assert_eq!(formatar_tamanho(1500 * 1024 * 1024), "1.5 GB");
    }

    #[test]
    fn test_formatar_data_modificacao() {
        // 1731888000 = 2024-11-18 00:00:00 UTC
        let t = SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1731888000);
        let s = formatar_data_modificacao(t);
        assert!(s.contains("18/11/2024"));
    }
}
