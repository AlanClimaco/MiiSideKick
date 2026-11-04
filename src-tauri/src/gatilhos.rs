use std::cmp::Ordering;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tauri::AppHandle;

use crate::comandos::{self, disparar_notificacao};

/// Lista de curiosidades locais para fallback offline ou quando não há chave de API.
pub const CURIOSIDADES_OFFLINE: &[(&str, &str)] = &[
    (
        "Em 1518, uma misteriosa epidemia fez centenas de pessoas dançarem sem parar por semanas em Estrasburgo.",
        "explicando",
    ),
    (
        "O inventor do frisbee foi cremado e suas cinzas foram transformadas em um frisbee após a sua morte.",
        "normal",
    ),
    (
        "Bananas são levemente radioativas por conterem potássio-40. Mas você precisaria comer 10 milhões de uma vez para sofrer algum dano.",
        "feliz",
    ),
    (
        "Corvos conseguem reconhecer rostos humanos e guardam rancor de pessoas específicas por anos.",
        "assustado",
    ),
    (
        "A guerra mais curta da história durou apenas 38 minutos, travada entre o Reino Unido e Zanzibar em 1896.",
        "normal",
    ),
    (
        "O mel puro nunca estraga. Arqueólogos acharam potes de mel de 3.000 anos em tumbas egípcias perfeitamente comestíveis.",
        "feliz",
    ),
    (
        "Polvos têm três corações e o sangue deles é azul por causa da hemocianina à base de cobre.",
        "explicando",
    ),
    (
        "Em 1983, Stanislav Petrov evitou uma guerra nuclear mundial ao deduzir que o alerta de ataque no radar era um falso alarme.",
        "assustado",
    ),
    (
        "Tubarões já existiam na Terra antes mesmo de existirem árvores e anéis em Saturno.",
        "normal",
    ),
    (
        "O criador da prova do bafômetro foi preso anos depois por dirigir embriagado.",
        "bocejo",
    ),
    (
        "Formigas nunca dormem no sentido humano e conseguem carregar até 50 vezes o próprio peso.",
        "explicando",
    ),
    (
        "Cavalos-marinhos machos são os que engravidam e dão à luz centenas de filhotes.",
        "feliz",
    ),
];

/// Compara versão remota com a versão local da aplicação seguindo semver simples.
pub fn comparar_versoes(remota: &str, local: &str) -> Ordering {
    let parse = |v: &str| -> Vec<u64> {
        v.trim_start_matches(|c| c == 'v' || c == 'V')
            .split('.')
            .filter_map(|s| s.split('-').next().unwrap_or("").parse::<u64>().ok())
            .collect()
    };

    let r = parse(remota);
    let l = parse(local);

    for (a, b) in r.iter().zip(l.iter()) {
        if a != b {
            return a.cmp(b);
        }
    }
    r.len().cmp(&l.len())
}

#[derive(serde::Deserialize)]
struct GithubRelease {
    tag_name: String,
    html_url: String,
}

/// Verifica se há nova versão publicada no repositório do GitHub.
pub fn checar_atualizacoes_github(app: &AppHandle) {
    let url = "https://api.github.com/repos/AlanClimaco/MiiSideKick/releases/latest";
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(8))
        .build();

    let client = match client {
        Ok(c) => c,
        Err(_) => return,
    };

    let response = client
        .get(url)
        .header("User-Agent", "MiiSideKick-App")
        .header("Accept", "application/vnd.github.v3+json")
        .send();

    if let Ok(res) = response {
        if res.status().is_success() {
            if let Ok(release) = res.json::<GithubRelease>() {
                let versao_local = env!("CARGO_PKG_VERSION");
                if comparar_versoes(&release.tag_name, versao_local) == Ordering::Greater {
                    let msg = format!(
                        "Nova versão disponível: {}!\nClique abaixo para conferir as novidades.",
                        release.tag_name
                    );
                    disparar_notificacao(
                        app,
                        &msg,
                        "feliz",
                        Some(release.html_url),
                        Some("Ver Atualização".to_string()),
                        false,
                        true,
                    );
                }
            }
        }
    }
}

