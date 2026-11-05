use std::cmp::Ordering;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tauri::AppHandle;

use crate::comandos::{self, disparar_notificacao, disparar_notificacao_com_acao};

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

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct UpdateCheckResult {
    pub tem_atualizacao: bool,
    pub versao_atual: String,
    pub versao_nova: Option<String>,
    pub url_release: Option<String>,
    pub mensagem: String,
}

#[derive(serde::Deserialize)]
struct GithubRelease {
    tag_name: String,
    html_url: String,
}

/// Verifica manualmente se há nova versão publicada no repositório do GitHub.
pub fn verificar_atualizacoes_manual(app: &AppHandle) -> Result<UpdateCheckResult, String> {
    let cfg = crate::comandos::carregar_configuracoes();
    let is_en = cfg.idioma == "en";
    let versao_local = env!("CARGO_PKG_VERSION");
    let url = "https://api.github.com/repos/AlanClimaco/MiiSideKick/releases/latest";

    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(8))
        .build()
        .map_err(|e| e.to_string())?;

    let response = client
        .get(url)
        .header("User-Agent", "MiiSideKick-App")
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .map_err(|e| {
            if is_en {
                format!("Failed to connect to update server: {}", e)
            } else {
                format!("Falha ao conectar ao servidor de atualizações: {}", e)
            }
        })?;

    if response.status().is_success() {
        if let Ok(release) = response.json::<GithubRelease>() {
            if comparar_versoes(&release.tag_name, versao_local) == Ordering::Greater {
                let msg = if is_en {
                    format!(
                        "New version available: {}!\nClick below to check out what's new.",
                        release.tag_name
                    )
                } else {
                    format!(
                        "Nova versão disponível: {}!\nClique abaixo para conferir as novidades.",
                        release.tag_name
                    )
                };
                let rotulo = if is_en { "View Update" } else { "Ver Atualização" };
                disparar_notificacao(
                    app,
                    &msg,
                    "feliz",
                    Some(release.html_url.clone()),
                    Some(rotulo.to_string()),
                    false,
                    true,
                );

                let ret_msg = if is_en {
                    format!("New version available: {}!", release.tag_name)
                } else {
                    format!("Nova versão disponível: {}!", release.tag_name)
                };

                return Ok(UpdateCheckResult {
                    tem_atualizacao: true,
                    versao_atual: versao_local.to_string(),
                    versao_nova: Some(release.tag_name),
                    url_release: Some(release.html_url),
                    mensagem: ret_msg,
                });
            } else {
                let ret_msg = if is_en {
                    format!("You are already on the latest version (v{})!", versao_local)
                } else {
                    format!("Você já está na versão mais recente (v{})!", versao_local)
                };

                return Ok(UpdateCheckResult {
                    tem_atualizacao: false,
                    versao_atual: versao_local.to_string(),
                    versao_nova: None,
                    url_release: None,
                    mensagem: ret_msg,
                });
            }
        }
    }

    let erro = if is_en {
        "Unable to check for updates. Please try again later."
    } else {
        "Não foi possível verificar atualizações no momento."
    };
    Err(erro.to_string())
}

