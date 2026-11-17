use image::imageops::FilterType;
use image::{DynamicImage, ImageEncoder};
use std::io::Cursor;
use std::sync::atomic::{AtomicBool, Ordering};
use xcap::Monitor;

/// Trava atômica para prevenir múltiplas capturas concorrentes travando threads e I/O
static CAPTURA_EM_ANDAMENTO: AtomicBool = AtomicBool::new(false);

/// Captura o monitor principal, redimensiona proporcionalmente para no máximo 1024px,
/// comprime em JPEG 75% diretamente em memória RAM e retorna em Base64.
/// NÃO grava nenhum arquivo no disco (respeito aos guardrails de privacidade e performance).
pub fn capturar_tela_base64() -> Result<String, String> {
    if CAPTURA_EM_ANDAMENTO
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        return Err("Uma captura de tela já está sendo processada.".to_string());
    }

    let resultado = executar_captura_e_compressao();
    CAPTURA_EM_ANDAMENTO.store(false, Ordering::SeqCst);
    resultado
}

fn executar_captura_e_compressao() -> Result<String, String> {
    // 1. Obtém os monitores disponíveis
    let monitores = Monitor::all().map_err(|e| format!("Falha ao listar monitores: {}", e))?;
    if monitores.is_empty() {
        return Err("Nenhum monitor detectado no sistema.".to_string());
    }

    // Seleciona o monitor primário ou o primeiro disponível
    let monitor = monitores
        .into_iter()
        .find(|m| m.is_primary())
        .or_else(|| Monitor::all().ok().and_then(|m| m.into_iter().next()))
        .ok_or_else(|| "Monitor primário não encontrado.".to_string())?;

    // 2. Executa a captura do monitor em buffer de imagem RgbaImage
    let rgba_img = monitor
        .capture_image()
        .map_err(|e| format!("Falha ao capturar imagem da tela: {}", e))?;

    let (largura, altura) = rgba_img.dimensions();
    let img_dinamica = DynamicImage::ImageRgba8(rgba_img);

    // 3. Redimensionamento proporcional para no máximo 1024px no lado maior (reduz tokens e banda)
    let max_dim = 1024u32;
    let img_redimensionada = if largura > max_dim || altura > max_dim {
        img_dinamica.resize(max_dim, max_dim, FilterType::Triangle)
    } else {
        img_dinamica
    };

    // 4. Converte para RGB8 e codifica em JPEG com qualidade 75% em memória
    let rgb_img = img_redimensionada.to_rgb8();
    let mut jpeg_buffer = Vec::with_capacity(128 * 1024);
    let mut cursor = Cursor::new(&mut jpeg_buffer);

    let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut cursor, 75);
    encoder
        .write_image(
            rgb_img.as_raw(),
            rgb_img.width(),
            rgb_img.height(),
            image::ExtendedColorType::Rgb8,
        )
        .map_err(|e| format!("Falha ao comprimir captura em JPEG: {}", e))?;

    // 5. Codifica o buffer JPEG diretamente para Base64
    Ok(codificar_base64(&jpeg_buffer))
}

/// Codificação Base64 rápida sem dependências externas adicionais
fn codificar_base64(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut buf = String::with_capacity(bytes.len() * 4 / 3 + 4);

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

    buf
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_codificar_base64() {
        assert_eq!(codificar_base64(b""), "");
        assert_eq!(codificar_base64(b"f"), "Zg==");
        assert_eq!(codificar_base64(b"fo"), "Zm8=");
        assert_eq!(codificar_base64(b"foo"), "Zm9v");
    }
}
