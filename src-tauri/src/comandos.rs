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
    pub emulacao_roms: &'a Path,
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
        // Linux: deb, appimage, snap, flatpakref, rpm, bin, run
        // Windows, Mac e Mobile: exe, msi, dmg, pkg, apk, xapk
        "deb" | "appimage" | "snap" | "flatpakref" | "rpm" | "bin" | "run" | "exe" | "msi"
        | "dmg" | "pkg" | "apk" | "xapk" => Some(pastas.instaladores),

        // Downloads/Compactados:
        // zip, rar, 7z, tar, gz, bz2, xz, zst, tgz, iso, img
        "zip" | "rar" | "7z" | "tar" | "gz" | "bz2" | "xz" | "zst" | "tgz" | "iso" | "img" => {
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

        // Documentos/Emulacao/ROMs e ISOS:
        // chd, cso, rvz, gcm, rom, gba, sfc, nes
        "chd" | "cso" | "rvz" | "gcm" | "rom" | "gba" | "sfc" | "nes" => {
            Some(pastas.emulacao_roms)
        }

        // Documentos/Livros e HQs:
        // epub, mobi, azw, azw3, fb2, cbr, cbz, cbt, cb7
        "epub" | "mobi" | "azw" | "azw3" | "fb2" | "cbr" | "cbz" | "cbt" | "cb7" => {
            Some(pastas.livros_hqs)
        }

        // Documentos/Codigo:
        // Código e Scripts: py, js, (ts tratado abaixo), jsx, tsx, rs, c, cpp, h, hpp, cs, java, go, php, rb, lua, sh, bash
        // Configurações e Dados: json, yaml, yml, toml, xml, sql, env
        // Documentação e Web: md, markdown, tex, rst, html, css
        "py" | "js" | "jsx" | "tsx" | "rs" | "c" | "cpp" | "h" | "hpp" | "cs" | "java"
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

        // Documentos (Documentos de Escritório):
        // Textos: pdf, doc, docx, odt, rtf, txt
        // Planilhas: xlsx, xls, ods, csv, tsv
        // Apresentações: pptx, ppt, odp, key
        "pdf" | "doc" | "docx" | "odt" | "rtf" | "txt" | "xlsx" | "xls" | "ods" | "csv"
        | "tsv" | "pptx" | "ppt" | "odp" | "key" => Some(pastas.documentos),

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
/// - Cria o diretório de destino caso não exista.
/// - Previne sobrescrita anexando timestamp UNIX se o arquivo de destino já existir.
/// - Suporta partições distintas/cross-device (fallback de fs::rename para fs::copy + fs::remove_file).
fn mover_arquivo_seguro(origem: &Path, destino_dir: &Path) -> std::io::Result<()> {
    fs::create_dir_all(destino_dir)?;

    let file_name = match origem.file_name() {
        Some(name) => name,
        None => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Nome de arquivo inválido",
            ))
        }
    };

    let mut dest_path = destino_dir.join(file_name);

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

        dest_path = destino_dir.join(&novo_nome);

        if dest_path.exists() {
            let millis = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis();
            novo_nome = match ext {
                Some(e) if !e.is_empty() => format!("{}_{}.{}", stem, millis, e),
                _ => format!("{}_{}", stem, millis),
            };
            dest_path = destino_dir.join(&novo_nome);
        }
    }

    if origem == dest_path {
        return Ok(());
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

            let emulacao_saves_dir = docs_dir.join("Emulacao").join("Saves e Cheats");
            let emulacao_roms_dir = docs_dir.join("Emulacao").join("ROMs e ISOS");
            let livros_hqs_dir = docs_dir.join("Livros e HQs");
            let codigo_dir = docs_dir.join("Codigo");

            let design_3d_dir = pics_dir.join("Design e 3D");

            let pastas = PastasDestino {
                instaladores: &instaladores_dir,
                compactados: &compactados_dir,
                fontes: &fontes_dir,
                emulacao_saves: &emulacao_saves_dir,
                emulacao_roms: &emulacao_roms_dir,
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
    fn test_classificacao_todas_as_extensoes() {
        let p_inst = Path::new("/downloads/Instaladores");
        let p_comp = Path::new("/downloads/Compactados");
        let p_font = Path::new("/downloads/Fontes");
        let p_saves = Path::new("/docs/Emulacao/Saves e Cheats");
        let p_roms = Path::new("/docs/Emulacao/ROMs e ISOS");
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
            emulacao_roms: p_roms,
            livros_hqs: p_livros,
            codigo: p_codigo,
            documentos: p_docs,
            design_3d: p_d3d,
            imagens: p_pics,
            musicas: p_mus,
            videos: p_vid,
        };

        // Downloads/Instaladores:
        // Linux: deb, appimage, snap, flatpakref, rpm, bin, run
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
        // zip, rar, 7z, tar, gz, bz2, xz, zst, tgz, iso, img
        for ext in &[
            "zip", "rar", "7z", "tar", "gz", "bz2", "xz", "zst", "tgz", "iso", "img",
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

        // Documentos/Emulacao/ROMs e ISOS:
        // chd, cso, rvz, gcm, rom, gba, sfc, nes
        for ext in &["chd", "cso", "rvz", "gcm", "rom", "gba", "sfc", "nes"] {
            let f = PathBuf::from(format!("arquivo.{}", ext));
            assert_eq!(
                classificar_arquivo(&f, &pastas),
                Some(p_roms),
                "Falhou na extensão rom: {}",
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
        // Código e Scripts: py, js, jsx, tsx, rs, c, cpp, h, hpp, cs, java, go, php, rb, lua, sh, bash
        // Configurações e Dados: json, yaml, yml, toml, xml, sql, env
        // Documentação e Web: md, markdown, tex, rst, html, css
        for ext in &[
            "py", "js", "jsx", "tsx", "rs", "c", "cpp", "h", "hpp", "cs", "java", "go", "php",
            "rb", "lua", "sh", "bash", "json", "yaml", "yml", "toml", "xml", "sql", "env", "md",
            "markdown", "tex", "rst", "html", "css",
        ] {
            let f = PathBuf::from(format!("arquivo.{}", ext));
            assert_eq!(
                classificar_arquivo(&f, &pastas),
                Some(p_codigo),
                "Falhou na extensão codigo: {}",
                ext
            );
        }

        // Documentos (Escritório):
        // Textos: pdf, doc, docx, odt, rtf, txt
        // Planilhas: xlsx, xls, ods, csv, tsv
        // Apresentações: pptx, ppt, odp, key
        for ext in &[
            "pdf", "doc", "docx", "odt", "rtf", "txt", "xlsx", "xls", "ods", "csv", "tsv",
            "pptx", "ppt", "odp", "key",
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
}