/// Verifica se a lixeira do Windows contém mais de 20 itens ou mais de 500 MB.
#[cfg(target_os = "windows")]
pub fn verificar_lixeira_cheia() -> bool {
    #[repr(C)]
    #[derive(Default)]
    #[allow(non_snake_case)]
    struct SHQUERYRBINFO {
        cbSize: u32,
        i64Size: i64,
        i64NumItems: i64,
    }

    #[link(name = "shell32")]
    extern "system" {
        fn SHQueryRecycleBinW(pszRootPath: *const u16, pSHQueryRBInfo: *mut SHQUERYRBINFO) -> i32;
    }

    let mut info = SHQUERYRBINFO {
        cbSize: std::mem::size_of::<SHQUERYRBINFO>() as u32,
        i64Size: 0,
        i64NumItems: 0,
    };

    unsafe {
        if SHQueryRecycleBinW(std::ptr::null(), &mut info) == 0 {
            // Mais de 20 itens ou mais de 500 MB (524.288.000 bytes)
            return info.i64NumItems >= 20 || info.i64Size >= 500 * 1024 * 1024;
        }
    }
    false
}

#[cfg(not(target_os = "windows"))]
pub fn verificar_lixeira_cheia() -> bool {
    if let Some(home) = dirs::home_dir() {
        let trash = home.join(".local/share/Trash/files");
        if let Ok(entries) = std::fs::read_dir(trash) {
            return entries.count() >= 20;
        }
    }
    false
}

/// Verifica se a pasta Downloads contém 15 ou mais arquivos soltos no nível raiz.
pub fn verificar_downloads_acumulados() -> bool {
    if let Some(down) = dirs::download_dir() {
        if let Ok(entries) = std::fs::read_dir(down) {
            let mut count = 0;
            for entry in entries.flatten() {
                if let Ok(ft) = entry.file_type() {
                    if ft.is_file() {
                        count += 1;
                        if count >= 15 {
                            return true;
                        }
                    }
                }
            }
        }
    }
    false
}

