use std::fs;
use std::path::Path;
use std::sync::Mutex;
use tauri::{AppHandle, Manager};

#[cfg(target_os = "windows")]
#[link(name = "shell32")]
extern "system" {
    fn SHEmptyRecycleBinW(hwnd: *mut std::ffi::c_void, pszRootPath: *const u16, dwFlags: u32) -> i32;
}

#[derive(serde::Serialize)]
pub struct UserInfo {
    pub name: String,
    pub downloads: String,
}

#[derive(serde::Serialize, Clone)]
pub struct NotificacaoPayload {
    pub texto: String,
    pub tipo: String,
}

static ULTIMA_NOTIFICACAO: Mutex<Option<NotificacaoPayload>> = Mutex::new(None);

#[tauri::command]
pub fn get_system_info() -> UserInfo {
    let name = std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "Usuário".to_string());
    let downloads = dirs::download_dir()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|| {
            let base = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
            base.join("Downloads").to_string_lossy().into_owned()
        });
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

        let script = format!(
            "if (window.atualizarInterface) {{ window.atualizarInterface('{}', '{}'); }}",
            texto_seguro, tipo_seguro
        );
        let _ = janela.eval(&script);
    }
}

#[tauri::command]
pub fn limpar_lixo_pc(app: AppHandle) {
    // AVISA IMEDIATAMENTE QUANDO COMEÇA
    disparar_notificacao(&app, "Iniciando a faxina do sistema...", "normal");

    std::thread::spawn(move || {
        #[cfg(target_os = "windows")]
        unsafe {
            SHEmptyRecycleBinW(std::ptr::null_mut(), std::ptr::null(), 7);
        }

        #[cfg(target_os = "linux")]
        {
            if let Some(home) = dirs::home_dir() {
                let trash_files = home.join(".local/share/Trash/files");
                let trash_info = home.join(".local/share/Trash/info");

                let _ = fs::remove_dir_all(&trash_files);
                let _ = fs::create_dir_all(&trash_files);

                let _ = fs::remove_dir_all(&trash_info);
                let _ = fs::create_dir_all(&trash_info);
            }
        }

        let temp_dir = std::env::temp_dir();
        let mut apagados = 0;

        if let Ok(entradas) = fs::read_dir(temp_dir) {
            for entrada in entradas.flatten() {
                let path = entrada.path();
                if path.is_file() {
                    if fs::remove_file(&path).is_ok() {
                        apagados += 1;
                    }
                } else if path.is_dir() && fs::remove_dir_all(&path).is_ok() {
                    apagados += 1;
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

/// Estrutura com as referências para as pastas de destino do organizador.
pub(crate) struct PastasDestino<'a> {
    pub instaladores: &'a Path,
    pub compactados: &'a Path,
    pub fontes: &'a Path,
    pub emulacao_saves: &'a Path,
    pub emulacao_roms_nds: &'a Path,
    pub emulacao_roms_3ds: &'a Path,
    pub emulacao_roms_gba: &'a Path,
    pub emulacao_roms_snes: &'a Path,
    pub emulacao_roms_nes: &'a Path,
    pub emulacao_roms_ps1: &'a Path,
    pub emulacao_roms_ps2: &'a Path,
    pub emulacao_roms_psp: &'a Path,
    pub livros_hqs: &'a Path,
    pub codigo: &'a Path,
    pub documentos: &'a Path,
    pub design_3d: &'a Path,
    pub imagens: &'a Path,
    pub musicas: &'a Path,
    pub videos: &'a Path,
}

/// Identifica a pasta de destino correspondente à extensão do arquivo.
pub(crate) fn classificar_arquivo<'a>(path: &Path, pastas: &'a PastasDestino<'a>) -> Option<&'a Path> {
    let ext = path.extension()?.to_str()?.to_lowercase();

    match ext.as_str() {
        // Downloads/Instaladores:
        // Linux: deb, appimage, snap, flatpakref, rpm, run
        // Windows, Mac e Mobile: exe, msi, dmg, pkg, apk, xapk
        "deb" | "appimage" | "snap" | "flatpakref" | "rpm" | "run" | "exe" | "msi"
        | "dmg" | "pkg" | "apk" | "xapk" => Some(pastas.instaladores),

        // A extensão .bin é tratada como instalador/pacote se não houver arquivo .cue correspondente,
        // evitando conflitos com instaladores Linux. Caso haja um .cue correspondente, vai para PS1.
        "bin" => {
            let has_cue = (|| -> Option<bool> {
                let stem = path.file_stem()?;
                if path.with_extension("cue").exists() || path.with_extension("CUE").exists() {
                    return Some(true);
                }
                if pastas.emulacao_roms_ps1.join(stem).with_extension("cue").exists()
                    || pastas.emulacao_roms_ps1.join(stem).with_extension("CUE").exists()
                {
                    return Some(true);
                }
                if let Some(parent) = path.parent() {
                    if !parent.as_os_str().is_empty() {
                        if let Ok(entries) = fs::read_dir(parent) {
                            for entry in entries.flatten() {
                                let p = entry.path();
                                if p.extension()
                                    .and_then(|e| e.to_str())
                                    .map(|e| e.eq_ignore_ascii_case("cue"))
                                    .unwrap_or(false)
                                {
                                    if let Some(cue_stem) = p.file_stem().and_then(|s| s.to_str()) {
                                        if let Some(bin_stem) = stem.to_str() {
                                            if cue_stem.eq_ignore_ascii_case(bin_stem) {
                                                return Some(true);
                                            }
                                        }
                                    }
                                    if let Ok(content) = fs::read_to_string(&p) {
                                        if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                                            if content.to_lowercase().contains(&file_name.to_lowercase()) {
                                                return Some(true);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                if let Ok(entries) = fs::read_dir(pastas.emulacao_roms_ps1) {
                    for entry in entries.flatten() {
                        let p = entry.path();
                        if p.extension()
                            .and_then(|e| e.to_str())
                            .map(|e| e.eq_ignore_ascii_case("cue"))
                            .unwrap_or(false)
                        {
                            if let Some(cue_stem) = p.file_stem().and_then(|s| s.to_str()) {
                                if let Some(bin_stem) = stem.to_str() {
                                    if cue_stem.eq_ignore_ascii_case(bin_stem) {
                                        return Some(true);
                                    }
                                }
                            }
                            if let Ok(content) = fs::read_to_string(&p) {
                                if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                                    if content.to_lowercase().contains(&file_name.to_lowercase()) {
                                        return Some(true);
                                    }
                                }
                            }
                        }
                    }
                }
                Some(false)
            })()
            .unwrap_or(false);

            if has_cue {
                Some(pastas.emulacao_roms_ps1)
            } else {
                Some(pastas.instaladores)
            }
        }

        // Downloads/Compactados:
        // zip, rar, 7z, tar, gz, bz2, xz, zst, tgz
        "zip" | "rar" | "7z" | "tar" | "gz" | "bz2" | "xz" | "zst" | "tgz" => {
            Some(pastas.compactados)
        }

        // Downloads/Fontes:
        // ttf, otf, woff, woff2
        "ttf" | "otf" | "woff" | "woff2" => Some(pastas.fontes),

        // Documentos/Emulacao/Saves e Cheats:
        // cbs, max, psu, xps, pnach, srm, mcd, ps2
        "cbs" | "max" | "psu" | "xps" | "pnach" | "srm" | "mcd" | "ps2" => {
            Some(pastas.emulacao_saves)
        }

        // Documentos/Emulacao/ROMs/NDS:
        // nds
        "nds" => Some(pastas.emulacao_roms_nds),

        // Documentos/Emulacao/ROMs/3DS:
        // 3ds, cia
        "3ds" | "cia" => Some(pastas.emulacao_roms_3ds),

        // Documentos/Emulacao/ROMs/GBA:
        // gba, gbc, gb
        "gba" | "gbc" | "gb" => Some(pastas.emulacao_roms_gba),

        // Documentos/Emulacao/ROMs/SNES:
        // smc, sfc
        "smc" | "sfc" => Some(pastas.emulacao_roms_snes),

        // Documentos/Emulacao/ROMs/NES:
        // nes
        "nes" => Some(pastas.emulacao_roms_nes),

        // Documentos/Emulacao/ROMs/PS1:
        // cue, pbp
        "cue" | "pbp" => Some(pastas.emulacao_roms_ps1),

        // Documentos/Emulacao/ROMs/PS2:
        // iso, img, chd
        "iso" | "img" | "chd" => Some(pastas.emulacao_roms_ps2),

        // Documentos/Emulacao/ROMs/PSP:
        // cso
        "cso" => Some(pastas.emulacao_roms_psp),

        // Documentos/Livros e HQs:
        // epub, mobi, azw, azw3, fb2, cbr, cbz, cbt, cb7
        "epub" | "mobi" | "azw" | "azw3" | "fb2" | "cbr" | "cbz" | "cbt" | "cb7" => {
            Some(pastas.livros_hqs)
        }

        // Documentos/Codigo:
        // Código e Scripts: py, js, mjs, (ts tratado abaixo), jsx, tsx, rs, c, cpp, h, hpp, cs, java, go, php, rb, lua, sh, bash
        // Configurações e Dados: json, yaml, yml, toml, xml, sql, env
        // Documentação e Web: md, markdown, tex, rst, html, css
        "py" | "js" | "mjs" | "jsx" | "tsx" | "rs" | "c" | "cpp" | "h" | "hpp" | "cs" | "java"
        | "go" | "php" | "rb" | "lua" | "sh" | "bash" | "json" | "yaml" | "yml" | "toml"
        | "xml" | "sql" | "env" | "md" | "markdown" | "tex" | "rst" | "html" | "css" => {
            Some(pastas.codigo)
        }

        // A extensão .ts é compartilhada por TypeScript (Código) e MPEG Transport Stream (Vídeo).
        // Distinguimos examinando os primeiros bytes: MPEG-TS inicia com o sync byte 0x47.
        "ts" => {
            let is_video = (|| -> Option<bool> {
                use std::io::Read;
                let mut file = fs::File::open(path).ok()?;
                let mut buf = [0u8; 512];
                let n = file.read(&mut buf).ok()?;
                if n > 0 && buf[0] == 0x47 && buf[..n].contains(&0) {
                    Some(true)
                } else {
                    Some(false)
                }
            })()
            .unwrap_or(false);

            if is_video {
                Some(pastas.videos)
            } else {
                Some(pastas.codigo)
            }
        }

        // Documentos (Documentos de Escritório e Financeiro):
        // Textos: pdf, doc, docx, odt, rtf, txt
        // Planilhas e Financeiro: xlsx, xls, ods, csv, tsv, ofx
        // Apresentações: pptx, ppt, odp, key
        "pdf" | "doc" | "docx" | "odt" | "rtf" | "txt" | "xlsx" | "xls" | "ods" | "csv"
        | "tsv" | "pptx" | "ppt" | "odp" | "key" | "ofx" => Some(pastas.documentos),

        // Imagens/Design e 3D:
        // psd, ai, xcf, blend, obj, stl, fbx
        "psd" | "ai" | "xcf" | "blend" | "obj" | "stl" | "fbx" => Some(pastas.design_3d),

        // Imagens (Fotos e Imagens Gerais):
        // jpg, jpeg, png, gif, bmp, tiff, webp, svg, ico, avif
        "jpg" | "jpeg" | "png" | "gif" | "bmp" | "tiff" | "webp" | "svg" | "ico" | "avif" => {
            Some(pastas.imagens)
        }

        // Musicas (Áudio):
        // mp3, wav, flac, aac, ogg, m4a, wma, opus, mid
        "mp3" | "wav" | "flac" | "aac" | "ogg" | "m4a" | "wma" | "opus" | "mid" => {
            Some(pastas.musicas)
        }

        // Videos:
        // mp4, mkv, avi, mov, wmv, flv, webm, m4v
        "mp4" | "mkv" | "avi" | "mov" | "wmv" | "flv" | "webm" | "m4v" => {
            Some(pastas.videos)
        }

        _ => None, // Ignora arquivos não mapeados
    }
}

/// Função auxiliar segura para mover arquivos:
/// - Cria o diretório de destino sob demanda (lazy) apenas antes de mover o arquivo.
/// - Previne sobrescrita anexando timestamp UNIX se o arquivo de destino já existir.
/// - Suporta partições distintas/cross-device (fallback de fs::rename para fs::copy + fs::remove_file).
fn mover_arquivo_seguro(origem: &Path, destino: &Path) -> std::io::Result<()> {
    let file_name = match origem.file_name() {
        Some(name) => name,
        None => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Nome de arquivo inválido",
            ))
        }
    };

    let mut dest_path = destino.join(file_name);

    if dest_path.exists() {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let stem = origem
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("arquivo");
        let ext = origem.extension().and_then(|e| e.to_str());

        let mut novo_nome = match ext {
            Some(e) if !e.is_empty() => format!("{}_{}.{}", stem, timestamp, e),
            _ => format!("{}_{}", stem, timestamp),
        };

        dest_path = destino.join(&novo_nome);

        if dest_path.exists() {
            let millis = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis();
            novo_nome = match ext {
                Some(e) if !e.is_empty() => format!("{}_{}.{}", stem, millis, e),
                _ => format!("{}_{}", stem, millis),
            };
            dest_path = destino.join(&novo_nome);
        }
    }

    if origem == dest_path {
        return Ok(());
    }

    // Criação lazy do diretório de destino estritamente antes de mover
    if let Some(pai) = dest_path.parent() {
        fs::create_dir_all(pai)?;
    }

    // Tenta mover atomicamente com rename
    if fs::rename(origem, &dest_path).is_err() {
        // Fallback cross-device (erro EXDEV): copia e em seguida remove o original
        fs::copy(origem, &dest_path)?;
        fs::remove_file(origem)?;
    }

    Ok(())
}

#[tauri::command]
pub fn organizar_downloads(app: AppHandle) {
    // Avisa que começou
    disparar_notificacao(&app, "Iniciando organização de arquivos...", "normal");

    // Joga pra background thread para não travar a bandeja!
    std::thread::spawn(move || {
        let mut movidos = 0;

        // Pega os caminhos padrão do usuário
        if let (
            Some(downloads_dir),
            Some(docs_dir),
            Some(musics_dir),
            Some(pics_dir),
            Some(videos_dir),
        ) = (
            dirs::download_dir().or_else(|| dirs::home_dir().map(|h| h.join("Downloads"))),
            dirs::document_dir().or_else(|| dirs::home_dir().map(|h| h.join("Documents"))),
            dirs::audio_dir().or_else(|| dirs::home_dir().map(|h| h.join("Music"))),
            dirs::picture_dir().or_else(|| dirs::home_dir().map(|h| h.join("Pictures"))),
            dirs::video_dir().or_else(|| dirs::home_dir().map(|h| h.join("Videos"))),
        ) {
            let instaladores_dir = downloads_dir.join("Instaladores");
            let compactados_dir = downloads_dir.join("Compactados");
            let fontes_dir = downloads_dir.join("Fontes");

            let emulacao_dir = docs_dir.join("Emulacao");
            let emulacao_saves_dir = emulacao_dir.join("Saves e Cheats");
            let emulacao_roms_dir = emulacao_dir.join("ROMs");
            let emulacao_roms_nds_dir = emulacao_roms_dir.join("NDS");
            let emulacao_roms_3ds_dir = emulacao_roms_dir.join("3DS");
            let emulacao_roms_gba_dir = emulacao_roms_dir.join("GBA");
            let emulacao_roms_snes_dir = emulacao_roms_dir.join("SNES");
            let emulacao_roms_nes_dir = emulacao_roms_dir.join("NES");
            let emulacao_roms_ps1_dir = emulacao_roms_dir.join("PS1");
            let emulacao_roms_ps2_dir = emulacao_roms_dir.join("PS2");
            let emulacao_roms_psp_dir = emulacao_roms_dir.join("PSP");

            let livros_hqs_dir = docs_dir.join("Livros e HQs");
            let codigo_dir = docs_dir.join("Codigo");

            let design_3d_dir = pics_dir.join("Design e 3D");

            let pastas = PastasDestino {
                instaladores: &instaladores_dir,
                compactados: &compactados_dir,
                fontes: &fontes_dir,
                emulacao_saves: &emulacao_saves_dir,
                emulacao_roms_nds: &emulacao_roms_nds_dir,
                emulacao_roms_3ds: &emulacao_roms_3ds_dir,
                emulacao_roms_gba: &emulacao_roms_gba_dir,
                emulacao_roms_snes: &emulacao_roms_snes_dir,
                emulacao_roms_nes: &emulacao_roms_nes_dir,
                emulacao_roms_ps1: &emulacao_roms_ps1_dir,
                emulacao_roms_ps2: &emulacao_roms_ps2_dir,
                emulacao_roms_psp: &emulacao_roms_psp_dir,
                livros_hqs: &livros_hqs_dir,
                codigo: &codigo_dir,
                documentos: &docs_dir,
                design_3d: &design_3d_dir,
                imagens: &pics_dir,
                musicas: &musics_dir,
                videos: &videos_dir,
            };

            // Lê a pasta de downloads
            if let Ok(entradas) = fs::read_dir(&downloads_dir) {
                for entrada in entradas.flatten() {
                    let path = entrada.path();

                    if path.is_file() {
                        if let Some(dest_dir) = classificar_arquivo(&path, &pastas) {
                            if mover_arquivo_seguro(&path, dest_dir).is_ok() {
                                movidos += 1;
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
            let msg = format!("Mágica feita!\n{} arquivos organizados.", movidos);
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
            format!(
                "Atenção! Faltam {} minutos e meio para:\n{}",
                inteira, mensagem
            )
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

#[tauri::command]
pub fn esconder_pergunta(app: tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("pergunta") {
        let _ = window.hide();
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, Default)]
pub struct AppConfig {
    pub user_name: String,
    pub gemini_api_key: String,
}

fn obter_caminho_config() -> Option<std::path::PathBuf> {
    dirs::config_dir().map(|p| p.join("MiiSideKick").join("config.json"))
}

#[tauri::command]
pub fn carregar_configuracoes() -> AppConfig {
    if let Some(caminho) = obter_caminho_config() {
        if caminho.exists() {
            if let Ok(conteudo) = fs::read_to_string(&caminho) {
                if let Ok(cfg) = serde_json::from_str::<AppConfig>(&conteudo) {
                    return cfg;
                }
            }
        }
    }

    let user_name = std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "Usuário".to_string());

    AppConfig {
        user_name,
        gemini_api_key: String::new(),
    }
}

#[tauri::command]
pub fn salvar_configuracoes(config: AppConfig) -> Result<(), String> {
    if let Some(caminho) = obter_caminho_config() {
        if let Some(pai) = caminho.parent() {
            let _ = fs::create_dir_all(pai);
        }
        let json = serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?;
        fs::write(&caminho, json).map_err(|e| e.to_string())?;
        Ok(())
    } else {
        Err("Não foi possível acessar a pasta de configurações".to_string())
    }
}

#[derive(serde::Deserialize)]
struct GeminiResposta {
    reacao: Option<String>,
    resposta: Option<String>,
}

#[derive(serde::Deserialize)]
struct GeminiCandidatePart {
    text: Option<String>,
}

#[derive(serde::Deserialize)]
struct GeminiCandidateContent {
    parts: Option<Vec<GeminiCandidatePart>>,
}

#[derive(serde::Deserialize)]
struct GeminiCandidate {
    content: Option<GeminiCandidateContent>,
}

#[derive(serde::Deserialize)]
struct GeminiResponse {
    candidates: Option<Vec<GeminiCandidate>>,
}

#[tauri::command]
pub fn perguntar_ao_mii(app: AppHandle, pergunta: String) {
    let app_clone = app.clone();

    std::thread::spawn(move || {
        // Disparo imediato do aviso inicial
        disparar_notificacao(&app_clone, "Pensando...", "normal");

        // Carrega configurações para obter chave e nome
        let config = carregar_configuracoes();
        let api_key = config.gemini_api_key.trim();

        if api_key.is_empty() {
            disparar_notificacao(
                &app_clone,
                "Cadê minha chave de API? Configure no Setup!",
                "frustrado",
            );
            return;
        }

        let user_name = if config.user_name.trim().is_empty() {
            "Usuário".to_string()
        } else {
            config.user_name.trim().to_string()
        };

        let system_instruction = format!(
            "Você é o MiiSideKick, um companheiro de desktop com personalidade expressiva. O nome do usuário é: {}. Você pode usar o nome dele na resposta se fizer sentido, mas não é obrigatório. REGRAS OBRIGATÓRIAS:\n\
            1. Espelhe o tom do usuário: se ele for grosseiro, sarcástico ou agressivo, responda no mesmo nível de deboche e grosseria; se for educado, seja educado e prestativo; se for engraçado, seja engraçado.\n\
            2. Tamanho máximo estrito: sua resposta DEVE ter no máximo 70 caracteres. Seja direto e conciso.\n\
            3. Retorne ESTRITAMENTE um JSON válido no seguinte formato, sem blocos de código markdown: {{\"reacao\": \"normal\" | \"tranquilo\" | \"frustrado\", \"resposta\": \"texto de ate 70 chars\"}}",
            user_name
        );

        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent?key={}",
            api_key
        );

        let payload = serde_json::json!({
            "contents": [
                {
                    "role": "user",
                    "parts": [
                        { "text": pergunta }
                    ]
                }
            ],
            "systemInstruction": {
                "parts": [
                    { "text": system_instruction }
                ]
            },
            "generationConfig": {
                "responseMimeType": "application/json"
            },
            "safetySettings": [
                { "category": "HARM_CATEGORY_HARASSMENT", "threshold": "BLOCK_NONE" },
                { "category": "HARM_CATEGORY_HATE_SPEECH", "threshold": "BLOCK_NONE" },
                { "category": "HARM_CATEGORY_SEXUALLY_EXPLICIT", "threshold": "BLOCK_NONE" },
                { "category": "HARM_CATEGORY_DANGEROUS_CONTENT", "threshold": "BLOCK_NONE" }
            ]
        });

        let client = match reqwest::blocking::Client::builder()
            .user_agent("MiiSideKick/1.0")
            .timeout(std::time::Duration::from_secs(15))
            .build()
        {
            Ok(c) => c,
            Err(e) => {
                eprintln!("[Gemini API Falha de Rede]: {:?}", e);
                disparar_notificacao(&app_clone, "Falha de conexão com a API", "frustrado");
                return;
            }
        };

        eprintln!("[Gemini API]: Enviando pergunta: {:?}", pergunta);

        let response = match client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
        {
            Ok(r) => r,
            Err(e) => {
                eprintln!("[Gemini API Falha de Rede]: {:?}", e);
                disparar_notificacao(&app_clone, "Falha de conexão com a API", "frustrado");
                return;
            }
        };

        if !response.status().is_success() {
            let status = response.status();
            let erro_body = response.text().unwrap_or_default();
            eprintln!("[Gemini API Erro {}]: {}", status, erro_body);
            let msg = format!("Erro Gemini ({})", status);
            disparar_notificacao(&app_clone, &msg, "frustrado");
            return;
        }

        let gemini_resp: GeminiResponse = match response.json() {
            Ok(parsed) => parsed,
            Err(e) => {
                eprintln!("[Gemini API]: Erro ao decodificar JSON da API: {:?}", e);
                disparar_notificacao(&app_clone, "Sem resposta da IA...", "frustrado");
                return;
            }
        };

        let raw_text = gemini_resp
            .candidates
            .as_ref()
            .and_then(|c| c.first())
            .and_then(|c| c.content.as_ref())
            .and_then(|c| c.parts.as_ref())
            .and_then(|p| p.first())
            .and_then(|p| p.text.as_ref())
            .map(|s| s.trim())
            .unwrap_or("");

        if raw_text.is_empty() {
            eprintln!("[Gemini API]: Resposta vazia ou bloqueada");
            disparar_notificacao(&app_clone, "Sem resposta da IA...", "frustrado");
            return;
        }

        eprintln!("[Gemini API]: Resposta bruta recebida: {:?}", raw_text);

        // Remove eventuais blocos de código markdown ```json ... ```
        let clean_json = if raw_text.starts_with("```json") {
            raw_text.trim_start_matches("```json").trim_end_matches("```").trim()
        } else if raw_text.starts_with("```") {
            raw_text.trim_start_matches("```").trim_end_matches("```").trim()
        } else {
            raw_text
        };

        let gemini_resultado: GeminiResposta = match serde_json::from_str(clean_json) {
            Ok(res) => res,
            Err(e) => {
                eprintln!("[Gemini API]: Não foi possível parsear resposta como GeminiResposta ({:?}). Usando texto direto.", e);
                GeminiResposta {
                    reacao: Some("normal".to_string()),
                    resposta: Some(raw_text.to_string()),
                }
            }
        };

        let reacao = match gemini_resultado.reacao.as_deref() {
            Some("frustrado") => "frustrado",
            Some("tranquilo") => "tranquilo",
            Some("assustado") => "assustado",
            _ => "normal",
        };

        let mut resposta = gemini_resultado
            .resposta
            .unwrap_or_else(|| "Hmm, não sei o que dizer...".to_string());

        if resposta.chars().count() > 70 {
            resposta = resposta.chars().take(67).collect::<String>() + "...";
        }

        eprintln!("[Gemini API]: Notificação disparada com humor '{}': {}", reacao, resposta);
        disparar_notificacao(&app_clone, &resposta, reacao);
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_mover_arquivo_com_timestamp() {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let tmp_dir = std::env::temp_dir().join(format!("miisidekick_test_{}", unique));
        let orig_dir = tmp_dir.join("orig");
        let dest_dir = tmp_dir.join("dest");
        fs::create_dir_all(&orig_dir).unwrap();
        fs::create_dir_all(&dest_dir).unwrap();

        let orig_file = orig_dir.join("teste.txt");
        fs::write(&orig_file, "conteudo 1").unwrap();

        // 1. Move normal quando o arquivo não existe no destino
        mover_arquivo_seguro(&orig_file, &dest_dir).unwrap();
        assert!(!orig_file.exists());
        assert!(dest_dir.join("teste.txt").exists());

        // 2. Cria arquivo com mesmo nome na origem para testar prevenção de sobrescrita
        fs::write(&orig_file, "conteudo 2").unwrap();
        mover_arquivo_seguro(&orig_file, &dest_dir).unwrap();
        assert!(!orig_file.exists());

        // Deve existir o original "teste.txt" e um novo com timestamp "teste_<ts>.txt"
        let files: Vec<_> = fs::read_dir(&dest_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();
        assert_eq!(files.len(), 2);
        assert!(files.iter().any(|f| {
            let name = f.file_name().to_string_lossy().to_string();
            name.starts_with("teste_") && name.ends_with(".txt")
        }));

        let _ = fs::remove_dir_all(&tmp_dir);
    }

    #[test]
    fn test_mover_arquivo_criacao_lazy() {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let tmp_dir = std::env::temp_dir().join(format!("miisidekick_lazy_test_{}", unique));
        let orig_dir = tmp_dir.join("orig");
        let dest_dir = tmp_dir.join("ROMs").join("NDS");
        let uncreated_dir = tmp_dir.join("ROMs").join("PS1");
        fs::create_dir_all(&orig_dir).unwrap();

        let orig_file = orig_dir.join("pokemon.nds");
        fs::write(&orig_file, "dados nds").unwrap();

        // Antes de mover, o diretório de destino não deve existir (lazy)
        assert!(!dest_dir.exists());
        assert!(!uncreated_dir.exists());

        // Ao mover, o diretório de destino deve ser criado sob demanda
        mover_arquivo_seguro(&orig_file, &dest_dir).unwrap();
        assert!(dest_dir.exists());
        assert!(dest_dir.join("pokemon.nds").exists());
        assert!(!orig_file.exists());

        // Diretórios para os quais nenhum arquivo foi movido continuam não existindo
        assert!(!uncreated_dir.exists());

        let _ = fs::remove_dir_all(&tmp_dir);
    }

    #[test]
    fn test_classificacao_todas_as_extensoes() {
        let p_inst = Path::new("/downloads/Instaladores");
        let p_comp = Path::new("/downloads/Compactados");
        let p_font = Path::new("/downloads/Fontes");
        let p_saves = Path::new("/docs/Emulacao/Saves e Cheats");
        let p_roms_nds = Path::new("/docs/Emulacao/ROMs/NDS");
        let p_roms_3ds = Path::new("/docs/Emulacao/ROMs/3DS");
        let p_roms_gba = Path::new("/docs/Emulacao/ROMs/GBA");
        let p_roms_snes = Path::new("/docs/Emulacao/ROMs/SNES");
        let p_roms_nes = Path::new("/docs/Emulacao/ROMs/NES");
        let p_roms_ps1 = Path::new("/docs/Emulacao/ROMs/PS1");
        let p_roms_ps2 = Path::new("/docs/Emulacao/ROMs/PS2");
        let p_roms_psp = Path::new("/docs/Emulacao/ROMs/PSP");
        let p_livros = Path::new("/docs/Livros e HQs");
        let p_codigo = Path::new("/docs/Codigo");
        let p_docs = Path::new("/docs");
        let p_d3d = Path::new("/pics/Design e 3D");
        let p_pics = Path::new("/pics");
        let p_mus = Path::new("/music");
        let p_vid = Path::new("/videos");

        let pastas = PastasDestino {
            instaladores: p_inst,
            compactados: p_comp,
            fontes: p_font,
            emulacao_saves: p_saves,
            emulacao_roms_nds: p_roms_nds,
            emulacao_roms_3ds: p_roms_3ds,
            emulacao_roms_gba: p_roms_gba,
            emulacao_roms_snes: p_roms_snes,
            emulacao_roms_nes: p_roms_nes,
            emulacao_roms_ps1: p_roms_ps1,
            emulacao_roms_ps2: p_roms_ps2,
            emulacao_roms_psp: p_roms_psp,
            livros_hqs: p_livros,
            codigo: p_codigo,
            documentos: p_docs,
            design_3d: p_d3d,
            imagens: p_pics,
            musicas: p_mus,
            videos: p_vid,
        };

        // Downloads/Instaladores:
        // Linux: deb, appimage, snap, flatpakref, rpm, bin (sem .cue), run
        // Windows, Mac e Mobile: exe, msi, dmg, pkg, apk, xapk
        for ext in &[
            "deb", "appimage", "snap", "flatpakref", "rpm", "bin", "run", "exe", "msi", "dmg",
            "pkg", "apk", "xapk",
        ] {
            let f = PathBuf::from(format!("arquivo.{}", ext));
            assert_eq!(
                classificar_arquivo(&f, &pastas),
                Some(p_inst),
                "Falhou na extensão instalador: {}",
                ext
            );
        }

        // Downloads/Compactados:
        // zip, rar, 7z, tar, gz, bz2, xz, zst, tgz
        for ext in &[
            "zip", "rar", "7z", "tar", "gz", "bz2", "xz", "zst", "tgz",
        ] {
            let f = PathBuf::from(format!("arquivo.{}", ext));
            assert_eq!(
                classificar_arquivo(&f, &pastas),
                Some(p_comp),
                "Falhou na extensão compactado: {}",
                ext
            );
        }

        // Downloads/Fontes:
        // ttf, otf, woff, woff2
        for ext in &["ttf", "otf", "woff", "woff2"] {
            let f = PathBuf::from(format!("arquivo.{}", ext));
            assert_eq!(
                classificar_arquivo(&f, &pastas),
                Some(p_font),
                "Falhou na extensão fonte: {}",
                ext
            );
        }

        // Documentos/Emulacao/Saves e Cheats:
        // cbs, max, psu, xps, pnach, srm, mcd, ps2
        for ext in &["cbs", "max", "psu", "xps", "pnach", "srm", "mcd", "ps2"] {
            let f = PathBuf::from(format!("arquivo.{}", ext));
            assert_eq!(
                classificar_arquivo(&f, &pastas),
                Some(p_saves),
                "Falhou na extensão save/cheat: {}",
                ext
            );
        }

        // Documentos/Emulacao/ROMs/NDS:
        // nds
        for ext in &["nds"] {
            let f = PathBuf::from(format!("arquivo.{}", ext));
            assert_eq!(
                classificar_arquivo(&f, &pastas),
                Some(p_roms_nds),
                "Falhou na extensão NDS: {}",
                ext
            );
        }

        // Documentos/Emulacao/ROMs/3DS:
        // 3ds, cia
        for ext in &["3ds", "cia"] {
            let f = PathBuf::from(format!("arquivo.{}", ext));
            assert_eq!(
                classificar_arquivo(&f, &pastas),
                Some(p_roms_3ds),
                "Falhou na extensão 3DS: {}",
                ext
            );
        }

        // Documentos/Emulacao/ROMs/GBA:
        // gba, gbc, gb
        for ext in &["gba", "gbc", "gb"] {
            let f = PathBuf::from(format!("arquivo.{}", ext));
            assert_eq!(
                classificar_arquivo(&f, &pastas),
                Some(p_roms_gba),
                "Falhou na extensão GBA: {}",
                ext
            );
        }

        // Documentos/Emulacao/ROMs/SNES:
        // smc, sfc
        for ext in &["smc", "sfc"] {
            let f = PathBuf::from(format!("arquivo.{}", ext));
            assert_eq!(
                classificar_arquivo(&f, &pastas),
                Some(p_roms_snes),
                "Falhou na extensão SNES: {}",
                ext
            );
        }

        // Documentos/Emulacao/ROMs/NES:
        // nes
        for ext in &["nes"] {
            let f = PathBuf::from(format!("arquivo.{}", ext));
            assert_eq!(
                classificar_arquivo(&f, &pastas),
                Some(p_roms_nes),
                "Falhou na extensão NES: {}",
                ext
            );
        }

        // Documentos/Emulacao/ROMs/PS1:
        // cue, pbp
        for ext in &["cue", "pbp"] {
            let f = PathBuf::from(format!("arquivo.{}", ext));
            assert_eq!(
                classificar_arquivo(&f, &pastas),
                Some(p_roms_ps1),
                "Falhou na extensão PS1: {}",
                ext
            );
        }

        // Documentos/Emulacao/ROMs/PS2:
        // iso, img, chd
        for ext in &["iso", "img", "chd"] {
            let f = PathBuf::from(format!("arquivo.{}", ext));
            assert_eq!(
                classificar_arquivo(&f, &pastas),
                Some(p_roms_ps2),
                "Falhou na extensão PS2: {}",
                ext
            );
        }

        // Documentos/Emulacao/ROMs/PSP:
        // cso
        for ext in &["cso"] {
            let f = PathBuf::from(format!("arquivo.{}", ext));
            assert_eq!(
                classificar_arquivo(&f, &pastas),
                Some(p_roms_psp),
                "Falhou na extensão PSP: {}",
                ext
            );
        }

        // Documentos/Livros e HQs:
        // epub, mobi, azw, azw3, fb2, cbr, cbz, cbt, cb7
        for ext in &[
            "epub", "mobi", "azw", "azw3", "fb2", "cbr", "cbz", "cbt", "cb7",
        ] {
            let f = PathBuf::from(format!("arquivo.{}", ext));
            assert_eq!(
                classificar_arquivo(&f, &pastas),
                Some(p_livros),
                "Falhou na extensão livro/hq: {}",
                ext
            );
        }

        // Documentos/Codigo:
        // Código e Scripts: py, js, mjs, jsx, tsx, rs, c, cpp, h, hpp, cs, java, go, php, rb, lua, sh, bash
        // Configurações e Dados: json, yaml, yml, toml, xml, sql, env
        // Documentação e Web: md, markdown, tex, rst, html, css
        for ext in &[
            "py", "js", "mjs", "jsx", "tsx", "rs", "c", "cpp", "h", "hpp", "cs", "java", "go",
            "php", "rb", "lua", "sh", "bash", "json", "yaml", "yml", "toml", "xml", "sql", "env",
            "md", "markdown", "tex", "rst", "html", "css",
        ] {
            let f = PathBuf::from(format!("arquivo.{}", ext));
            assert_eq!(
                classificar_arquivo(&f, &pastas),
                Some(p_codigo),
                "Falhou na extensão codigo: {}",
                ext
            );
        }

        // Documentos (Escritório e Financeiro):
        // Textos: pdf, doc, docx, odt, rtf, txt
        // Planilhas e Financeiro: xlsx, xls, ods, csv, tsv, ofx
        // Apresentações: pptx, ppt, odp, key
        for ext in &[
            "pdf", "doc", "docx", "odt", "rtf", "txt", "xlsx", "xls", "ods", "csv", "tsv",
            "pptx", "ppt", "odp", "key", "ofx",
        ] {
            let f = PathBuf::from(format!("arquivo.{}", ext));
            assert_eq!(
                classificar_arquivo(&f, &pastas),
                Some(p_docs),
                "Falhou na extensão documento: {}",
                ext
            );
        }

        // Imagens/Design e 3D:
        // psd, ai, xcf, blend, obj, stl, fbx
        for ext in &["psd", "ai", "xcf", "blend", "obj", "stl", "fbx"] {
            let f = PathBuf::from(format!("arquivo.{}", ext));
            assert_eq!(
                classificar_arquivo(&f, &pastas),
                Some(p_d3d),
                "Falhou na extensão design 3D: {}",
                ext
            );
        }

        // Imagens:
        // jpg, jpeg, png, gif, bmp, tiff, webp, svg, ico, avif
        for ext in &[
            "jpg", "jpeg", "png", "gif", "bmp", "tiff", "webp", "svg", "ico", "avif",
        ] {
            let f = PathBuf::from(format!("arquivo.{}", ext));
            assert_eq!(
                classificar_arquivo(&f, &pastas),
                Some(p_pics),
                "Falhou na extensão imagem: {}",
                ext
            );
        }

        // Musicas:
        // mp3, wav, flac, aac, ogg, m4a, wma, opus, mid
        for ext in &[
            "mp3", "wav", "flac", "aac", "ogg", "m4a", "wma", "opus", "mid",
        ] {
            let f = PathBuf::from(format!("arquivo.{}", ext));
            assert_eq!(
                classificar_arquivo(&f, &pastas),
                Some(p_mus),
                "Falhou na extensão musica: {}",
                ext
            );
        }

        // Videos:
        // mp4, mkv, avi, mov, wmv, flv, webm, m4v
        for ext in &["mp4", "mkv", "avi", "mov", "wmv", "flv", "webm", "m4v"] {
            let f = PathBuf::from(format!("arquivo.{}", ext));
            assert_eq!(
                classificar_arquivo(&f, &pastas),
                Some(p_vid),
                "Falhou na extensão video: {}",
                ext
            );
        }
    }

    #[test]
    fn test_classificacao_bin_com_e_sem_cue() {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let tmp_dir = std::env::temp_dir().join(format!("miisidekick_cue_test_{}", unique));
        let downloads = tmp_dir.join("Downloads");
        let ps1_dir = tmp_dir.join("PS1");
        let inst_dir = tmp_dir.join("Instaladores");
        fs::create_dir_all(&downloads).unwrap();
        fs::create_dir_all(&ps1_dir).unwrap();
        fs::create_dir_all(&inst_dir).unwrap();

        let dummy_dir = tmp_dir.join("dummy");
        let pastas = PastasDestino {
            instaladores: &inst_dir,
            compactados: &dummy_dir,
            fontes: &dummy_dir,
            emulacao_saves: &dummy_dir,
            emulacao_roms_nds: &dummy_dir,
            emulacao_roms_3ds: &dummy_dir,
            emulacao_roms_gba: &dummy_dir,
            emulacao_roms_snes: &dummy_dir,
            emulacao_roms_nes: &dummy_dir,
            emulacao_roms_ps1: &ps1_dir,
            emulacao_roms_ps2: &dummy_dir,
            emulacao_roms_psp: &dummy_dir,
            livros_hqs: &dummy_dir,
            codigo: &dummy_dir,
            documentos: &dummy_dir,
            design_3d: &dummy_dir,
            imagens: &dummy_dir,
            musicas: &dummy_dir,
            videos: &dummy_dir,
        };

        // 1. Arquivo .bin sem .cue correspondente deve ser classificado como instalador
        let bin_sem_cue = downloads.join("setup.bin");
        fs::write(&bin_sem_cue, "conteudo executavel").unwrap();
        assert_eq!(classificar_arquivo(&bin_sem_cue, &pastas), Some(&inst_dir as &Path));

        // 2. Arquivo .bin com .cue de mesmo nome na mesma pasta deve ir para PS1
        let bin_com_cue = downloads.join("jogo.bin");
        let cue_mesmo_nome = downloads.join("jogo.cue");
        fs::write(&bin_com_cue, "dados de jogo").unwrap();
        fs::write(&cue_mesmo_nome, "FILE \"jogo.bin\" BINARY").unwrap();
        assert_eq!(classificar_arquivo(&bin_com_cue, &pastas), Some(&ps1_dir as &Path));

        // 3. Arquivo .bin cujo .cue correspondente já foi movido para a pasta de destino PS1
        let bin_movido = downloads.join("outro_jogo.bin");
        let cue_no_destino = ps1_dir.join("outro_jogo.cue");
        fs::write(&bin_movido, "dados").unwrap();
        fs::write(&cue_no_destino, "FILE \"outro_jogo.bin\" BINARY").unwrap();
        assert_eq!(classificar_arquivo(&bin_movido, &pastas), Some(&ps1_dir as &Path));

        // 4. Arquivo .bin multi-faixa referenciado dentro de um .cue
        let bin_track = downloads.join("multi (Track 1).bin");
        let cue_multi = downloads.join("multi.cue");
        fs::write(&bin_track, "dados faixa 1").unwrap();
        fs::write(&cue_multi, "FILE \"multi (Track 1).bin\" BINARY\n  TRACK 01 MODE2/2352\n    INDEX 01 00:00:00").unwrap();
        assert_eq!(classificar_arquivo(&bin_track, &pastas), Some(&ps1_dir as &Path));

        // Limpeza
        let _ = fs::remove_dir_all(&tmp_dir);
    }

    #[test]
    fn test_app_config_serializacao() {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let tmp_dir = std::env::temp_dir().join(format!("miisidekick_cfg_test_{}", unique));
        let _ = fs::create_dir_all(&tmp_dir);
        let cfg_file = tmp_dir.join("config.json");

        let config = AppConfig {
            user_name: "Alan".to_string(),
            gemini_api_key: "AIzaSyTest123".to_string(),
        };

        let json = serde_json::to_string_pretty(&config).unwrap();
        fs::write(&cfg_file, json).unwrap();

        let lida: AppConfig = serde_json::from_str(&fs::read_to_string(&cfg_file).unwrap()).unwrap();
        assert_eq!(lida.user_name, "Alan");
        assert_eq!(lida.gemini_api_key, "AIzaSyTest123");

        let _ = fs::remove_dir_all(&tmp_dir);
    }

    #[test]
    fn test_gemini_resposta_parsing() {
        let json_valido = r#"{"reacao": "tranquilo", "resposta": "E aí Alan! Tudo certo?"}"#;
        let res: GeminiResposta = serde_json::from_str(json_valido).unwrap();
        assert_eq!(res.reacao.as_deref(), Some("tranquilo"));
        assert_eq!(res.resposta.as_deref(), Some("E aí Alan! Tudo certo?"));

        let json_com_bloco = "```json\n{\"reacao\": \"frustrado\", \"resposta\": \"Nem te conto!\"}\n```";
        let limpo = json_com_bloco.trim().trim_start_matches("```json").trim_end_matches("```").trim();
        let res2: GeminiResposta = serde_json::from_str(limpo).unwrap();
        assert_eq!(res2.reacao.as_deref(), Some("frustrado"));
        assert_eq!(res2.resposta.as_deref(), Some("Nem te conto!"));
    }

    #[test]
    fn test_gemini_payload_estrutura() {
        let pergunta = "Qual é a boa?";
        let system_instruction = "Você é o MiiSideKick.";
        let payload = serde_json::json!({
            "contents": [
                {
                    "role": "user",
                    "parts": [
                        { "text": pergunta }
                    ]
                }
            ],
            "systemInstruction": {
                "parts": [
                    { "text": system_instruction }
                ]
            },
            "generationConfig": {
                "responseMimeType": "application/json"
            },
            "safetySettings": [
                { "category": "HARM_CATEGORY_HARASSMENT", "threshold": "BLOCK_NONE" },
                { "category": "HARM_CATEGORY_HATE_SPEECH", "threshold": "BLOCK_NONE" },
                { "category": "HARM_CATEGORY_SEXUALLY_EXPLICIT", "threshold": "BLOCK_NONE" },
                { "category": "HARM_CATEGORY_DANGEROUS_CONTENT", "threshold": "BLOCK_NONE" }
            ]
        });

        assert_eq!(payload["contents"][0]["role"], "user");
        assert_eq!(payload["contents"][0]["parts"][0]["text"], "Qual é a boa?");
        assert_eq!(payload["systemInstruction"]["parts"][0]["text"], "Você é o MiiSideKick.");
        assert_eq!(payload["generationConfig"]["responseMimeType"], "application/json");
        let safety = payload["safetySettings"].as_array().unwrap();
        assert_eq!(safety.len(), 4);
        assert!(safety.iter().all(|s| s["threshold"] == "BLOCK_NONE"));
    }
}

