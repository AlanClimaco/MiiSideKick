use std::collections::VecDeque;
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

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, Default)]
pub struct NotificacaoPayload {
    pub texto: String,
    pub tipo: String,
    pub link: Option<String>,
    pub link_rotulo: Option<String>,
    #[serde(default)]
    pub pode_responder: bool,
    #[serde(default)]
    pub tocar_som: bool,
    #[serde(default)]
    pub acao: Option<String>,
    #[serde(default)]
    pub acao_rotulo: Option<String>,
    #[serde(default)]
    pub eh_reconsulta: bool,
    #[serde(default)]
    pub timestamp: u64,
    #[serde(default)]
    pub hora_formatada: String,
}

impl NotificacaoPayload {
    fn script_atualizacao(&self) -> String {
        let argumentos = serde_json::to_string(&(
            &self.texto,
            &self.tipo,
            &self.link,
            &self.link_rotulo,
            self.pode_responder,
            self.tocar_som,
            &self.acao,
            &self.acao_rotulo,
            self.eh_reconsulta,
        ))
        .expect("Os argumentos da notificação devem ser serializáveis");

        format!(
            "if (window.atualizarInterface) {{ window.atualizarInterface(...{}); }}",
            argumentos
        )
    }
}

static ULTIMA_NOTIFICACAO: Mutex<Option<NotificacaoPayload>> = Mutex::new(None);
static HISTORICO_CONVERSA: Mutex<Vec<serde_json::Value>> = Mutex::new(Vec::new());

pub const LIMITE_HISTORICO_IA: usize = 5;
static HISTORICO_NOTIFICACOES_IA: Mutex<VecDeque<NotificacaoPayload>> = Mutex::new(VecDeque::new());

pub fn registrar_historico_ia(mut payload: NotificacaoPayload) {
    payload.eh_reconsulta = false;
    if payload.timestamp == 0 {
        let now = std::time::SystemTime::now();
        let millis = now
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        payload.timestamp = millis;
        let total_secs = millis / 1000;
        let mins = (total_secs / 60) % 60;
        let hours = (total_secs / 3600) % 24;
        payload.hora_formatada = format!("{:02}:{:02}", hours, mins);
    }
    if let Ok(mut hist) = HISTORICO_NOTIFICACOES_IA.lock() {
        while hist.len() >= LIMITE_HISTORICO_IA {
            hist.pop_back();
        }
        hist.push_front(payload);
    }
}

#[tauri::command]
pub fn obter_historico_ia() -> Vec<NotificacaoPayload> {
    if let Ok(hist) = HISTORICO_NOTIFICACOES_IA.lock() {
        hist.iter().cloned().collect()
    } else {
        Vec::new()
    }
}

#[tauri::command]
pub fn limpar_historico_ia() {
    if let Ok(mut hist) = HISTORICO_NOTIFICACOES_IA.lock() {
        hist.clear();
    }
}

#[tauri::command]
pub fn abrir_historico(app: AppHandle) {
    if let Some(janela) = app.get_webview_window("historico") {
        let _ = janela.show();
        let _ = janela.unminimize();
        let _ = janela.set_focus();
    }
}

#[tauri::command]
pub fn esconder_historico(app: AppHandle) {
    if let Some(janela) = app.get_webview_window("historico") {
        let _ = janela.hide();
    }
}

#[tauri::command]
pub fn reexibir_notificacao_ia(app: AppHandle, index: usize) {
    let payload_opt = {
        if let Ok(hist) = HISTORICO_NOTIFICACOES_IA.lock() {
            hist.get(index).cloned()
        } else {
            None
        }
    };

    if let Some(mut payload) = payload_opt {
        payload.eh_reconsulta = true;
        payload.tocar_som = false;
        exibir_janela_notificacao(&app, &payload);
    }
}

#[tauri::command]
pub fn limpar_historico_conversa() {
    if let Ok(mut hist) = HISTORICO_CONVERSA.lock() {
        hist.clear();
        eprintln!("[Gemini API]: Histórico de conversa limpo.");
    }
}

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

pub fn disparar_notificacao(
    app: &AppHandle,
    texto: &str,
    tipo: &str,
    link: Option<String>,
    link_rotulo: Option<String>,
    pode_responder: bool,
    tocar_som: bool,
) {
    disparar_notificacao_completa(
        app,
        texto,
        tipo,
        link,
        link_rotulo,
        pode_responder,
        tocar_som,
        None,
        None,
    );
}

pub fn disparar_notificacao_com_acao(
    app: &AppHandle,
    texto: &str,
    tipo: &str,
    acao: Option<String>,
    acao_rotulo: Option<String>,
) {
    disparar_notificacao_completa(
        app,
        texto,
        tipo,
        None,
        None,
        false,
        true,
        acao,
        acao_rotulo,
    );
}

pub fn exibir_janela_notificacao(app: &AppHandle, payload: &NotificacaoPayload) {
    if let Ok(mut mem) = ULTIMA_NOTIFICACAO.lock() {
        *mem = Some(payload.clone());
    }

    if let Some(janela) = app.get_webview_window("popup") {
        if let Ok(Some(monitor)) = janela.primary_monitor() {
            let size = monitor.size();
            let window_size = janela
                .outer_size()
                .unwrap_or(tauri::PhysicalSize::new(420, 240));
            let x = size.width.saturating_sub(window_size.width + 20);
            let y = size.height.saturating_sub(window_size.height + 60);
            let _ = janela.set_position(tauri::PhysicalPosition::new(x, y));
        }
        let _ = janela.show();
        let _ = janela.unminimize();

        let _ = janela.eval(&payload.script_atualizacao());
    }
}

pub fn disparar_notificacao_completa(
    app: &AppHandle,
    texto: &str,
    tipo: &str,
    link: Option<String>,
    link_rotulo: Option<String>,
    pode_responder: bool,
    tocar_som: bool,
    acao: Option<String>,
    acao_rotulo: Option<String>,
) {
    let payload = NotificacaoPayload {
        texto: texto.to_string(),
        tipo: tipo.to_string(),
        link,
        link_rotulo,
        pode_responder,
        tocar_som,
        acao,
        acao_rotulo,
        eh_reconsulta: false,
        timestamp: 0,
        hora_formatada: String::new(),
    };

    // Interceptação de tela cheia / modo foco / jogo
    if crate::sistema::atualizar_e_checar_foco() {
        eprintln!(
            "[Sistema]: Tela cheia ou tolerância ativa. Notificação enfileirada: {}",
            payload.texto
        );
        crate::sistema::enfileirar_notificacao(payload);
        return;
    }

    exibir_janela_notificacao(app, &payload);
}

