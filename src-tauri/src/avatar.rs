use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

const REACOES_OFICIAIS: [&str; 8] = [
    "normal",
    "feliz",
    "frustrado",
    "assustado",
    "explicando",
    "bocejo",
    "lambendo",
    "raiva",
];

const README_CONTEUDO: &str = r#"====================================================================
           MiiSideKick — Sprites de Avatar Customizado
                     Custom Avatar Sprites
====================================================================

[PORTUGUÊS / BRAZILIAN PORTUGUESE]
Coloque suas imagens personalizadas nesta pasta para substituir o avatar do Mii.
Para que todas as 8 emoções funcionem, nomeie seus arquivos exatamente assim:

1. normal.png      -> Expressão padrão / neutra
2. feliz.png       -> Elogios, alegria, agradecimentos e bom humor
3. frustrado.png   -> Erros, tristeza, decepção ou cansaço
4. assustado.png   -> Surpresas, perguntas estranhas ou sustos
5. explicando.png  -> Dicas, tutoriais ou explicações técnicas
6. bocejo.png      -> Tédio, sono ou preguiça
7. lambendo.png    -> Deboche atrevido ou quando comida for mencionada
8. raiva.png       -> Irritação, estresse ou broncas

DICA DE FALLBACK RESILIENTE:
Se você colocar apenas UMA imagem nesta pasta (como 'normal.png' ou qualquer
arquivo .png/.jpg), o Mii usará essa mesma imagem para todas as emoções!
Formatos recomendados: PNG (fundo transparente), resolução de 256x256 ou 512x512.

--------------------------------------------------------------------

[ENGLISH]
Place your custom character images in this folder to replace Mii's avatar.
To support all 8 emotional states, name your files exactly as follows:

1. normal.png      -> Default / neutral expression
2. feliz.png       -> Happiness, compliments, gratitude, success
3. frustrado.png   -> Errors, sadness, disappointment, fatigue
4. assustado.png   -> Shock, bizarre questions, surprises
5. explicando.png  -> Tips, trivia, technical explanations
6. bocejo.png      -> Boredom, sleepiness, laziness
7. lambendo.png    -> Smug comeback or food cravings
8. raiva.png       -> Anger, raw stress, scolding

RESILIENT FALLBACK TIP:
If you place only ONE image in this folder (e.g. 'normal.png' or any .png/.jpg),
Mii will automatically use that single image for all emotions!
Recommended format: PNG (transparent background), 256x256 or 512x512 pixels.
====================================================================
"#;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StatusAvatar {
    pub modo: String,
    pub imagens_encontradas: usize,
    pub total_esperado: usize,
    pub usa_fallback_estatico: bool,
    pub pasta_caminho: String,
}

/// Obtém o caminho do diretório `%APPDATA%\MiiSideKick\custom_avatar\`
pub fn obter_diretorio_custom_avatar() -> PathBuf {
    let dir_base = dirs::config_dir()
        .or_else(dirs::data_dir)
        .unwrap_or_else(|| PathBuf::from("."));
    let avatar_dir = dir_base.join("MiiSideKick").join("custom_avatar");
    if !avatar_dir.exists() {
        let _ = fs::create_dir_all(&avatar_dir);
    }
    avatar_dir
}

/// Garante a criação da pasta e do arquivo LEIAME_AVATAR.txt bilíngue
pub fn inicializar_diretorio_avatar() {
    let dir = obter_diretorio_custom_avatar();
    let readme_path = dir.join("LEIAME_AVATAR.txt");
    if !readme_path.exists() {
        let _ = fs::write(readme_path, README_CONTEUDO);
    }
}

/// Lê uma imagem do disco e a converte para Data URI Base64 (garante compatibilidade total no WebView)
fn converter_imagem_para_data_uri(caminho: &Path) -> Option<String> {
    if !caminho.exists() || !caminho.is_file() {
        return None;
    }

    let bytes = fs::read(caminho).ok()?;
    let ext = caminho
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("png")
        .to_lowercase();

    let mime = match ext.as_str() {
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "gif" => "image/gif",
        _ => "image/png",
    };

    let mut buf = String::with_capacity(bytes.len() * 4 / 3 + 64);
    buf.push_str("data:");
    buf.push_str(mime);
    buf.push_str(";base64,");

    // Codificação Base64 simples e sem dependência adicional
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut chunks = bytes.chunks_exact(3);
    for chunk in &mut chunks {
        let b0 = chunk[0] as usize;
        let b1 = chunk[1] as usize;
        let b2 = chunk[2] as usize;
        let n = (b0 << 16) | (b1 << 8) | b2;
        buf.push(ALPHABET[(n >> 18) & 63] as char);
        buf.push(ALPHABET[(n >> 12) & 63] as char);
        buf.push(ALPHABET[(n >> 6) & 63] as char);
        buf.push(ALPHABET[n & 63] as char);
    }
    let rem = chunks.remainder();
    if rem.len() == 1 {
        let n = (rem[0] as usize) << 16;
        buf.push(ALPHABET[(n >> 18) & 63] as char);
        buf.push(ALPHABET[(n >> 12) & 63] as char);
        buf.push('=');
        buf.push('=');
    } else if rem.len() == 2 {
        let n = ((rem[0] as usize) << 16) | ((rem[1] as usize) << 8);
        buf.push(ALPHABET[(n >> 18) & 63] as char);
        buf.push(ALPHABET[(n >> 12) & 63] as char);
        buf.push(ALPHABET[(n >> 6) & 63] as char);
        buf.push('=');
    }

    Some(buf)
}

