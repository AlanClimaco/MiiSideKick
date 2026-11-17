use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct RegraPasta {
    pub id: String,
    pub nome: String,
    pub destino_chave: String, // "downloads", "documents", "pictures", "audio", "videos"
    pub subpasta: Option<String>,
    pub extensoes: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct RegrasOrganizador {
    pub versao: u32,
    pub regras: Vec<RegraPasta>,
}

impl Default for RegrasOrganizador {
    fn default() -> Self {
        Self::obter_padrao()
    }
}

impl RegrasOrganizador {
    /// Gera a lista oficial padrão de regras de organização do MiiSideKick
    pub fn obter_padrao() -> Self {
        Self {
            versao: 1,
            regras: vec![
                RegraPasta {
                    id: "instaladores".to_string(),
                    nome: "Instaladores & Programas".to_string(),
                    destino_chave: "downloads".to_string(),
                    subpasta: Some("Instaladores".to_string()),
                    extensoes: vec![
                        "exe".to_string(), "msi".to_string(), "bat".to_string(),
                        "cmd".to_string(), "com".to_string(), "apk".to_string(),
                        "appx".to_string(), "appxbundle".to_string(),
                        "deb".to_string(), "rpm".to_string(),
                    ],
                },
                RegraPasta {
                    id: "compactados".to_string(),
                    nome: "Arquivos Compactados & Imagens de Disco".to_string(),
                    destino_chave: "downloads".to_string(),
                    subpasta: Some("Compactados".to_string()),
                    extensoes: vec![
                        "zip".to_string(), "rar".to_string(), "7z".to_string(),
                        "tar".to_string(), "gz".to_string(), "bz2".to_string(),
                        "xz".to_string(), "iso".to_string(), "img".to_string(),
                        "bin".to_string(), "cue".to_string(), "dmg".to_string(),
                        "vhd".to_string(), "vmdk".to_string(),
                    ],
                },
                RegraPasta {
                    id: "fontes".to_string(),
                    nome: "Tipografia & Fontes".to_string(),
                    destino_chave: "downloads".to_string(),
                    subpasta: Some("Fontes".to_string()),
                    extensoes: vec![
                        "ttf".to_string(), "otf".to_string(), "woff".to_string(),
                        "woff2".to_string(), "eot".to_string(), "fon".to_string(),
                    ],
                },
                RegraPasta {
                    id: "emulacao".to_string(),
                    nome: "Emulação & Jogos Retrô".to_string(),
                    destino_chave: "documents".to_string(),
                    subpasta: Some("Emulacao/ROMs".to_string()),
                    extensoes: vec![
                        "nds".to_string(), "3ds".to_string(), "cia".to_string(),
                        "gba".to_string(), "smc".to_string(), "sfc".to_string(),
                        "nes".to_string(), "chd".to_string(), "pbp".to_string(),
                        "cso".to_string(), "sav".to_string(), "srm".to_string(),
                        "dsv".to_string(), "cht".to_string(), "state".to_string(),
                    ],
                },
                RegraPasta {
                    id: "livros_hqs".to_string(),
                    nome: "Livros, Mangás & HQs".to_string(),
                    destino_chave: "documents".to_string(),
                    subpasta: Some("Livros e HQs".to_string()),
                    extensoes: vec![
                        "epub".to_string(), "mobi".to_string(), "azw".to_string(),
                        "azw3".to_string(), "cbr".to_string(), "cbz".to_string(),
                        "fb2".to_string(),
                    ],
                },
                RegraPasta {
                    id: "codigo".to_string(),
                    nome: "Projetos & Código-Fonte".to_string(),
                    destino_chave: "documents".to_string(),
                    subpasta: Some("Codigo".to_string()),
                    extensoes: vec![
                        "rs".to_string(), "py".to_string(), "js".to_string(),
                        "ts".to_string(), "html".to_string(), "css".to_string(),
                        "c".to_string(), "cpp".to_string(), "h".to_string(),
                        "hpp".to_string(), "java".to_string(), "cs".to_string(),
                        "go".to_string(), "php".to_string(), "rb".to_string(),
                        "sql".to_string(), "sh".to_string(), "json".to_string(),
                        "xml".to_string(), "yaml".to_string(), "yml".to_string(),
                        "toml".to_string(), "md".to_string(),
                    ],
                },
                RegraPasta {
                    id: "documentos".to_string(),
                    nome: "Documentos, Textos & Planilhas".to_string(),
                    destino_chave: "documents".to_string(),
                    subpasta: None,
                    extensoes: vec![
                        "pdf".to_string(), "doc".to_string(), "docx".to_string(),
                        "odt".to_string(), "rtf".to_string(), "txt".to_string(),
                        "xlsx".to_string(), "xls".to_string(), "ods".to_string(),
                        "csv".to_string(), "tsv".to_string(), "pptx".to_string(),
                        "ppt".to_string(), "odp".to_string(), "key".to_string(),
                        "ofx".to_string(),
                    ],
                },
                RegraPasta {
                    id: "design_3d".to_string(),
                    nome: "Design Gráfico & Modelos 3D".to_string(),
                    destino_chave: "pictures".to_string(),
                    subpasta: Some("Design e 3D".to_string()),
                    extensoes: vec![
                        "psd".to_string(), "ai".to_string(), "xcf".to_string(),
                        "blend".to_string(), "obj".to_string(), "stl".to_string(),
                        "fbx".to_string(),
                    ],
                },
                RegraPasta {
                    id: "imagens".to_string(),
                    nome: "Imagens & Fotos".to_string(),
                    destino_chave: "pictures".to_string(),
                    subpasta: None,
                    extensoes: vec![
                        "jpg".to_string(), "jpeg".to_string(), "png".to_string(),
                        "gif".to_string(), "bmp".to_string(), "tiff".to_string(),
                        "webp".to_string(), "svg".to_string(), "ico".to_string(),
                        "avif".to_string(),
                    ],
                },
                RegraPasta {
                    id: "musicas".to_string(),
                    nome: "Músicas & Áudios".to_string(),
                    destino_chave: "audio".to_string(),
                    subpasta: None,
                    extensoes: vec![
                        "mp3".to_string(), "wav".to_string(), "flac".to_string(),
                        "aac".to_string(), "ogg".to_string(), "m4a".to_string(),
                        "wma".to_string(), "opus".to_string(), "mid".to_string(),
                    ],
                },
                RegraPasta {
                    id: "videos".to_string(),
                    nome: "Vídeos & Filmes".to_string(),
                    destino_chave: "videos".to_string(),
                    subpasta: None,
                    extensoes: vec![
                        "mp4".to_string(), "mkv".to_string(), "avi".to_string(),
                        "mov".to_string(), "wmv".to_string(), "flv".to_string(),
                        "webm".to_string(), "m4v".to_string(),
                    ],
                },
            ],
        }
    }
}

/// Obtém o caminho do arquivo de regras em %APPDATA%\MiiSideKick\organizador_regras.json
pub fn obter_caminho_regras() -> PathBuf {
    let dir_base = dirs::config_dir()
        .or_else(dirs::data_dir)
        .unwrap_or_else(|| PathBuf::from("."));
    let app_dir = dir_base.join("MiiSideKick");
    if !app_dir.exists() {
        let _ = fs::create_dir_all(&app_dir);
    }
    app_dir.join("organizador_regras.json")
}

/// Carrega as regras de organização com fallback resiliente para os padrões embutidos
pub fn carregar_regras() -> RegrasOrganizador {
    let caminho = obter_caminho_regras();

    if caminho.exists() {
        if let Ok(conteudo) = fs::read_to_string(&caminho) {
            if let Ok(regras) = serde_json::from_str::<RegrasOrganizador>(&conteudo) {
                return regras;
            }
            eprintln!("[Organizador]: organizador_regras.json corrompido. Restaurando fallback.");
        }
    }

    // Se o arquivo não existir ou o parse falhar, gera os padrões e salva atomicamente
    let padrao = RegrasOrganizador::obter_padrao();
    let _ = salvar_regras_atomico(&padrao);
    padrao
}

/// Gravação atômica do arquivo de regras usando arquivo temporário (.tmp) e rename
pub fn salvar_regras_atomico(regras: &RegrasOrganizador) -> Result<(), String> {
    let caminho = obter_caminho_regras();
    let caminho_tmp = caminho.with_extension("tmp");

    let json_bytes = serde_json::to_string_pretty(regras).map_err(|e| e.to_string())?;

    fs::write(&caminho_tmp, json_bytes).map_err(|e| e.to_string())?;
    fs::rename(&caminho_tmp, &caminho).map_err(|e| e.to_string())?;

    Ok(())
}

/// Determina o diretório de destino de um arquivo com base em sua extensão e regras ativas
pub fn encontrar_destino_para_arquivo(
    arquivo: &Path,
    regras: &RegrasOrganizador,
) -> Option<PathBuf> {
    let ext = arquivo
        .extension()
        .and_then(|e| e.to_str())?
        .to_lowercase();

    for regra in &regras.regras {
        let contem_ext = regra
            .extensoes
            .iter()
            .any(|e| e.trim_start_matches('.').eq_ignore_ascii_case(&ext));

        if contem_ext {
            let base = match regra.destino_chave.as_str() {
                "downloads" => dirs::download_dir()
                    .or_else(|| dirs::home_dir().map(|h| h.join("Downloads"))),
                "documents" => dirs::document_dir()
                    .or_else(|| dirs::home_dir().map(|h| h.join("Documents"))),
                "pictures" => dirs::picture_dir()
                    .or_else(|| dirs::home_dir().map(|h| h.join("Pictures"))),
                "audio" => {
                    dirs::audio_dir().or_else(|| dirs::home_dir().map(|h| h.join("Music")))
                }
                "videos" => {
                    dirs::video_dir().or_else(|| dirs::home_dir().map(|h| h.join("Videos")))
                }
                _ => None,
            }?;

            return if let Some(ref sub) = regra.subpasta {
                Some(base.join(sub))
            } else {
                Some(base)
            };
        }
    }

    None
}

// ----------------------------------------------------------------------------
// COMANDOS TAURI (IPC)
// ----------------------------------------------------------------------------

#[tauri::command]
pub fn obter_regras_organizador() -> Result<RegrasOrganizador, String> {
    Ok(carregar_regras())
}

#[tauri::command]
pub fn salvar_regras_organizador(regras: RegrasOrganizador) -> Result<bool, String> {
    salvar_regras_atomico(&regras)?;
    Ok(true)
}

#[tauri::command]
pub fn restaurar_regras_padrao() -> Result<RegrasOrganizador, String> {
    let padrao = RegrasOrganizador::obter_padrao();
    salvar_regras_atomico(&padrao)?;
    Ok(padrao)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classificacao_extensoes_padrao() {
        let regras = RegrasOrganizador::obter_padrao();

        let instalador = Path::new("setup_teste.exe");
        let destino_instalador = encontrar_destino_para_arquivo(instalador, &regras)
            .expect("Deveria encontrar destino para instalador");
        assert!(destino_instalador.to_string_lossy().contains("Instaladores"));

        let foto = Path::new("ferias.png");
        let destino_foto = encontrar_destino_para_arquivo(foto, &regras)
            .expect("Deveria encontrar destino para foto");
        assert!(!destino_foto.to_string_lossy().is_empty());

        let desconhecido = Path::new("arquivo.desconhecido123");
        assert!(encontrar_destino_para_arquivo(desconhecido, &regras).is_none());
    }

    #[test]
    fn test_salvar_e_carregar_atomico() {
        let mut regras = RegrasOrganizador::obter_padrao();
        regras.regras[0].extensoes.push("teste_ext".to_string());

        assert!(salvar_regras_atomico(&regras).is_ok());
        let carregadas = carregar_regras();
        assert!(carregadas.regras[0]
            .extensoes
            .contains(&"teste_ext".to_string()));

        // Restaura
        let _ = restaurar_regras_padrao();
    }
}