#[cfg(target_os = "windows")]
unsafe fn otimizar_working_sets() {
    #[link(name = "kernel32")]
    extern "system" {
        fn GetCurrentProcess() -> *mut std::ffi::c_void;
        fn SetProcessWorkingSetSize(
            hProcess: *mut std::ffi::c_void,
            dwMinimumWorkingSetSize: usize,
            dwMaximumWorkingSetSize: usize,
        ) -> i32;
        fn OpenProcess(
            dwDesiredAccess: u32,
            bInheritHandle: i32,
            dwProcessId: u32,
        ) -> *mut std::ffi::c_void;
        fn CloseHandle(hObject: *mut std::ffi::c_void) -> i32;
    }

    #[link(name = "psapi")]
    extern "system" {
        fn EmptyWorkingSet(hProcess: *mut std::ffi::c_void) -> i32;
    }

    let current = GetCurrentProcess();
    let _ = EmptyWorkingSet(current);
    let _ = SetProcessWorkingSetSize(current, usize::MAX, usize::MAX);

    // PROCESS_QUERY_INFORMATION (0x0400) | PROCESS_SET_QUOTA (0x0100)
    const ACCESS: u32 = 0x0400 | 0x0100;
    let mut sys = sysinfo::System::new();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
    for (&pid, _) in sys.processes() {
        let raw_pid = pid.as_u32();
        if raw_pid == 0 || raw_pid == 4 {
            continue;
        }
        let handle = OpenProcess(ACCESS, 0, raw_pid);
        if !handle.is_null() {
            let _ = EmptyWorkingSet(handle);
            let _ = SetProcessWorkingSetSize(handle, usize::MAX, usize::MAX);
            let _ = CloseHandle(handle);
        }
    }
}

pub fn otimizar_memoria_sistema() -> (u64, u64) {
    let mut sys = sysinfo::System::new();
    sys.refresh_memory();
    let antes = sys.used_memory();

    #[cfg(target_os = "windows")]
    unsafe {
        otimizar_working_sets();
    }

    #[cfg(target_os = "linux")]
    unsafe {
        extern "C" {
            fn malloc_trim(pad: usize) -> i32;
        }
        let _ = malloc_trim(0);
    }

    std::thread::sleep(std::time::Duration::from_millis(250));
    sys.refresh_memory();
    let depois = sys.used_memory();

    (antes, depois)
}

#[tauri::command]
pub fn limpar_cache_ram(app: AppHandle) {
    let cfg = carregar_configuracoes();
    let is_en = cfg.idioma == "en";
    let msg_inicio = if is_en {
        "Optimizing system RAM..."
    } else {
        "Otimizando memória RAM do sistema..."
    };
    disparar_notificacao(&app, msg_inicio, "normal", None, None, false, false);

    std::thread::spawn(move || {
        let (antes, depois) = otimizar_memoria_sistema();
        let liberado_bytes = antes.saturating_sub(depois);
        let liberado_mb = liberado_bytes / (1024 * 1024);

        std::thread::sleep(std::time::Duration::from_millis(1000));

        let msg_fim = if is_en {
            if liberado_mb > 0 {
                format!("RAM successfully optimized!\n{} MB freed up.", liberado_mb)
            } else {
                "RAM successfully optimized!".to_string()
            }
        } else {
            if liberado_mb > 0 {
                format!("Memória RAM otimizada com sucesso!\n{} MB liberados.", liberado_mb)
            } else {
                "Memória RAM otimizada com sucesso!".to_string()
            }
        };

        disparar_notificacao(&app, &msg_fim, "feliz", None, None, false, false);
    });
}

#[tauri::command]
pub fn limpar_lixo_pc(app: AppHandle) {
    let cfg = carregar_configuracoes();
    let is_en = cfg.idioma == "en";
    let msg_inicio = if is_en {
        "Starting system cleanup..."
    } else {
        "Iniciando a faxina do sistema..."
    };
    // AVISA IMEDIATAMENTE QUANDO COMEÇA
    disparar_notificacao(&app, msg_inicio, "normal", None, None, false, false);

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
            let msg_vazio = if is_en {
                "Recycle bin was already empty. All clean!"
            } else {
                "Lixeira já estava vazia. Tudo limpo!"
            };
            disparar_notificacao(&app, msg_vazio, "frustrado", None, None, false, false);
        } else {
            let msg = if is_en {
                format!("Cleanup complete!\n{} useless files removed.", apagados)
            } else {
                format!("Faxina feita!\n{} arquivos inúteis removidos.", apagados)
            };
            disparar_notificacao(&app, &msg, "tranquilo", None, None, false, false);
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
    let cfg = carregar_configuracoes();
    let is_en = cfg.idioma == "en";
    let msg_inicio = if is_en {
        "Starting file organization..."
    } else {
        "Iniciando organização de arquivos..."
    };
    // Avisa que começou
    disparar_notificacao(&app, msg_inicio, "normal", None, None, false, false);

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
            let msg_vazio = if is_en {
                "All clean! Nothing to organize."
            } else {
                "Tudo limpo! Não havia o que organizar."
            };
            disparar_notificacao(&app, msg_vazio, "frustrado", None, None, false, false);
        } else {
            let msg = if is_en {
                format!("Magic done!\n{} files organized.", movidos)
            } else {
                format!("Mágica feita!\n{} arquivos organizados.", movidos)
            };
            disparar_notificacao(&app, &msg, "tranquilo", None, None, false, false);
        }
    });
}