/// Varre a pasta custom_avatar procurando imagens válidas (.png, .jpg, .jpeg, .webp)
fn varrer_arquivos_avatar_customizado() -> HashMap<String, PathBuf> {
    let dir = obter_diretorio_custom_avatar();
    let mut mapa = HashMap::new();

    if let Ok(entries) = fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    let ext_lower = ext.to_lowercase();
                    if ["png", "jpg", "jpeg", "webp"].contains(&ext_lower.as_str()) {
                        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                            let stem_lower = stem.to_lowercase();
                            mapa.insert(stem_lower, path);
                        }
                    }
                }
            }
        }
    }

    mapa
}

/// Constrói o mapa final de Data URIs para as 8 reações aplicando a política de fallback resiliente
pub fn resolver_mapa_avatares(modo: &str) -> HashMap<String, String> {
    let mut resultado = HashMap::new();

    if modo == "custom" {
        let arquivos_locais = varrer_arquivos_avatar_customizado();

        if !arquivos_locais.is_empty() {
            // Busca imagem estática única como curinga caso nem todos os 8 sprites estejam presentes
            let curinga_opt = arquivos_locais
                .get("normal")
                .or_else(|| arquivos_locais.get("avatar"))
                .or_else(|| arquivos_locais.get("default"))
                .or_else(|| arquivos_locais.values().next())
                .cloned();

            let data_uri_curinga = curinga_opt
                .as_ref()
                .and_then(|p| converter_imagem_para_data_uri(p));

            for &reacao in &REACOES_OFICIAIS {
                if let Some(caminho) = arquivos_locais.get(reacao) {
                    if let Some(data_uri) = converter_imagem_para_data_uri(caminho) {
                        resultado.insert(reacao.to_string(), data_uri);
                        continue;
                    }
                }

                // Fallback para o curinga local se existir
                if let Some(ref fallback_uri) = data_uri_curinga {
                    resultado.insert(reacao.to_string(), fallback_uri.clone());
                }
            }

            if !resultado.is_empty() {
                return resultado;
            }
        }
    }

    // Modo "classico" ou "nostalgico" ou fallback seguro:
    // Retorna vazio para que o frontend utilize os assets embutidos empacotados pelo bundler
    for &reacao in &REACOES_OFICIAIS {
        resultado.insert(reacao.to_string(), String::new());
    }

    resultado
}

// ----------------------------------------------------------------------------
// COMANDOS TAURI (IPC)
// ----------------------------------------------------------------------------

#[tauri::command]
pub fn obter_mapa_avatares(modo: Option<String>) -> HashMap<String, String> {
    let modo_final = modo.unwrap_or_else(|| "classico".to_string());
    resolver_mapa_avatares(&modo_final)
}

#[tauri::command]
pub fn obter_status_avatar_customizado() -> StatusAvatar {
    inicializar_diretorio_avatar();
    let dir = obter_diretorio_custom_avatar();
    let arquivos = varrer_arquivos_avatar_customizado();

    let encontradas = REACOES_OFICIAIS
        .iter()
        .filter(|&&r| arquivos.contains_key(r))
        .count();

    let usa_fallback_estatico = encontradas < 8 && !arquivos.is_empty();

    StatusAvatar {
        modo: "custom".to_string(),
        imagens_encontradas: se_arquivos_existem(encontradas, arquivos.len()),
        total_esperado: 8,
        usa_fallback_estatico,
        pasta_caminho: dir.to_string_lossy().to_string(),
    }
}

fn se_arquivos_existem(encontradas: usize, total_arquivos: usize) -> usize {
    if encontradas > 0 {
        encontradas
    } else {
        total_arquivos
    }
}

#[tauri::command]
pub fn abrir_pasta_avatar_customizado() -> Result<(), String> {
    inicializar_diretorio_avatar();
    let dir = obter_diretorio_custom_avatar();

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&dir)
            .spawn()
            .map_err(|e| e.to_string())?;
        return Ok(());
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&dir)
            .spawn()
            .map_err(|e| e.to_string())?;
        return Ok(());
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&dir)
            .spawn()
            .map_err(|e| e.to_string())?;
        return Ok(());
    }

    #[allow(unreachable_code)]
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inicializacao_readme() {
        inicializar_diretorio_avatar();
        let dir = obter_diretorio_custom_avatar();
        let readme = dir.join("LEIAME_AVATAR.txt");
        assert!(readme.exists(), "LEIAME_AVATAR.txt deveria ter sido criado");

        let conteudo = fs::read_to_string(readme).unwrap();
        assert!(conteudo.contains("normal.png"));
        assert!(conteudo.contains("CUSTOM AVATAR SPRITES") || conteudo.contains("Custom Avatar Sprites"));
    }

    #[test]
    fn test_fallback_mapa_quando_vazio() {
        let mapa = resolver_mapa_avatares("classico");
        assert_eq!(mapa.len(), 8);
        assert!(mapa.contains_key("normal"));
        assert!(mapa.contains_key("feliz"));
    }
}