/// Verifica se há nova versão publicada no repositório do GitHub.
pub fn checar_atualizacoes_github(app: &AppHandle, is_en: bool) {
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
                    let msg = if is_en {
                        format!(
                            "New version available: {}!\nClick below to check out what's new.",
                            release.tag_name
                        )
                    } else {
                        format!(
                            "Nova versão disponível: {}!\nClique abaixo para conferir as novidades.",
                            release.tag_name
                        )
                    };
                    let rotulo = if is_en { "View Update" } else { "Ver Atualização" };
                    disparar_notificacao(
                        app,
                        &msg,
                        "feliz",
                        Some(release.html_url),
                        Some(rotulo.to_string()),
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

/// Gera uma curiosidade para o alerta proativo consultando a API do Gemini.
/// Se a requisição for bem-sucedida, retorna Some((texto, reacao)).
/// Se falhar por qualquer motivo (sem chave, sem internet, erro de status ou cota), retorna None.
pub fn obter_curiosidade_proativa(
    api_key: &str,
    limite_chars: u32,
    personalidade: &str,
    idioma: &str,
    gostos_usuario: &str,
) -> Option<(String, String)> {
    let api_key_trim = api_key.trim();
    if api_key_trim.is_empty() {
        return None;
    }

    let is_en = idioma.trim().eq_ignore_ascii_case("en");
    let gostos = gostos_usuario.trim();
    let injetar_gostos = if !gostos.is_empty() {
        let sorteio = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .subsec_nanos() % 100;
        sorteio < 25 // 25% de chance de personalizar curiosidades
    } else {
        false
    };

    let prompt = if is_en {
        let foco = if injetar_gostos {
            format!("related to one of the user's interests: \"{gostos}\"")
        } else {
            "science, history, bizarre facts, or everyday ironies".to_string()
        };
        let t = match personalidade {
            "hostil" | "sarcastico" => {
                "Adopt a tone of dark humor, morbid wit, sarcasm, and irony (bizarre historical facts, ironic deaths, cosmic absurdities). NOTE: NEVER insult or curse at the user directly, as they did not ask any question; direct cynicism and sarcasm strictly at the story or fact itself."
            }
            "educado" => "Be extremely polite, formal, patient, and courteous.",
            "engracado" => "Be funny, witty, playful, and crack lighthearted jokes and puns.",
            "direto" => "Be 100% serious, technical, concise, and straight to the point.",
            _ => "Be balanced, curious, friendly, and engaging.",
        };
        format!(
            "You are MiiSideKick. Tell a short, fascinating curiosity in English ({foco}). \
            Tone instruction: {t} \
            STRICT maximum length: {limite_chars} characters. \
            STRICTLY return a JSON in the format: {{\"reacao\": \"<normal|feliz|explicando|bocejo|lambendo|raiva>\", \"resposta\": \"<text>\"}} without markdown."
        )
    } else {
        let foco = if injetar_gostos {
            format!("relacionada a um dos interesses do usuário: \"{gostos}\"")
        } else {
            "ciência, história, fatos bizarros ou ironias cotidianas".to_string()
        };
        let t = match personalidade {
            "hostil" | "sarcastico" => {
                "Adote um tom de humor negro, mórbido, ácido e sarcasmo inteligente (fatos bizarros, mortes cômicas ou ironias da história). ATENÇÃO: NUNCA xingue nem ofenda o usuário diretamente, pois ele não fez pergunta alguma; direcione o cinismo e deboche exclusivamente ao fato narrado."
            }
            "educado" => "Seja extremamente educado, formal, paciente e gentil.",
            "engracado" => "Seja muito bem-humorado, brincalhão, faça piadinhas leves e trocadilhos.",
            "direto" => "Seja 100% sério, técnico, conciso e direto ao ponto.",
            _ => "Seja equilibrado, curioso, amigável e descontraído.",
        };
        format!(
            "Você é o MiiSideKick. Conte uma curiosidade curta, curiosa e fascinante em português ({foco}). \
            Instrução de tom da personalidade: {t} \
            Tamanho máximo ESTRITO: {limite_chars} caracteres. \
            Retorne ESTRITAMENTE um JSON no formato: {{\"reacao\": \"<normal|feliz|explicando|bocejo|lambendo|raiva>\", \"resposta\": \"<texto>\"}} sem markdown."
        )
    };

    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash-lite:generateContent?key={}",
        api_key_trim
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

    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(6))
        .build()
        .ok()?;

    let res = client.post(&url).json(&payload).send().ok()?;
    if !res.status().is_success() {
        return None;
    }

    let val = res.json::<serde_json::Value>().ok()?;
    let raw_text = val["candidates"][0]["content"]["parts"][0]["text"].as_str()?;
    let clean_json = if raw_text.starts_with("```json") {
        raw_text.trim_start_matches("```json").trim_end_matches("```").trim()
    } else if raw_text.starts_with("```") {
        raw_text.trim_start_matches("```").trim_end_matches("```").trim()
    } else {
        raw_text.trim()
    };
    let parsed = serde_json::from_str::<serde_json::Value>(clean_json).ok()?;
    let resposta = parsed["resposta"].as_str()?;
    if resposta.trim().is_empty() {
        return None;
    }

    let reacao = parsed["reacao"]
        .as_str()
        .map(|r| comandos::validar_reacao(Some(r)))
        .unwrap_or("explicando");

    Some((resposta.trim().to_string(), reacao.to_string()))
}

struct EstadoMotor {
    ultimo_disparo_global: Instant,
    ultimo_alerta_lixeira: Option<Instant>,
    ultimo_alerta_downloads: Option<Instant>,
    ultimo_alerta_bem_estar: Instant,
    ultimo_alerta_curiosidade: Instant,
    ultimo_check_updater: Option<Instant>,
    ultimo_alerta_ram: Option<Instant>,
    leituras_ram_alta: usize,
    contador_bem_estar: usize,
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
            ultimo_alerta_ram: None,
            leituras_ram_alta: 0,
            contador_bem_estar: 0,
        };

        // Pequeno atraso inicial de 8 segundos antes da primeira verificação de update
        std::thread::sleep(Duration::from_secs(8));

        let mut segundos_acumulados = 0u64;

        loop {
            std::thread::sleep(Duration::from_secs(1));

            // Esvazia fila de notificações se não estiver em tela cheia nem em período de tolerância
            if !crate::sistema::atualizar_e_checar_foco() && crate::sistema::tamanho_fila() > 0 {
                if let Some(pendente) = crate::sistema::desenfileirar_notificacao() {
                    eprintln!(
                        "[Sistema]: Descarregando notificação pendente pós-tela cheia: {}",
                        pendente.texto
                    );
                    crate::comandos::exibir_janela_notificacao(&app, &pendente);
                }
            }

            segundos_acumulados += 1;
            if segundos_acumulados < 30 {
                continue;
            }
            segundos_acumulados = 0;

            let config = comandos::carregar_configuracoes();
            let is_en = config.idioma == "en";

            // 0. Monitoramento contínuo de RAM do sistema
            let mut sys = sysinfo::System::new();
            sys.refresh_memory();
            let total_ram = sys.total_memory();
            let used_ram = sys.used_memory();
            let pct_ram = if total_ram > 0 {
                (used_ram as f64 / total_ram as f64) * 100.0
            } else {
                0.0
            };

            if pct_ram >= 85.0 {
                estado.leituras_ram_alta = estado.leituras_ram_alta.saturating_add(1);
            } else {
                estado.leituras_ram_alta = 0;
            }

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
                checar_atualizacoes_github(&app, is_en);
            }

            // 2. Cooldown anti-spam global entre alertas na tela
            let em_cooldown_global = agora.duration_since(estado.ultimo_disparo_global)
                < Duration::from_secs(cooldown_global_minutos * 60);

            if !em_cooldown_global {
                let mut disparou = false;

                // 2.1 Categoria 1: Manutenção do Sistema (RAM, Lixeira e Downloads)
                if config.alerta_manutencao && !disparou {
                    // Verificação de RAM alta contínua (>= 85% por pelo menos 2 checagens consecutivas)
                    let pode_ram = match estado.ultimo_alerta_ram {
                        None => true,
                        Some(inst) => agora.duration_since(inst) >= Duration::from_secs(30 * 60),
                    };

                    if pode_ram && estado.leituras_ram_alta >= 2 {
                        estado.ultimo_alerta_ram = Some(agora);
                        estado.ultimo_disparo_global = agora;
                        disparou = true;
                        let msg = if is_en {
                            "RAM usage reached 85%. Would you like to optimize memory now?"
                        } else {
                            "O uso de memória RAM está em 85%. Deseja otimizar o sistema agora?"
                        };
                        let rotulo = if is_en { "Optimize RAM" } else { "Otimizar RAM" };
                        disparar_notificacao_com_acao(
                            &app,
                            msg,
                            "normal",
                            Some("limpar_cache_ram".to_string()),
                            Some(rotulo.to_string()),
                        );
                    }

                    if !disparou {
                        let pode_lixeira = match estado.ultimo_alerta_lixeira {
                        None => true,
                        Some(inst) => agora.duration_since(inst) >= Duration::from_secs(2 * 3600),
                    };

                    if pode_lixeira && verificar_lixeira_cheia() {
                        estado.ultimo_alerta_lixeira = Some(agora);
                        estado.ultimo_disparo_global = agora;
                        disparou = true;
                        let msg = if is_en {
                            "The Recycle Bin is getting full! Want to do a quick cleanup?"
                        } else {
                            "A lixeira do sistema está bem cheia! Quer fazer uma faxina rápida?"
                        };
                        disparar_notificacao(
                            &app,
                            msg,
                            "explicando",
                            None,
                            None,
                            false,
                            true,
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
                            let msg = if is_en {
                                "Your Downloads folder has piled up. I can organize it if you want!"
                            } else {
                                "Sua pasta de Downloads acumulou muitos arquivos. Posso organizá-los se quiser!"
                            };
                            disparar_notificacao(
                                &app,
                                msg,
                                "feliz",
                                None,
                                None,
                                false,
                                true,
                            );
                        }
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

                        let msg = if is_en {
                            if estado.contador_bem_estar % 2 == 0 {
                                "Time to hydrate! Drink some water to keep your focus."
                            } else {
                                "Posture check! Straighten your back and relax your shoulders for a moment."
                            }
                        } else if estado.contador_bem_estar % 2 == 0 {
                            "Hora de se hidratar! Beba um pouco de água para manter o foco."
                        } else {
                            "Lembrete de postura! Ajuste a coluna e relaxe os ombros por um instante."
                        };
                        estado.contador_bem_estar += 1;

                        disparar_notificacao(&app, msg, "feliz", None, None, false, true);
                    }

                    if !disparou {
                        let tempo_curiosidade = agora.duration_since(estado.ultimo_alerta_curiosidade);
                        if tempo_curiosidade >= Duration::from_secs(curiosidade_minutos * 60) {
                            estado.ultimo_alerta_curiosidade = agora;

                            if let Some((texto, reacao)) = obter_curiosidade_proativa(
                                &config.gemini_api_key,
                                config.limite_caracteres,
                                &config.personalidade,
                                &config.idioma,
                                &config.gostos_usuario,
                            ) {
                                estado.ultimo_disparo_global = agora;

                                let payload_historico = crate::comandos::NotificacaoPayload {
                                    texto: texto.clone(),
                                    tipo: reacao.clone(),
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
                                crate::comandos::registrar_historico_ia(payload_historico);
                                crate::atualizar_menu_bandeja(&app);

                                disparar_notificacao(&app, &texto, &reacao, None, None, true, true);
                            }
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
        assert_eq!(comparar_versoes("v1.5.0", "1.5.0"), Ordering::Equal);
        assert_eq!(comparar_versoes("1.5.0", "1.4.2"), Ordering::Greater);
        assert_eq!(comparar_versoes("1.4.2", "1.4.1"), Ordering::Greater);
        assert_eq!(comparar_versoes("1.4.1", "1.4.0"), Ordering::Greater);
        assert_eq!(comparar_versoes("v2.0.0", "1.9.9"), Ordering::Greater);
        assert_eq!(comparar_versoes("v1.3.5", "1.4.0"), Ordering::Less);
        assert_eq!(comparar_versoes("1.4.0", "v1.4.1"), Ordering::Less);
        assert_eq!(comparar_versoes("1.4.0-beta", "1.4.0"), Ordering::Equal);
    }

    #[test]
    fn test_mensagens_alerta_ram() {
        let msg_pt = "O uso de memória RAM está em 85%. Deseja otimizar o sistema agora?";
        let rotulo_pt = "Otimizar RAM";
        let msg_en = "RAM usage reached 85%. Would you like to optimize memory now?";
        let rotulo_en = "Optimize RAM";

        assert_eq!(msg_pt, "O uso de memória RAM está em 85%. Deseja otimizar o sistema agora?");
        assert_eq!(rotulo_pt, "Otimizar RAM");
        assert_eq!(msg_en, "RAM usage reached 85%. Would you like to optimize memory now?");
        assert_eq!(rotulo_en, "Optimize RAM");
    }

    #[test]
    fn test_sysinfo_memoria_disponivel() {
        let mut sys = sysinfo::System::new();
        sys.refresh_memory();
        assert!(sys.total_memory() > 0);
        assert!(sys.used_memory() > 0);
    }

    #[test]
    fn test_obter_curiosidade_proativa_sem_chave_ou_offline_retorna_none() {
        // Sem chave de API ou com chave vazia deve retornar None
        assert_eq!(obter_curiosidade_proativa("", 70, "espelho", "pt", ""), None);
        assert_eq!(obter_curiosidade_proativa("   ", 70, "espelho", "en", ""), None);

        // Chave fictícia / erro de autenticação ou rede deve retornar None
        assert_eq!(obter_curiosidade_proativa("chave_invalida_mock", 70, "hostil", "pt", ""), None);
    }
}