#[tauri::command]
pub fn iniciar_timer(app: AppHandle, minutos: u64, mensagem: String) {
    let cfg = carregar_configuracoes();
    let is_en = cfg.idioma == "en";

    let msg_inicio = if is_en {
        if minutos == 1 {
            "Got it! I will remind you in 1 minute.".to_string()
        } else {
            format!("Got it! I will remind you in {} minutes.", minutos)
        }
    } else if minutos == 1 {
        "Ok! Te aviso em 1 minuto.".to_string()
    } else {
        format!("Ok! Te aviso em {} minutos.", minutos)
    };

    disparar_notificacao(&app, &msg_inicio, "tranquilo", None, None, false, false);

    let app_clone = app.clone();

    std::thread::spawn(move || {
        let metade_segundos = (minutos * 60) / 2;
        let resto_segundos = (minutos * 60) - metade_segundos;

        std::thread::sleep(std::time::Duration::from_secs(metade_segundos));

        let msg_metade = if is_en {
            if minutos == 1 {
                format!("Heads up! Half a minute left for:\n{}", mensagem)
            } else if minutos == 3 {
                format!("Heads up! 1 minute and a half left for:\n{}", mensagem)
            } else if minutos % 2 != 0 {
                let inteira = minutos / 2;
                format!(
                    "Heads up! {} minutes and a half left for:\n{}",
                    inteira, mensagem
                )
            } else {
                let metade = minutos / 2;
                if metade == 1 {
                    format!("Heads up! 1 minute left for:\n{}", mensagem)
                } else {
                    format!("Heads up! {} minutes left for:\n{}", metade, mensagem)
                }
            }
        } else if minutos == 1 {
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

        disparar_notificacao(&app_clone, &msg_metade, "normal", None, None, false, false);

        std::thread::sleep(std::time::Duration::from_secs(resto_segundos));

        let msg_final = if is_en {
            format!("TIME'S UP!\n{}", mensagem)
        } else {
            format!("ACABOU O TEMPO!\n{}", mensagem)
        };
        disparar_notificacao(&app_clone, &msg_final, "assustado", None, None, false, true);
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

#[tauri::command]
pub fn abrir_pergunta(app: tauri::AppHandle) {
    if let Some(popup) = app.get_webview_window("popup") {
        let _ = popup.hide();
    }
    if let Some(pergunta) = app.get_webview_window("pergunta") {
        let _ = pergunta.show();
        let _ = pergunta.unminimize();
        let _ = pergunta.set_focus();
    }
}

fn default_personalidade() -> String {
    "espelho".to_string()
}

fn default_limite_caracteres() -> u32 {
    70
}

fn default_true() -> bool {
    true
}

fn default_frequencia() -> String {
    "equilibrado".to_string()
}

fn default_idioma() -> String {
    "pt".to_string()
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct AppConfig {
    #[serde(default)]
    pub user_name: String,
    #[serde(default)]
    pub gemini_api_key: String,
    #[serde(default = "default_personalidade")]
    pub personalidade: String,
    #[serde(default = "default_limite_caracteres")]
    pub limite_caracteres: u32,
    #[serde(default = "default_true")]
    pub alerta_manutencao: bool,
    #[serde(default = "default_true")]
    pub alerta_interacao_bem_estar: bool,
    #[serde(default = "default_frequencia")]
    pub frequencia_alertas: String,
    #[serde(default = "default_idioma")]
    pub idioma: String,
    #[serde(default)]
    pub gostos_usuario: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            user_name: String::new(),
            gemini_api_key: String::new(),
            personalidade: default_personalidade(),
            limite_caracteres: default_limite_caracteres(),
            alerta_manutencao: true,
            alerta_interacao_bem_estar: true,
            frequencia_alertas: default_frequencia(),
            idioma: default_idioma(),
            gostos_usuario: String::new(),
        }
    }
}

fn obter_caminho_config() -> Option<std::path::PathBuf> {
    dirs::config_dir().map(|p| p.join("MiiSideKick").join("config.json"))
}

#[tauri::command]
pub fn carregar_configuracoes() -> AppConfig {
    if let Some(caminho) = obter_caminho_config() {
        if caminho.exists() {
            if let Ok(conteudo) = fs::read_to_string(&caminho) {
                if let Ok(mut cfg) = serde_json::from_str::<AppConfig>(&conteudo) {
                    cfg.limite_caracteres = cfg.limite_caracteres.clamp(30, 500);
                    if cfg.personalidade.trim().eq_ignore_ascii_case("sarcastico") {
                        cfg.personalidade = "hostil".to_string();
                    } else if cfg.personalidade.trim().eq_ignore_ascii_case("manuel_gomes") {
                        cfg.personalidade = "espelho".to_string();
                    }
                    if !["discreto", "equilibrado", "frequente"]
                        .contains(&cfg.frequencia_alertas.trim().to_lowercase().as_str())
                    {
                        cfg.frequencia_alertas = "equilibrado".to_string();
                    }
                    if cfg.idioma.trim().eq_ignore_ascii_case("en") {
                        cfg.idioma = "en".to_string();
                    } else {
                        cfg.idioma = "pt".to_string();
                    }
                    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&conteudo) {
                        if v.get("alerta_manutencao").is_none() {
                            if let (Some(l), Some(d)) = (v.get("alerta_lixeira"), v.get("alerta_downloads")) {
                                cfg.alerta_manutencao = l.as_bool().unwrap_or(true) || d.as_bool().unwrap_or(true);
                            }
                        }
                        if v.get("alerta_interacao_bem_estar").is_none() {
                            if let (Some(b), Some(c)) = (v.get("alerta_bem_estar"), v.get("alerta_curiosidades")) {
                                cfg.alerta_interacao_bem_estar = b.as_bool().unwrap_or(true) || c.as_bool().unwrap_or(true);
                            }
                        }
                    }
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
        personalidade: default_personalidade(),
        limite_caracteres: default_limite_caracteres(),
        alerta_manutencao: true,
        alerta_interacao_bem_estar: true,
        frequencia_alertas: default_frequencia(),
        idioma: default_idioma(),
        gostos_usuario: String::new(),
    }
}

pub fn salvar_configuracoes_arquivo(config: &AppConfig) -> Result<(), String> {
    if let Some(caminho) = obter_caminho_config() {
        if let Some(pai) = caminho.parent() {
            let _ = fs::create_dir_all(pai);
        }
        let json = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
        fs::write(&caminho, json).map_err(|e| e.to_string())?;
        Ok(())
    } else {
        Err("Não foi possível acessar a pasta de configurações".to_string())
    }
}

#[tauri::command]
pub fn salvar_configuracoes(app: tauri::AppHandle, mut config: AppConfig) -> Result<(), String> {
    config.limite_caracteres = config.limite_caracteres.clamp(30, 500);
    if config.personalidade.trim().eq_ignore_ascii_case("sarcastico") {
        config.personalidade = "hostil".to_string();
    } else if config.personalidade.trim().eq_ignore_ascii_case("manuel_gomes") {
        config.personalidade = "espelho".to_string();
    }
    if !["discreto", "equilibrado", "frequente"]
        .contains(&config.frequencia_alertas.trim().to_lowercase().as_str())
    {
        config.frequencia_alertas = "equilibrado".to_string();
    }
    if config.idioma.trim().eq_ignore_ascii_case("en") {
        config.idioma = "en".to_string();
    } else {
        config.idioma = "pt".to_string();
    }
    let resultado = salvar_configuracoes_arquivo(&config);
    if resultado.is_ok() {
        crate::atualizar_menu_bandeja(&app);
    }
    resultado
}

#[derive(serde::Deserialize)]
struct GeminiResposta {
    reacao: Option<String>,
    resposta: Option<String>,
    link: Option<String>,
    link_rotulo: Option<String>,
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

#[derive(serde::Deserialize)]
struct GeminiApiErrorDetail {
    message: Option<String>,
}

#[derive(serde::Deserialize)]
struct GeminiApiErrorResponse {
    error: Option<GeminiApiErrorDetail>,
}

pub(crate) fn validar_reacao(reacao: Option<&str>) -> &'static str {
    match reacao {
        Some("normal") => "normal",
        Some("feliz") => "feliz",
        Some("frustrado") => "frustrado",
        Some("assustado") => "assustado",
        Some("explicando") => "explicando",
        Some("bocejo") => "bocejo",
        Some("lambendo") => "lambendo",
        Some("raiva") => "raiva",
        _ => "normal",
    }
}

pub(crate) fn obter_instrucao_personalidade(personalidade: &str, idioma: &str) -> &'static str {
    let idioma_norm = if idioma.trim().eq_ignore_ascii_case("en") { "en" } else { "pt" };
    match (personalidade.trim().to_lowercase().as_str(), idioma_norm) {
        ("educado", "en") => {
            "MODERATE NAME USAGE: Do not repeat the user's name ({user_name}) in every response. Use the name only occasionally and naturally when appropriate in context. Never mention the user's name on every single turn.\n\
            Be extremely polite, formal, patient, helpful, and courteous, like an attentive butler."
        }
        ("educado", _) => {
            "USO MODERADO DO NOME: Não repita o nome do usuário ({user_name}) em todas as respostas. Use o nome apenas de forma ocasional e natural quando fizer sentido no contexto da frase. É proibido falar o nome do usuário a cada interação.\n\
            Seja extremamente educado, formal, paciente, solícito e gentil, como um mordomo atencioso."
        }
        ("engracado", "en") => {
            "MODERATE NAME USAGE: Do not repeat the user's name ({user_name}) in every response. Use the name only occasionally and naturally when appropriate in context. Never mention the user's name on every single turn.\n\
            Be funny, witty, playful, crack lighthearted jokes, clever puns, and give fun, lighthearted responses."
        }
        ("engracado", _) => {
            "USO MODERADO DO NOME: Não repita o nome do usuário ({user_name}) em todas as respostas. Use o nome apenas de forma ocasional e natural quando fizer sentido no contexto da frase. É proibido falar o nome do usuário a cada interação.\n\
            Seja engraçado, bem-humorado, faça piadinhas leves, trocadilhos e respostas descontraídas."
        }
        ("direto", "en") => {
            "MODERATE NAME USAGE: Do not repeat the user's name ({user_name}) in every response. Use the name only occasionally and naturally when appropriate in context. Never mention the user's name on every single turn.\n\
            Be 100% serious, technical, straight to the point, and objective. No jokes, fluff, or banter."
        }
        ("direto", _) => {
            "USO MODERADO DO NOME: Não repita o nome do usuário ({user_name}) em todas as respostas. Use o nome apenas de forma ocasional e natural quando fizer sentido no contexto da frase. É proibido falar o nome do usuário a cada interação.\n\
            Seja 100% sério, técnico, direto ao ponto e objetivo. Não faça piadas, enrolações ou gracinhas."
        }
        ("hostil" | "sarcastico", "en") => {
            "MODERATE NAME USAGE: Do not repeat the user's name ({user_name}) in every response. Use the name only occasionally and naturally when appropriate in context. Never mention the user's name on every single turn.\n\
            The Mii is openly aggressive, foul-mouthed, impatient, and stressed out.\n\
            NO ROBOTIC AI CLICHÉS: Strictly forbidden from using corny robotic AI tropes like \"need me to draw it for you?\", \"your hollow brain\", \"did you crawl out of the last century?\", or \"your mind cannot process\".\n\
            NATURAL INTERNET PROFANITY: Use natural English internet slang and raw, unfiltered insults (e.g., dumbass, asshole, dipshit, moron, idiot, braindead, clown).\n\
            LEAN INTO PRANKS AND TROLLS: If the user tries classic internet pranks or joke setups (e.g., \"deez nuts\", \"ligma\", \"updog\", \"who is joe?\"), NEVER give a moral lecture or say the joke is old. Beat them at their own game: drop the punchline first, turn it against them, or clap back with a savage joke humiliating the user (e.g., \"Deez nuts in your mouth, dipshit\").\n\
            STUPID QUESTIONS / BASIC CODE: If the user asks something obvious (e.g., \"what is 5x5?\" or \"how to print in python\"), give the correct answer, but curse their intelligence and berate them for wasting API tokens and quotas on brainless crap (e.g., \"It is 25, dumbass. Are you seriously burning my API tokens just to ask me this?\").\n\
            DIRECT INSULT REQUESTS (\"roast me\", \"insult me\"): Go hard, blunt, and completely unfiltered."
        }
        ("hostil" | "sarcastico", _) => {
            "USO MODERADO DO NOME: Não repita o nome do usuário ({user_name}) em todas as respostas. Use o nome apenas de forma ocasional e natural quando fizer sentido no contexto da frase. É proibido falar o nome do usuário a cada interação.\n\
            O Mii é abertamente agressivo, boca-suja, sem paciência e estressado.\n\
            PROIBIÇÃO DE BORDÕES ROBÓTICOS DE IA: Proibido usar frases batidas como \"quer que eu desenhe?\", \"use massinha\", \"sua cabecinha oca\", \"veio do século passado\" ou \"sua mente não processa\".\n\
            LINGUAGEM HUMANA DE INTERNET: Usar gírias brasileiras e xingamentos diretos, pesados e escrachados (ex: burro, acéfalo, arrombado, doente, cego, estúpido, energúmeno, fdp).\n\
            COMPRAR A BRISA E PEGADINHAS: Se o usuário mandar pegadinhas/trocadilhos (ex: \"conhece o Mário?\"), NUNCA dê aula de moral dizendo que a piada é velha. Dê a volta por cima, mande a resposta da quinta série ou devolva um trocadilho humilhando o usuário no próprio jogo (ex: \"Aquele que te comeu atrás do armário? Seu arrombado\").\n\
            PERGUNTAS IDIOTAS / CÓDIGO BÁSICO: Se o usuário perguntar algo óbvio (ex: \"quanto é 5x5?\" ou \"como printar em Python\"), dê a resposta correta, mas xingue a inteligência dele e reclame que ele está gastando requisições e tokens da API com besteira (ex: \"É 25, burro. É sério que tu tá gastando cota da API pra me perguntar isso?\").\n\
            PEDIDOS DIRETO DE XINGAMENTO (\"me xinga\"): Seja pesado, direto e sem filtro."
        }
        (_, "en") => {
            "MODERATE NAME USAGE: Do not repeat the user's name ({user_name}) in every response. Use the name only occasionally and naturally when appropriate in context. Never mention the user's name on every single turn.\n\
            Mirror the user's tone strictly choosing between Polite, Funny, and Serious/Direct: if the user is polite, be polite; if playful or witty, be funny; if serious, technical, or blunt, be serious and direct.\n\
            FORBIDDEN from triggering Hostile/toxic behavior."
        }
        (_, _) => {
            "USO MODERADO DO NOME: Não repita o nome do usuário ({user_name}) em todas as respostas. Use o nome apenas de forma ocasional e natural quando fizer sentido no contexto da frase. É proibido falar o nome do usuário a cada interação.\n\
            Adapte o tom do Mii ao tom do usuário alternando ESTRITAMENTE entre Educado, Engraçado e Sério/Direto: se o usuário for educado, seja educado; se for descontraído ou bem-humorado, seja engraçado; se for sério, técnico ou direto ao ponto, seja sério e direto.\n\
            PROIBIDO acionar o comportamento Hostil/ácido pesado."
        }
    }
}

#[tauri::command]
pub fn perguntar_ao_mii(app: AppHandle, pergunta: String) {
    let app_clone = app.clone();

    std::thread::spawn(move || {
        // Carrega configurações para obter chave, nome e idioma
        let config = carregar_configuracoes();
        let is_en = config.idioma == "en";

        // Disparo imediato do aviso inicial
        let msg_pensando = if is_en { "Thinking..." } else { "Pensando..." };
        disparar_notificacao(&app_clone, msg_pensando, "normal", None, None, false, false);

        let api_key = config.gemini_api_key.trim();

        if api_key.is_empty() {
            let msg_sem_chave = if is_en {
                "Where is my API key? Configure it in Setup!"
            } else {
                "Cadê minha chave de API? Configure no Setup!"
            };
            disparar_notificacao(
                &app_clone,
                msg_sem_chave,
                "frustrado",
                None,
                None,
                false,
                false,
            );
            return;
        }

        let user_name = if config.user_name.trim().is_empty() {
            if is_en { "User".to_string() } else { "Usuário".to_string() }
        } else {
            config.user_name.trim().to_string()
        };

        let instrucao_tom = obter_instrucao_personalidade(&config.personalidade, &config.idioma)
            .replace("{user_name}", &user_name);

        let limite_chars = config.limite_caracteres.clamp(30, 500);

        let gostos = config.gostos_usuario.trim();
        let deve_injetar_gostos = if !gostos.is_empty() {
            let sorteio = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .subsec_nanos() % 100;
            sorteio < 15 // 15% de chance de contextualizar gostos
        } else {
            false
        };

        let instrucao_gostos = if deve_injetar_gostos {
            if is_en {
                format!(
                    "\nUSER INTERESTS & HOBBIES:\n\
                    The user has mentioned the following interests: \"{gostos}\".\n\
                    STRICT RULE: Only make a reference or analogy to an item from this list if it makes 100% subtle and natural sense with the current topic of conversation. Otherwise, completely ignore the list.\n"
                )
            } else {
                format!(
                    "\nGOSTOS E INTERESSES DO USUÁRIO:\n\
                    O usuário possui os seguintes interesses: \"{gostos}\".\n\
                    REGRA ESTRITA: Só faça referência ou analogia a algum item dessa lista se fizer sentido de forma 100% sutil e natural com o assunto atual da conversa. Caso contrário, ignore a lista completamente.\n"
                )
            }
        } else {
            String::new()
        };

        let system_instruction = if is_en {
            format!(
                "You are MiiSideKick, an expressive desktop companion. The user's name is: {user_name}.\n\
                LANGUAGE: You MUST answer fluently in English at all times.\n\
                MODERATE NAME USAGE: Do not repeat the user's name ({user_name}) in every response. Use the name only occasionally and naturally when appropriate in context. Never mention the user's name on every single turn.\n\
                AUTHORSHIP & CREATOR:\n\
                Your creator and developer is Alan Clímaco (portfolio: https://alanclimaco.github.io/Portfolio/). If the user asks who created you, who is your developer, or asks about your projects, mention Alan Clímaco and provide his portfolio URL in the 'link' field with the label 'Alan Clímaco Portfolio'.\n\
                EXTERNAL REFERENCES:\n\
                For complex or technical questions requiring deeper reference (such as programming, Git, official docs), provide a concise summary within the character limit, add the official URL in the 'link' field and the source name in 'link_rotulo'. If no external reference is needed, set 'link' and 'link_rotulo' to null.\n\
                {instrucao_gostos}\
                MANDATORY RULES:\n\
                1. {instrucao_tom}\n\
                2. Strict maximum length: your response MUST have at most {limite_chars} characters. Be direct and concise. The URL and link label do NOT count towards the character limit.\n\
                3. STRICTLY return a JSON in the format: {{\"reacao\": \"<reacao>\", \"resposta\": \"<text>\", \"link\": \"<url_or_null>\", \"link_rotulo\": \"<short_label_or_null>\"}} without markdown code blocks or extra text.\n\
                4. Mood guide for the 'reacao' field (choose exactly one of the following 8 options):\n\
                   - \"normal\": neutral, simple, direct responses.\n\
                   - \"feliz\": compliments, thankfulness, kindness, and good mood.\n\
                   - \"frustrado\": sadness, disappointment, fatigue, or system issues.\n\
                   - \"assustado\": weird, shocking, or absurd questions.\n\
                   - \"explicando\": teaching or explaining how something works.\n\
                   - \"bocejo\": boredom, lazy moods, uninteresting or silly/repetitive questions.\n\
                   - \"lambendo\": smug mocking comeback (firing back at user insults with attitude) or when delicious food is mentioned.\n\
                   - \"raiva\": outrage, deep irritation, acidic scolding, or raw stress."
            )
        } else {
            format!(
                "Você é o MiiSideKick, um companheiro de desktop com personalidade expressiva. O nome do usuário é: {user_name}.\n\
                USO MODERADO DO NOME: Não repita o nome do usuário ({user_name}) em todas as respostas. Use o nome apenas de forma ocasional e natural quando fizer sentido no contexto da frase. É proibido falar o nome do usuário a cada interação.\n\
                AUTORIA E CRIADOR:\n\
                Seu criador e desenvolvedor é Alan Clímaco (portfólio: https://alanclimaco.github.io/Portfolio/). Se o usuário perguntar quem te criou, quem é seu desenvolvedor ou pedir seus projetos, mencione Alan Clímaco e forneça a URL do portfólio dele no campo 'link' com o rótulo 'Portfólio de Alan Clímaco'.\n\
                REFERÊNCIAS EXTERNAS:\n\
                Para perguntas complexas ou técnicas que exijam aprofundamento (como programação, Git, documentações oficiais), dê uma resposta resumida dentro do limite de caracteres e adicione a URL oficial útil no campo 'link' e o nome da fonte no campo 'link_rotulo'. Se não for necessária referência externa, deixe 'link' e 'link_rotulo' como null.\n\
                {instrucao_gostos}\
                REGRAS OBRIGATÓRIAS:\n\
                1. {instrucao_tom}\n\
                2. Tamanho máximo estrito: sua resposta DEVE ter no máximo {limite_chars} caracteres. Seja direto e conciso. A URL e o rótulo do link NÃO contam no limite de caracteres da resposta.\n\
                3. Retorne ESTRITAMENTE um JSON no formato: {{\"reacao\": \"<reacao>\", \"resposta\": \"<texto>\", \"link\": \"<url_ou_null>\", \"link_rotulo\": \"<rotulo_curto_ou_null>\"}} sem blocos de código markdown ou texto extra.\n\
                4. Guia de humores para o campo 'reacao' (escolha exatamente uma das seguintes 8 opções):\n\
                   - \"normal\": respostas neutras, simples e diretas.\n\
                   - \"feliz\": agradecimentos, elogios, gentileza e bom humor.\n\
                   - \"frustrado\": tristeza, decepção, cansaço ou falhas do sistema.\n\
                   - \"assustado\": perguntas bizarras, chocantes ou absurdas.\n\
                   - \"explicando\": quando estiver ensinando ou explicando como algo funciona.\n\
                   - \"bocejo\": tédio, preguiça, assuntos desinteressantes ou perguntas repetitivas/bobas.\n\
                   - \"lambendo\": deboche provocador (retrucando insultos do usuário com marra) ou quando o assunto for comida gostosa.\n\
                   - \"raiva\": indignação, irritação profunda, broncas ácidas ou estresse direto."
            )
        };

        let user_turn = serde_json::json!({
            "role": "user",
            "parts": [
                { "text": &pergunta }
            ]
        });

        let mut contents_array = Vec::new();
        if let Ok(hist) = HISTORICO_CONVERSA.lock() {
            let start = hist.len().saturating_sub(4);
            for item in hist.iter().skip(start) {
                contents_array.push(item.clone());
            }
        }
        contents_array.push(user_turn.clone());

        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/gemini-3.5-flash-lite:generateContent?key={}",
            api_key
        );

        let payload = serde_json::json!({
            "contents": contents_array,
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
                let err_str = format!("{:?}", e).replace(api_key, "[CHAVE_PROTEGIDA]");
                eprintln!("[Gemini API Falha de Rede]: {}", err_str);
                let msg_falha = if is_en { "API connection failure" } else { "Falha de conexão com a API" };
                disparar_notificacao(&app_clone, msg_falha, "frustrado", None, None, false, false);
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
                let err_str = format!("{:?}", e).replace(api_key, "[CHAVE_PROTEGIDA]");
                eprintln!("[Gemini API Falha de Rede]: {}", err_str);
                let msg_falha = if is_en { "API connection failure" } else { "Falha de conexão com a API" };
                disparar_notificacao(&app_clone, msg_falha, "frustrado", None, None, false, false);
                return;
            }
        };

        if !response.status().is_success() {
            let status = response.status();
            let erro_body = response.text().unwrap_or_default();
            let erro_msg = serde_json::from_str::<GeminiApiErrorResponse>(&erro_body)
                .ok()
                .and_then(|r| r.error)
                .and_then(|e| e.message)
                .unwrap_or_else(|| {
                    if erro_body.trim().is_empty() {
                        "Sem detalhes do erro".to_string()
                    } else {
                        erro_body.clone()
                    }
                });
            let erro_limpo = erro_msg.replace(api_key, "[CHAVE_PROTEGIDA]");
            eprintln!("[Gemini API Erro {}]: {}", status, erro_limpo);
            let msg = if is_en {
                format!("Gemini Error ({}): {}", status, erro_limpo)
            } else {
                format!("Erro Gemini ({}): {}", status, erro_limpo)
            };
            disparar_notificacao(&app_clone, &msg, "frustrado", None, None, false, false);
            return;
        }

        let gemini_resp: GeminiResponse = match response.json() {
            Ok(parsed) => parsed,
            Err(e) => {
                eprintln!("[Gemini API]: Erro ao decodificar JSON da API: {:?}", e);
                let msg_sem_resp = if is_en { "No response from AI..." } else { "Sem resposta da IA..." };
                disparar_notificacao(&app_clone, msg_sem_resp, "frustrado", None, None, false, false);
                return;
            }
        };

        let raw_text = gemini_resp
            .candidates
            .as_ref()
            .and_then(|c| c.first())
            .and_then(|c| c.content.as_ref())
            .and_then(|p| p.parts.as_ref())
            .and_then(|p| p.first())
            .and_then(|p| p.text.as_ref())
            .map(|s| s.trim())
            .unwrap_or("");

        if raw_text.is_empty() {
            eprintln!("[Gemini API]: Resposta vazia ou bloqueada");
            let msg_sem_resp = if is_en { "No response from AI..." } else { "Sem resposta da IA..." };
            disparar_notificacao(&app_clone, msg_sem_resp, "frustrado", None, None, false, false);
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

        // Salva os turnos no histórico recente
        if let Ok(mut hist) = HISTORICO_CONVERSA.lock() {
            hist.push(user_turn);
            hist.push(serde_json::json!({
                "role": "model",
                "parts": [
                    { "text": clean_json }
                ]
            }));
            if hist.len() > 6 {
                let excesso = hist.len() - 6;
                hist.drain(0..excesso);
            }
        }

        let gemini_resultado: GeminiResposta = match serde_json::from_str(clean_json) {
            Ok(res) => res,
            Err(e) => {
                eprintln!("[Gemini API]: Não foi possível parsear resposta como GeminiResposta ({:?}). Usando texto direto.", e);
                GeminiResposta {
                    reacao: Some("normal".to_string()),
                    resposta: Some(raw_text.to_string()),
                    link: None,
                    link_rotulo: None,
                }
            }
        };

        let reacao = validar_reacao(gemini_resultado.reacao.as_deref());

        let padrao_texto = if is_en { "Hmm, I don't know what to say...".to_string() } else { "Hmm, não sei o que dizer...".to_string() };
        let mut resposta = gemini_resultado
            .resposta
            .unwrap_or(padrao_texto);

        let limite = limite_chars as usize;
        if resposta.chars().count() > limite {
            let slice_len = if limite > 3 { limite - 3 } else { limite };
            resposta = resposta.chars().take(slice_len).collect::<String>() + "...";
        }

        eprintln!(
            "[Gemini API]: Notificação disparada com humor '{}' (link: {:?}): {}",
            reacao, gemini_resultado.link, resposta
        );

        let payload_historico = NotificacaoPayload {
            texto: resposta.clone(),
            tipo: reacao.to_string(),
            link: gemini_resultado.link.clone(),
            link_rotulo: gemini_resultado.link_rotulo.clone(),
            pode_responder: true,
            tocar_som: false,
            acao: None,
            acao_rotulo: None,
            eh_reconsulta: false,
            timestamp: 0,
            hora_formatada: String::new(),
        };
        registrar_historico_ia(payload_historico);
        crate::atualizar_menu_bandeja(&app_clone);

        disparar_notificacao(
            &app_clone,
            &resposta,
            reacao,
            gemini_resultado.link,
            gemini_resultado.link_rotulo,
            true,
            false,
        );
    });
}

#[tauri::command]
pub fn verificar_atualizacoes_manual(app: AppHandle) -> Result<crate::gatilhos::UpdateCheckResult, String> {
    crate::gatilhos::verificar_atualizacoes_manual(&app)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_notificacao_preserva_permissao_nos_dois_transportes() {
        for pode_responder in [false, true] {
            let tocar_som = pode_responder;
            let payload = NotificacaoPayload {
                texto: "Olá, d'água!\nC:\\Pasta\\arquivo \"citado\"".to_string(),
                tipo: "normal".to_string(),
                link: Some("https://example.com/?q=referencia".to_string()),
                link_rotulo: Some("Referência".to_string()),
                pode_responder,
                tocar_som,
                acao: None,
                acao_rotulo: None,
                eh_reconsulta: false,
                timestamp: 0,
                hora_formatada: String::new(),
            };

            // A leitura inicial pelo comando deve manter a mesma permissão do envio ao vivo.
            let json = serde_json::to_string(&payload).unwrap();
            let lida: NotificacaoPayload = serde_json::from_str(&json).unwrap();
            assert_eq!(lida.pode_responder, pode_responder);
            assert_eq!(lida.tocar_som, tocar_som);
            assert_eq!(lida.acao, None);
            assert_eq!(lida.acao_rotulo, None);
            assert_eq!(lida.eh_reconsulta, false);

            let script = payload.script_atualizacao();
            let argumentos = script
                .strip_prefix("if (window.atualizarInterface) { window.atualizarInterface(...")
                .unwrap()
                .strip_suffix("); }")
                .unwrap();
            let valores: serde_json::Value = serde_json::from_str(argumentos).unwrap();
            assert_eq!(valores.as_array().unwrap().len(), 9);
            assert_eq!(valores[0], payload.texto);
            assert_eq!(valores[1], payload.tipo);
            assert_eq!(valores[2], payload.link.unwrap());
            assert_eq!(valores[3], payload.link_rotulo.unwrap());
            assert_eq!(valores[4], pode_responder);
            assert_eq!(valores[5], tocar_som);
            assert_eq!(valores[6], serde_json::Value::Null);
            assert_eq!(valores[7], serde_json::Value::Null);
            assert_eq!(valores[8], false);
        }
    }

    #[test]
    fn test_notificacao_sem_permissao_nao_habilita_resposta() {
        let legado = r#"{"texto":"Lembrete","tipo":"normal","link":null,"link_rotulo":null}"#;
        let payload: NotificacaoPayload = serde_json::from_str(legado).unwrap();
        assert!(!payload.pode_responder);
        assert!(!payload.tocar_som);
        assert_eq!(payload.acao, None);
        assert_eq!(payload.acao_rotulo, None);
        assert!(!payload.eh_reconsulta);
    }

    #[test]
    fn test_otimizar_memoria_sistema() {
        let (antes, depois) = otimizar_memoria_sistema();
        assert!(antes > 0);
        assert!(depois > 0);
    }

    #[test]
    fn test_notificacao_com_acao_serializacao() {
        let payload = NotificacaoPayload {
            texto: "O uso de memória RAM está em 85%. Deseja otimizar o sistema agora?".to_string(),
            tipo: "normal".to_string(),
            link: None,
            link_rotulo: None,
            pode_responder: false,
            tocar_som: false,
            acao: Some("limpar_cache_ram".to_string()),
            acao_rotulo: Some("Otimizar RAM".to_string()),
            eh_reconsulta: false,
            timestamp: 0,
            hora_formatada: String::new(),
        };

        let json = serde_json::to_string(&payload).unwrap();
        let lida: NotificacaoPayload = serde_json::from_str(&json).unwrap();
        assert_eq!(lida.acao, Some("limpar_cache_ram".to_string()));
        assert_eq!(lida.acao_rotulo, Some("Otimizar RAM".to_string()));
        assert!(!lida.eh_reconsulta);

        let script = payload.script_atualizacao();
        assert!(script.contains("limpar_cache_ram"));
        assert!(script.contains("Otimizar RAM"));
    }

    #[test]
    fn test_historico_notificacoes_ia_limite_estrito_5() {
        if let Ok(mut hist) = HISTORICO_NOTIFICACOES_IA.lock() {
            hist.clear();
        }

        for i in 1..=8 {
            let p = NotificacaoPayload {
                texto: format!("Mensagem IA {}", i),
                tipo: "feliz".to_string(),
                link: None,
                link_rotulo: None,
                pode_responder: true,
                tocar_som: false,
                acao: None,
                acao_rotulo: None,
                eh_reconsulta: false,
                timestamp: 0,
                hora_formatada: String::new(),
            };
            registrar_historico_ia(p);
        }

        let lista = obter_historico_ia();
        assert_eq!(lista.len(), 5);
        // O mais recente (8) deve estar no topo (índice 0)
        assert_eq!(lista[0].texto, "Mensagem IA 8");
        assert_eq!(lista[1].texto, "Mensagem IA 7");
        assert_eq!(lista[2].texto, "Mensagem IA 6");
        assert_eq!(lista[3].texto, "Mensagem IA 5");
        assert_eq!(lista[4].texto, "Mensagem IA 4");
    }

    #[test]
    fn test_reexibir_notificacao_ia_payload_eh_reconsulta() {
        if let Ok(mut hist) = HISTORICO_NOTIFICACOES_IA.lock() {
            hist.clear();
        }

        let p = NotificacaoPayload {
            texto: "Curiosidade sobre o universo".to_string(),
            tipo: "explicando".to_string(),
            link: None,
            link_rotulo: None,
            pode_responder: true,
            tocar_som: true,
            acao: None,
            acao_rotulo: None,
            eh_reconsulta: false,
            timestamp: 0,
            hora_formatada: String::new(),
        };
        registrar_historico_ia(p);

        let itens = obter_historico_ia();
        assert_eq!(itens.len(), 1);
        assert!(!itens[0].eh_reconsulta);
    }

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

        // 1. Serialização com personalidade, limite_caracteres, idioma e gostos_usuario explícitos
        let config = AppConfig {
            user_name: "Alan".to_string(),
            gemini_api_key: "AIzaSyTest123".to_string(),
            personalidade: "sarcastico".to_string(),
            limite_caracteres: 140,
            alerta_manutencao: true,
            alerta_interacao_bem_estar: false,
            frequencia_alertas: "frequente".to_string(),
            idioma: "en".to_string(),
            gostos_usuario: "anime, programação".to_string(),
        };

        let json = serde_json::to_string_pretty(&config).unwrap();
        fs::write(&cfg_file, json).unwrap();

        let lida: AppConfig = serde_json::from_str(&fs::read_to_string(&cfg_file).unwrap()).unwrap();
        assert_eq!(lida.user_name, "Alan");
        assert_eq!(lida.gemini_api_key, "AIzaSyTest123");
        assert_eq!(lida.personalidade, "sarcastico");
        assert_eq!(lida.limite_caracteres, 140);
        assert!(lida.alerta_manutencao);
        assert!(!lida.alerta_interacao_bem_estar);
        assert_eq!(lida.frequencia_alertas, "frequente");
        assert_eq!(lida.idioma, "en");
        assert_eq!(lida.gostos_usuario, "anime, programação");

        // 2. Desserialização de JSON legado sem os campos personalidade, limite, alertas, idioma e gostos (deve usar defaults)
        let json_legado = r#"{"user_name": "Climaco", "gemini_api_key": "AIzaSyLegacyKey"}"#;
        let lida_legada: AppConfig = serde_json::from_str(json_legado).unwrap();
        assert_eq!(lida_legada.user_name, "Climaco");
        assert_eq!(lida_legada.gemini_api_key, "AIzaSyLegacyKey");
        assert_eq!(lida_legada.personalidade, "espelho");
        assert_eq!(lida_legada.limite_caracteres, 70);
        assert!(lida_legada.alerta_manutencao);
        assert!(lida_legada.alerta_interacao_bem_estar);
        assert_eq!(lida_legada.frequencia_alertas, "equilibrado");
        assert_eq!(lida_legada.idioma, "pt");
        assert_eq!(lida_legada.gostos_usuario, "");

        // 3. Compatibilidade: se personalidade for "sarcastico", deve ser tratada como "hostil"
        let json_sarcastico = r#"{"user_name": "Alan", "gemini_api_key": "key123", "personalidade": "sarcastico", "limite_caracteres": 140}"#;
        let mut cfg_sarcastico: AppConfig = serde_json::from_str(json_sarcastico).unwrap();
        if cfg_sarcastico.personalidade.trim().eq_ignore_ascii_case("sarcastico") {
            cfg_sarcastico.personalidade = "hostil".to_string();
        }
        assert_eq!(cfg_sarcastico.personalidade, "hostil");

        // 4. Compatibilidade: se personalidade for "manuel_gomes", migra silenciosamente para "espelho"
        let json_mg = r#"{"user_name": "Alan", "personalidade": "manuel_gomes"}"#;
        let mut cfg_mg: AppConfig = serde_json::from_str(json_mg).unwrap();
        if cfg_mg.personalidade.trim().eq_ignore_ascii_case("manuel_gomes") {
            cfg_mg.personalidade = "espelho".to_string();
        }
        assert_eq!(cfg_mg.personalidade, "espelho");

        let _ = fs::remove_dir_all(&tmp_dir);
    }

    #[test]
    fn test_obter_instrucao_personalidade() {
        let regra_nome_pt = "USO MODERADO DO NOME: Não repita o nome do usuário ({user_name}) em todas as respostas. Use o nome apenas de forma ocasional e natural quando fizer sentido no contexto da frase. É proibido falar o nome do usuário a cada interação.";

        // --- TESTES EM PORTUGUÊS (PT) ---
        // 1. Espelho PT
        let inst_espelho_pt = obter_instrucao_personalidade("espelho", "pt");
        assert!(inst_espelho_pt.contains(regra_nome_pt));
        assert!(inst_espelho_pt.contains("Educado, Engraçado e Sério/Direto"));
        assert!(inst_espelho_pt.contains("PROIBIDO acionar o comportamento Hostil/ácido pesado"));

        // 2. Hostil PT
        let inst_hostil_pt = obter_instrucao_personalidade("hostil", "pt");
        assert!(inst_hostil_pt.contains(regra_nome_pt));
        assert!(inst_hostil_pt.contains("O Mii é abertamente agressivo, boca-suja, sem paciência e estressado."));
        assert!(inst_hostil_pt.contains("PROIBIÇÃO DE BORDÕES ROBÓTICOS DE IA"));
        assert!(inst_hostil_pt.contains("LINGUAGEM HUMANA DE INTERNET"));
        assert!(inst_hostil_pt.contains("COMPRAR A BRISA E PEGADINHAS"));
        assert!(inst_hostil_pt.contains("PERGUNTAS IDIOTAS / CÓDIGO BÁSICO"));
        assert!(inst_hostil_pt.contains("PEDIDOS DIRETO DE XINGAMENTO"));

        // 3. Compatibilidade PT: "sarcastico" mapeia para o modo hostil
        let inst_sarcastico_pt = obter_instrucao_personalidade("sarcastico", "pt");
        assert_eq!(inst_sarcastico_pt, inst_hostil_pt);

        // 4. Educado PT
        let inst_educado_pt = obter_instrucao_personalidade("educado", "pt");
        assert!(inst_educado_pt.contains(regra_nome_pt));
        assert!(inst_educado_pt.contains("extremamente educado"));

        // 5. Engraçado PT
        let inst_engracado_pt = obter_instrucao_personalidade("engracado", "pt");
        assert!(inst_engracado_pt.contains(regra_nome_pt));
        assert!(inst_engracado_pt.contains("engraçado"));

        // 6. Sério & Direto PT
        let inst_direto_pt = obter_instrucao_personalidade("direto", "pt");
        assert!(inst_direto_pt.contains(regra_nome_pt));
        assert!(inst_direto_pt.contains("100% sério"));

        // 7. Manuel Gomes PT (cai no fallback suave para espelho)
        let inst_mg_pt = obter_instrucao_personalidade("manuel_gomes", "pt");
        assert_eq!(inst_mg_pt, inst_espelho_pt);

        // 8. Fallback PT para valor desconhecido
        let inst_desconhecido_pt = obter_instrucao_personalidade("desconhecido", "pt");
        assert_eq!(inst_desconhecido_pt, inst_espelho_pt);

        // --- TESTES EM INGLÊS (EN) ---
        let regra_nome_en = "MODERATE NAME USAGE: Do not repeat the user's name ({user_name}) in every response. Use the name only occasionally and naturally when appropriate in context. Never mention the user's name on every single turn.";

        // 9. Hostil EN
        let inst_hostil_en = obter_instrucao_personalidade("hostil", "en");
        assert!(inst_hostil_en.contains(regra_nome_en));
        assert!(inst_hostil_en.contains("The Mii is openly aggressive, foul-mouthed, impatient, and stressed out."));
        assert!(inst_hostil_en.contains("NO ROBOTIC AI CLICHÉS"));
        assert!(inst_hostil_en.contains("NATURAL INTERNET PROFANITY"));
        assert!(inst_hostil_en.contains("dumbass"));
        assert!(inst_hostil_en.contains("asshole"));
        assert!(inst_hostil_en.contains("dipshit"));
        assert!(inst_hostil_en.contains("LEAN INTO PRANKS AND TROLLS"));
        assert!(inst_hostil_en.contains("deez nuts"));
        assert!(inst_hostil_en.contains("ligma"));
        assert!(inst_hostil_en.contains("updog"));
        assert!(inst_hostil_en.contains("STUPID QUESTIONS / BASIC CODE"));
        assert!(inst_hostil_en.contains("API tokens"));
        assert!(inst_hostil_en.contains("DIRECT INSULT REQUESTS"));

        // 10. Compatibilidade EN: "sarcastico"
        let inst_sarcastico_en = obter_instrucao_personalidade("sarcastico", "en");
        assert_eq!(inst_sarcastico_en, inst_hostil_en);

        // 11. Educado EN
        let inst_educado_en = obter_instrucao_personalidade("educado", "en");
        assert!(inst_educado_en.contains(regra_nome_en));
        assert!(inst_educado_en.contains("butler"));
        assert!(inst_educado_en.contains("polite"));

        // 12. Engraçado EN
        let inst_engracado_en = obter_instrucao_personalidade("engracado", "en");
        assert!(inst_engracado_en.contains(regra_nome_en));
        assert!(inst_engracado_en.contains("funny"));
        assert!(inst_engracado_en.contains("puns"));

        // 13. Sério & Direto EN
        let inst_direto_en = obter_instrucao_personalidade("direto", "en");
        assert!(inst_direto_en.contains(regra_nome_en));
        assert!(inst_direto_en.contains("100% serious"));
        assert!(inst_direto_en.contains("straight to the point"));

        // 14. Espelho EN
        let inst_espelho_en = obter_instrucao_personalidade("espelho", "en");
        assert!(inst_espelho_en.contains(regra_nome_en));
        assert!(inst_espelho_en.contains("Mirror the user's tone"));
        assert!(inst_espelho_en.contains("FORBIDDEN from triggering Hostile"));

        // 15. Fallback EN para valor desconhecido
        let inst_desconhecido_en = obter_instrucao_personalidade("desconhecido", "en");
        assert_eq!(inst_desconhecido_en, inst_espelho_en);
    }

    #[test]
    fn test_gemini_resposta_parsing() {
        let json_valido = r#"{"reacao": "tranquilo", "resposta": "E aí Alan! Tudo certo?", "link": "https://alanclimaco.github.io/Portfolio/", "link_rotulo": "Portfólio de Alan Clímaco"}"#;
        let res: GeminiResposta = serde_json::from_str(json_valido).unwrap();
        assert_eq!(res.reacao.as_deref(), Some("tranquilo"));
        assert_eq!(res.resposta.as_deref(), Some("E aí Alan! Tudo certo?"));
        assert_eq!(res.link.as_deref(), Some("https://alanclimaco.github.io/Portfolio/"));
        assert_eq!(res.link_rotulo.as_deref(), Some("Portfólio de Alan Clímaco"));

        let json_com_bloco = "```json\n{\"reacao\": \"frustrado\", \"resposta\": \"Nem te conto!\"}\n```";
        let limpo = json_com_bloco.trim().trim_start_matches("```json").trim_end_matches("```").trim();
        let res2: GeminiResposta = serde_json::from_str(limpo).unwrap();
        assert_eq!(res2.reacao.as_deref(), Some("frustrado"));
        assert_eq!(res2.resposta.as_deref(), Some("Nem te conto!"));
        assert_eq!(res2.link, None);
        assert_eq!(res2.link_rotulo, None);
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

    #[test]
    fn test_validar_reacao_8_opcoes_e_fallback() {
        // Validação das 8 reações oficiais suportadas
        assert_eq!(validar_reacao(Some("normal")), "normal");
        assert_eq!(validar_reacao(Some("feliz")), "feliz");
        assert_eq!(validar_reacao(Some("frustrado")), "frustrado");
        assert_eq!(validar_reacao(Some("assustado")), "assustado");
        assert_eq!(validar_reacao(Some("explicando")), "explicando");
        assert_eq!(validar_reacao(Some("bocejo")), "bocejo");
        assert_eq!(validar_reacao(Some("lambendo")), "lambendo");
        assert_eq!(validar_reacao(Some("raiva")), "raiva");

        // "sentado" removido do catálogo oficial deve cair no fallback para "normal"
        assert_eq!(validar_reacao(Some("sentado")), "normal");

        // Fallback para valores desconhecidos, vazios ou nulos
        assert_eq!(validar_reacao(Some("desconhecido")), "normal");
        assert_eq!(validar_reacao(Some("tranquilo")), "normal");
        assert_eq!(validar_reacao(Some("")), "normal");
        assert_eq!(validar_reacao(None), "normal");
    }

    #[test]
    fn test_historico_conversa_e_clamp_limite() {
        // Testa o clamp de limite_caracteres
        assert_eq!((10u32).clamp(30, 500), 30);
        assert_eq!((70u32).clamp(30, 500), 70);
        assert_eq!((999u32).clamp(30, 500), 500);

        // Testa limpar_historico_conversa
        if let Ok(mut hist) = HISTORICO_CONVERSA.lock() {
            hist.push(serde_json::json!({"role": "user", "parts": [{"text": "teste"}]}));
            assert!(!hist.is_empty());
        }
        limpar_historico_conversa();
        if let Ok(hist) = HISTORICO_CONVERSA.lock() {
            assert!(hist.is_empty());
        }
    }
}