/// Gera ou seleciona uma curiosidade para o alerta proativo.
pub fn obter_curiosidade_proativa(
    api_key: &str,
    limite_chars: u32,
    personalidade: &str,
    indice_fallback: usize,
) -> (String, String) {
    if !api_key.trim().is_empty() {
        let tom = match personalidade {
            "hostil" | "sarcastico" => {
                "Adote um tom de humor negro, mórbido, ácido e sarcasmo inteligente (fatos bizarros, mortes cômicas ou ironias da história). ATENÇÃO: NUNCA xingue nem ofenda o usuário diretamente, pois ele não fez pergunta alguma; direcione o cinismo e deboche exclusivamente ao fato narrado."
            }
            "educado" => "Seja extremamente educado, formal, paciente e gentil.",
            "engracado" => "Seja muito bem-humorado, brincalhão, faça piadinhas leves e trocadilhos.",
            "direto" => "Seja 100% sério, técnico, conciso e direto ao ponto.",
            _ => "Seja equilibrado, curioso, amigável e descontraído.",
        };

        let prompt = format!(
            "Você é o MiiSideKick. Conte uma curiosidade curta, curiosa e fascinante em português (ciência, história, fatos bizarros ou ironias cotidianas). \
            Instrução de tom da personalidade: {tom} \
            Tamanho máximo ESTRITO: {limite_chars} caracteres. \
            Retorne ESTRITAMENTE um JSON no formato: {{\"reacao\": \"<normal|feliz|explicando|bocejo|lambendo|raiva>\", \"resposta\": \"<texto>\"}} sem markdown."
        );

        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash-lite:generateContent?key={}",
            api_key.trim()
        );

        let payload = serde_json::json!({
            "contents": [
                {
                    "role": "user",
                    "parts": [{ "text": prompt }]
                }
            ],
            "generationConfig": {
                "responseMimeType": "application/json"
            }
        });

        if let Ok(client) = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(6))
            .build()
        {
            if let Ok(res) = client.post(&url).json(&payload).send() {
                if res.status().is_success() {
                    if let Ok(val) = res.json::<serde_json::Value>() {
                        if let Some(txt) = val["candidates"][0]["content"]["parts"][0]["text"].as_str() {
                            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(txt) {
                                if let Some(resposta) = parsed["resposta"].as_str() {
                                    let reacao = parsed["reacao"]
                                        .as_str()
                                        .map(|r| comandos::validar_reacao(Some(r)))
                                        .unwrap_or("explicando");
                                    return (resposta.to_string(), reacao.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Fallback offline
    let idx = indice_fallback % CURIOSIDADES_OFFLINE.len();
    let (texto, reacao) = CURIOSIDADES_OFFLINE[idx];
    (texto.to_string(), reacao.to_string())
}

struct EstadoMotor {
    ultimo_disparo_global: Instant,
    ultimo_alerta_lixeira: Option<Instant>,
    ultimo_alerta_downloads: Option<Instant>,
    ultimo_alerta_bem_estar: Instant,
    ultimo_alerta_curiosidade: Instant,
    ultimo_check_updater: Option<Instant>,
    contador_bem_estar: usize,
    indice_curiosidade: usize,
}

static MOTOR_INICIADO: Mutex<bool> = Mutex::new(false);

/// Inicia o motor proativo em background.
pub fn iniciar_motor_proativo(app: AppHandle) {
    if let Ok(mut iniciado) = MOTOR_INICIADO.lock() {
        if *iniciado {
            return;
        }
        *iniciado = true;
    }

    std::thread::spawn(move || {
        let mut estado = EstadoMotor {
            ultimo_disparo_global: Instant::now(),
            ultimo_alerta_lixeira: None,
            ultimo_alerta_downloads: None,
            ultimo_alerta_bem_estar: Instant::now(),
            ultimo_alerta_curiosidade: Instant::now(),
            ultimo_check_updater: None,
            contador_bem_estar: 0,
            indice_curiosidade: 0,
        };

        // Pequeno atraso inicial de 8 segundos antes da primeira verificação de update
        std::thread::sleep(Duration::from_secs(8));

        loop {
            let config = comandos::carregar_configuracoes();

            // Intervalos baseados na frequência configurada
            let (cooldown_global_minutos, bem_estar_minutos, curiosidade_minutos) =
                match config.frequencia_alertas.to_lowercase().as_str() {
                    "discreto" => (30, 90, 120),
                    "frequente" => (8, 30, 35),
                    _ => (15, 50, 60), // equilibrado (padrão)
                };

            let agora = Instant::now();

            // 1. Auto-updater assíncrono (verifica a cada 6 horas)
            let deve_checar_update = match estado.ultimo_check_updater {
                None => true,
                Some(inst) => agora.duration_since(inst) >= Duration::from_secs(6 * 3600),
            };

            if deve_checar_update {
                estado.ultimo_check_updater = Some(agora);
                checar_atualizacoes_github(&app);
            }

            // 2. Cooldown anti-spam global entre alertas na tela
            let em_cooldown_global = agora.duration_since(estado.ultimo_disparo_global)
                < Duration::from_secs(cooldown_global_minutos * 60);

            if !em_cooldown_global {
                let mut disparou = false;

                // 2.1 Categoria 1: Manutenção do Sistema (Lixeira e Downloads)
                if config.alerta_manutencao && !disparou {
                    let pode_lixeira = match estado.ultimo_alerta_lixeira {
                        None => true,
                        Some(inst) => agora.duration_since(inst) >= Duration::from_secs(2 * 3600),
                    };

                    if pode_lixeira && verificar_lixeira_cheia() {
                        estado.ultimo_alerta_lixeira = Some(agora);
                        estado.ultimo_disparo_global = agora;
                        disparou = true;
                        disparar_notificacao(
                            &app,
                            "A lixeira do sistema está bem cheia! Quer fazer uma faxina rápida?",
                            "explicando",
                            None,
                            None,
                            false,
                            false,
                        );
                    }

                    if !disparou {
                        let pode_downloads = match estado.ultimo_alerta_downloads {
                            None => true,
                            Some(inst) => agora.duration_since(inst) >= Duration::from_secs(2 * 3600),
                        };

                        if pode_downloads && verificar_downloads_acumulados() {
                            estado.ultimo_alerta_downloads = Some(agora);
                            estado.ultimo_disparo_global = agora;
                            disparou = true;
                            disparar_notificacao(
                                &app,
                                "Sua pasta de Downloads acumulou muitos arquivos. Posso organizá-los se quiser!",
                                "feliz",
                                None,
                                None,
                                false,
                                false,
                            );
                        }
                    }
                }

                // 2.2 Categoria 2: Interação & Bem-Estar (Saúde/Água/Postura e Curiosidades)
                if config.alerta_interacao_bem_estar && !disparou {
                    let tempo_bem_estar = agora.duration_since(estado.ultimo_alerta_bem_estar);
                    if tempo_bem_estar >= Duration::from_secs(bem_estar_minutos * 60) {
                        estado.ultimo_alerta_bem_estar = agora;
                        estado.ultimo_disparo_global = agora;
                        disparou = true;

                        let msg = if estado.contador_bem_estar % 2 == 0 {
                            "Hora de se hidratar! Beba um pouco de água para manter o foco."
                        } else {
                            "Lembrete de postura! Ajuste a coluna e relaxe os ombros por um instante."
                        };
                        estado.contador_bem_estar += 1;

                        disparar_notificacao(&app, msg, "feliz", None, None, false, false);
                    }

                    if !disparou {
                        let tempo_curiosidade = agora.duration_since(estado.ultimo_alerta_curiosidade);
                        if tempo_curiosidade >= Duration::from_secs(curiosidade_minutos * 60) {
                            estado.ultimo_alerta_curiosidade = agora;
                            estado.ultimo_disparo_global = agora;

                            let (texto, reacao) = obter_curiosidade_proativa(
                                &config.gemini_api_key,
                                config.limite_caracteres,
                                &config.personalidade,
                                estado.indice_curiosidade,
                            );
                            estado.indice_curiosidade += 1;

                            disparar_notificacao(&app, &texto, &reacao, None, None, true, false);
                        }
                    }
                }
            }

            // Intervalo de verificação da thread (a cada 30 segundos)
            std::thread::sleep(Duration::from_secs(30));
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comparar_versoes() {
        assert_eq!(comparar_versoes("v1.4.0", "1.4.0"), Ordering::Equal);
        assert_eq!(comparar_versoes("1.4.1", "1.4.0"), Ordering::Greater);
        assert_eq!(comparar_versoes("v2.0.0", "1.9.9"), Ordering::Greater);
        assert_eq!(comparar_versoes("v1.3.5", "1.4.0"), Ordering::Less);
        assert_eq!(comparar_versoes("1.4.0", "v1.4.1"), Ordering::Less);
        assert_eq!(comparar_versoes("1.4.0-beta", "1.4.0"), Ordering::Equal);
    }

    #[test]
    fn test_curiosidades_offline_validas() {
        assert!(!CURIOSIDADES_OFFLINE.is_empty());
        for (texto, reacao) in CURIOSIDADES_OFFLINE {
            assert!(!texto.trim().is_empty());
            assert!(texto.chars().count() <= 200);
            assert!(["normal", "feliz", "frustrado", "assustado", "explicando", "bocejo", "lambendo", "raiva"]
                .contains(reacao));
        }
    }

    #[test]
    fn test_obter_curiosidade_proativa_offline() {
        let (texto, reacao) = obter_curiosidade_proativa("", 70, "espelho", 0);
        assert!(!texto.is_empty());
        assert!(!reacao.is_empty());

        let (texto_hostil, reacao_hostil) = obter_curiosidade_proativa("", 70, "hostil", 1);
        assert!(!texto_hostil.is_empty());
        assert!(!reacao_hostil.is_empty());
    }
}
