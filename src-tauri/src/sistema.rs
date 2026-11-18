use std::collections::VecDeque;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use crate::comandos::NotificacaoPayload;

#[cfg(target_os = "windows")]
#[repr(C)]
#[allow(non_camel_case_types, dead_code)]
#[derive(Debug, PartialEq, Eq)]
pub enum QUERY_USER_NOTIFICATION_STATE {
    QUNS_NOT_PRESENT = 1,
    QUNS_BUSY = 2,
    QUNS_RUNNING_D3D_FULL_SCREEN = 3,
    QUNS_PRESENTATION_MODE = 4,
    QUNS_ACCEPTS_NOTIFICATIONS = 5,
    QUNS_QUIET_TIME = 6,
    QUNS_APP = 7,
}

#[cfg(target_os = "windows")]
#[repr(C)]
struct RECT {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

#[cfg(target_os = "windows")]
#[repr(C)]
struct MONITORINFO {
    cb_size: u32,
    rc_monitor: RECT,
    rc_work: RECT,
    dw_flags: u32,
}

#[cfg(target_os = "windows")]
#[link(name = "shell32")]
extern "system" {
    fn SHQueryUserNotificationState(pquns: *mut QUERY_USER_NOTIFICATION_STATE) -> i32;
}

#[cfg(target_os = "windows")]
#[link(name = "user32")]
extern "system" {
    fn GetForegroundWindow() -> *mut std::ffi::c_void;
    fn GetWindowRect(hwnd: *mut std::ffi::c_void, lp_rect: *mut RECT) -> i32;
    fn MonitorFromWindow(hwnd: *mut std::ffi::c_void, dw_flags: u32) -> *mut std::ffi::c_void;
    fn GetMonitorInfoW(h_monitor: *mut std::ffi::c_void, lpmi: *mut MONITORINFO) -> i32;
    fn GetDesktopWindow() -> *mut std::ffi::c_void;
    fn GetShellWindow() -> *mut std::ffi::c_void;
    fn IsIconic(hwnd: *mut std::ffi::c_void) -> i32;
    fn GetWindowTextW(hwnd: *mut std::ffi::c_void, lp_string: *mut u16, n_max_count: i32) -> i32;
    fn GetWindowThreadProcessId(hwnd: *mut std::ffi::c_void, lpdw_process_id: *mut u32) -> u32;
}

#[cfg(target_os = "windows")]
#[link(name = "kernel32")]
extern "system" {
    fn OpenProcess(
        dw_desired_access: u32,
        b_inherit_handle: i32,
        dw_process_id: u32,
    ) -> *mut std::ffi::c_void;
    fn QueryFullProcessImageNameW(
        h_process: *mut std::ffi::c_void,
        dw_flags: u32,
        lp_exe_name: *mut u16,
        lpdw_size: *mut u32,
    ) -> i32;
    fn CloseHandle(h_object: *mut std::ffi::c_void) -> i32;
}

#[cfg(target_os = "windows")]
const PROCESS_QUERY_LIMITED_INFORMATION: u32 = 0x1000;
#[cfg(target_os = "windows")]
const MONITOR_DEFAULTTOPRIMARY: u32 = 1;

/// Níveis de prioridade para a política de entrega inteligente
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[allow(dead_code)]
pub enum PrioridadeNotificacao {
    Baixa = 1,     // Curiosidades, comentários amenos
    Media = 2,     // Lembretes de rotina, bem-estar, novidades
    Alta = 3,      // Lembrete de timer do usuário, manutenção lixeira/downloads
    Critica = 4,   // Alerta de RAM severo, falhas críticas
}

/// Categorias temáticas das notificações para descarte inteligente
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub enum CategoriaNotificacao {
    Curiosidade,
    Ram,
    Lixeira,
    Downloads,
    Timer,
    BemEstar,
    Atualizacao,
    ContextualJogo,
    Geral,
}

/// Item estruturado na fila com expiração e criticidade
#[derive(Clone, Debug)]
pub struct ItemFilaNotificacao {
    #[allow(dead_code)]
    pub id: u64,
    pub payload: NotificacaoPayload,
    pub prioridade: PrioridadeNotificacao,
    pub categoria: CategoriaNotificacao,
    pub timestamp: Instant,
    pub validade: Duration,
}

/// Informações sobre uma sessão de processo/jogo em tela cheia
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct InfoSessaoFoco {
    pub nome_app: String,
    pub titulo_janela: String,
    pub timestamp_inicio: Instant,
    pub duracao: Duration,
}

/// Fila thread-safe de notificações retidas durante tela cheia
static FILA_NOTIFICACOES_INTELIGENTE: Mutex<VecDeque<ItemFilaNotificacao>> =
    Mutex::new(VecDeque::new());

/// Contador incremental para IDs únicos na fila
static CONTADOR_NOTIFICACOES: Mutex<u64> = Mutex::new(1);

/// Registra o momento em que o sistema saiu do modo de tela cheia (para tolerância de segurança)
static ULTIMA_SAIDA_TELA_CHEIA: Mutex<Option<Instant>> = Mutex::new(None);

/// Registra o estado da última checagem de tela cheia para detectar transição (saída)
static ESTAVA_EM_TELA_CHEIA: Mutex<bool> = Mutex::new(false);

/// Sessão de tela cheia atualmente em andamento
static SESSAO_TELA_CHEIA_ATIVA: Mutex<Option<InfoSessaoFoco>> = Mutex::new(None);

/// Última sessão finalizada que pode disparar comentário contextual amigável
static ULTIMA_SESSAO_FINALIZADA: Mutex<Option<InfoSessaoFoco>> = Mutex::new(None);

/// Registra o momento em que o último comentário contextual pós-jogo foi disparado (para cooldown por frequência)
static ULTIMO_COMENTARIO_CONTEXTUAL: Mutex<Option<Instant>> = Mutex::new(None);

/// Obtém informações da janela ativa em primeiro plano (nome do executável e título)
pub fn obter_info_janela_foco() -> Option<(String, String)> {
    #[cfg(target_os = "windows")]
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.is_null() {
            return None;
        }
        let desktop = GetDesktopWindow();
        let shell = GetShellWindow();
        if hwnd == desktop || hwnd == shell {
            return None;
        }

        // 1. Título da janela
        let mut title_buf = [0u16; 512];
        let len = GetWindowTextW(hwnd, title_buf.as_mut_ptr(), 512);
        let titulo = if len > 0 {
            String::from_utf16_lossy(&title_buf[..len as usize]).trim().to_string()
        } else {
            String::new()
        };

        // 2. Nome do executável do processo
        let mut pid: u32 = 0;
        GetWindowThreadProcessId(hwnd, &mut pid);
        let mut nome_executavel = String::new();

        if pid > 0 {
            let h_proc = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
            if !h_proc.is_null() {
                let mut exe_buf = [0u16; 1024];
                let mut size = 1024u32;
                if QueryFullProcessImageNameW(h_proc, 0, exe_buf.as_mut_ptr(), &mut size) != 0
                    && size > 0
                {
                    let full_path = String::from_utf16_lossy(&exe_buf[..size as usize]);
                    if let Some(filename) =
                        std::path::Path::new(&full_path).file_name().and_then(|f| f.to_str())
                    {
                        nome_executavel = filename
                            .trim_end_matches(".exe")
                            .trim_end_matches(".EXE")
                            .to_string();
                    }
                }
                CloseHandle(h_proc);
            }
        }

        if !titulo.is_empty() || !nome_executavel.is_empty() {
            let nome_final = if !nome_executavel.is_empty() {
                nome_executavel
            } else {
                titulo.clone()
            };
            return Some((nome_final, titulo));
        }
    }

    #[cfg(target_os = "linux")]
    {
        // Mock seguro para Linux/X11
        if std::env::var("DISPLAY").is_ok() {
            if let Ok(out) = std::process::Command::new("xprop")
                .args(["-root", "_NET_ACTIVE_WINDOW"])
                .output()
            {
                let texto = String::from_utf8_lossy(&out.stdout);
                if let Some(id_hex) = texto.split('#').nth(1).map(|s| s.trim()) {
                    if !id_hex.is_empty() && id_hex != "0x0" {
                        if let Ok(p_out) = std::process::Command::new("xprop")
                            .args(["-id", id_hex, "WM_NAME"])
                            .output()
                        {
                            let t = String::from_utf8_lossy(&p_out.stdout);
                            if let Some(pos) = t.find('=') {
                                let val = t[pos + 1..].trim().trim_matches('"').to_string();
                                if !val.is_empty() {
                                    return Some((val.clone(), val));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        // Fallback para outros sistemas
    }

    None
}

/// Detecta nativamente se o sistema operacional está em modo de tela cheia / jogo / apresentação.
pub fn esta_em_tela_cheia() -> bool {
    #[cfg(target_os = "windows")]
    {
        unsafe {
            // 1. Checagem via API oficial do Shell (DirectX/D3D e Apresentações)
            let mut state = QUERY_USER_NOTIFICATION_STATE::QUNS_NOT_PRESENT;
            if SHQueryUserNotificationState(&mut state) == 0 {
                match state {
                    QUERY_USER_NOTIFICATION_STATE::QUNS_RUNNING_D3D_FULL_SCREEN
                    | QUERY_USER_NOTIFICATION_STATE::QUNS_BUSY
                    | QUERY_USER_NOTIFICATION_STATE::QUNS_PRESENTATION_MODE => return true,
                    _ => {}
                }
            }

            // 2. Checagem geométrica da janela em foco em relação ao monitor ativo
            let hwnd = GetForegroundWindow();
            if hwnd.is_null() {
                return false;
            }

            let desktop = GetDesktopWindow();
            let shell = GetShellWindow();
            if hwnd == desktop || hwnd == shell {
                return false;
            }

            if IsIconic(hwnd) != 0 {
                return false;
            }

            let mut rect = RECT {
                left: 0,
                top: 0,
                right: 0,
                bottom: 0,
            };
            if GetWindowRect(hwnd, &mut rect) == 0 {
                return false;
            }

            let h_monitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTOPRIMARY);
            if h_monitor.is_null() {
                return false;
            }

            let mut mi = MONITORINFO {
                cb_size: std::mem::size_of::<MONITORINFO>() as u32,
                rc_monitor: RECT {
                    left: 0,
                    top: 0,
                    right: 0,
                    bottom: 0,
                },
                rc_work: RECT {
                    left: 0,
                    top: 0,
                    right: 0,
                    bottom: 0,
                },
                dw_flags: 0,
            };

            if GetMonitorInfoW(h_monitor, &mut mi) != 0 {
                let cobriu_tela = rect.left <= mi.rc_monitor.left
                    && rect.top <= mi.rc_monitor.top
                    && rect.right >= mi.rc_monitor.right
                    && rect.bottom >= mi.rc_monitor.bottom;
                if cobriu_tela {
                    return true;
                }
            }
        }
        false
    }

    #[cfg(target_os = "linux")]
    {
        // Checagem segura de _NET_WM_STATE_FULLSCREEN via xprop no Linux X11 (com timeout e fallback)
        if std::env::var("DISPLAY").is_ok() {
            let saida = std::process::Command::new("xprop")
                .args(["-root", "_NET_ACTIVE_WINDOW"])
                .output();

            if let Ok(out) = saida {
                let texto = String::from_utf8_lossy(&out.stdout);
                if let Some(id_hex) = texto.split('#').nth(1).map(|s| s.trim()) {
                    if !id_hex.is_empty() && id_hex != "0x0" {
                        let prop_check = std::process::Command::new("xprop")
                            .args(["-id", id_hex, "_NET_WM_STATE"])
                            .output();
                        if let Ok(p_out) = prop_check {
                            let p_texto = String::from_utf8_lossy(&p_out.stdout);
                            if p_texto.contains("_NET_WM_STATE_FULLSCREEN") {
                                return true;
                            }
                        }
                    }
                }
            }
        }
        false
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        false
    }
}

/// Verifica se o sistema saiu recentemente do modo de tela cheia (tolerância de segurança de 3 a 5 segundos)
pub fn esta_em_periodo_tolerancia(segundos: u64) -> bool {
    if let Ok(saida) = ULTIMA_SAIDA_TELA_CHEIA.lock() {
        if let Some(inst) = *saida {
            return inst.elapsed() < Duration::from_secs(segundos);
        }
    }
    false
}

/// Atualiza o ciclo de monitoramento de tela cheia e gerencia sessões ativas
pub fn atualizar_e_checar_foco() -> bool {
    let agora_tela_cheia = esta_em_tela_cheia();
    let mut estava = ESTAVA_EM_TELA_CHEIA.lock().unwrap();

    if !*estava && agora_tela_cheia {
        // Entrou em modo tela cheia! Inicia rastreamento de sessão
        let (nome_app, titulo_janela) = obter_info_janela_foco().unwrap_or_else(|| {
            ("Jogo/Aplicativo".to_string(), "Tela Cheia".to_string())
        });
        if let Ok(mut sessao) = SESSAO_TELA_CHEIA_ATIVA.lock() {
            *sessao = Some(InfoSessaoFoco {
                nome_app,
                titulo_janela,
                timestamp_inicio: Instant::now(),
                duracao: Duration::from_secs(0),
            });
        }
    } else if *estava && !agora_tela_cheia {
        // Acabou de sair de tela cheia! Finaliza sessão e inicia período de tolerância
        if let Ok(mut saida) = ULTIMA_SAIDA_TELA_CHEIA.lock() {
            *saida = Some(Instant::now());
        }

        if let Ok(mut ativa) = SESSAO_TELA_CHEIA_ATIVA.lock() {
            if let Some(mut sess) = ativa.take() {
                sess.duracao = sess.timestamp_inicio.elapsed();
                if let Ok(mut finalizada) = ULTIMA_SESSAO_FINALIZADA.lock() {
                    *finalizada = Some(sess);
                }
            }
        }
    }

    *estava = agora_tela_cheia;

    agora_tela_cheia || esta_em_periodo_tolerancia(5)
}

/// Enfileira uma notificação com inferência inteligente de categoria e prioridade
pub fn enfileirar_notificacao(payload: NotificacaoPayload) {
    let (categoria, prioridade, validade_secs) = inferir_categoria_e_prioridade(&payload);
    enfileirar_notificacao_inteligente(payload, categoria, prioridade, validade_secs);
}

/// Enfileira uma notificação de forma explícita com categoria e prioridade
pub fn enfileirar_notificacao_inteligente(
    payload: NotificacaoPayload,
    categoria: CategoriaNotificacao,
    prioridade: PrioridadeNotificacao,
    validade_secs: u64,
) {
    if let Ok(mut fila) = FILA_NOTIFICACOES_INTELIGENTE.lock() {
        let mut id_counter = CONTADOR_NOTIFICACOES.lock().unwrap();
        let id = *id_counter;
        *id_counter = id_counter.saturating_add(1);

        // Limite máximo de segurança para a fila em memória (evita vazamento de memória)
        if fila.len() >= 20 {
            fila.pop_front();
        }

        fila.push_back(ItemFilaNotificacao {
            id,
            payload,
            prioridade,
            categoria,
            timestamp: Instant::now(),
            validade: Duration::from_secs(validade_secs),
        });
    }
}

/// Infere a categoria e a prioridade de uma notificação com base no payload
fn inferir_categoria_e_prioridade(
    payload: &NotificacaoPayload,
) -> (CategoriaNotificacao, PrioridadeNotificacao, u64) {
    let texto_lower = payload.texto.to_lowercase();

    if payload.acao.as_deref() == Some("otimizar_ram") || texto_lower.contains("ram") {
        (CategoriaNotificacao::Ram, PrioridadeNotificacao::Critica, 60)
    } else if payload.pode_responder
        || texto_lower.contains("lembrete")
        || texto_lower.contains("timer")
    {
        (CategoriaNotificacao::Timer, PrioridadeNotificacao::Alta, 3600)
    } else if texto_lower.contains("downloads") {
        (CategoriaNotificacao::Downloads, PrioridadeNotificacao::Alta, 180)
    } else if texto_lower.contains("lixeira") || texto_lower.contains("lixo") {
        (CategoriaNotificacao::Lixeira, PrioridadeNotificacao::Alta, 180)
    } else if texto_lower.contains("versão disponível")
        || texto_lower.contains("version available")
    {
        (CategoriaNotificacao::Atualizacao, PrioridadeNotificacao::Media, 7200)
    } else if payload.tipo == "explicando" || payload.tipo == "bocejo" {
        (CategoriaNotificacao::Curiosidade, PrioridadeNotificacao::Baixa, 300)
    } else {
        (CategoriaNotificacao::Geral, PrioridadeNotificacao::Media, 300)
    }
}

/// Política de Descarte Inteligente (*Purge/Debounce*) ao Sair de Tela Cheia:
/// 1. Remove alertas cuja validade temporal expirou.
/// 2. Remove alertas de RAM alta caso o uso atual de memória já tenha normalizado (< 80%).
/// 3. Se houver múltiplas curiosidades, retém apenas a mais recente.
/// 4. Retorna no máximo 1 notificação prioritária selecionada, descartando as notificações obsoletas.
pub fn purgar_e_obter_proxima_notificacao(ram_atual_pct: f64) -> Option<NotificacaoPayload> {
    let mut fila = FILA_NOTIFICACOES_INTELIGENTE.lock().ok()?;

    if fila.is_empty() {
        return None;
    }

    // 1. Filtra itens expirados
    fila.retain(|item| item.timestamp.elapsed() <= item.validade);

    // 2. Descarta alertas de RAM acumulados se a memória já tiver normalizado (< 85.0%)
    if ram_atual_pct < 85.0 {
        let antes = fila.len();
        fila.retain(|item| item.categoria != CategoriaNotificacao::Ram);
        let descartados = antes - fila.len();
        if descartados > 0 {
            eprintln!(
                "[Sistema]: Descartado(s) {} alerta(s) de RAM gerados durante jogo (RAM normalizada em {:.1}%)",
                descartados, ram_atual_pct
            );
        }
    }

    // 3. Se houver múltiplas curiosidades acumuladas, mantém apenas a última inserida
    let mut encontrou_curiosidade = false;
    let mut itens_para_manter = Vec::new();

    // Itera do mais recente para o mais antigo
    for item in fila.iter().rev() {
        if item.categoria == CategoriaNotificacao::Curiosidade {
            if !encontrou_curiosidade {
                encontrou_curiosidade = true;
                itens_para_manter.push(item.clone());
            }
        } else {
            itens_para_manter.push(item.clone());
        }
    }

    fila.clear();
    for item in itens_para_manter.into_iter().rev() {
        fila.push_back(item);
    }

    if fila.is_empty() {
        return None;
    }

    // 4. Seleciona o item de maior prioridade (e mais recente em caso de empate)
    let mut melhor_idx = 0;
    let mut melhor_prioridade = PrioridadeNotificacao::Baixa;
    let mut melhor_tempo = Instant::now();

    for (idx, item) in fila.iter().enumerate() {
        if item.prioridade > melhor_prioridade
            || (item.prioridade == melhor_prioridade && item.timestamp >= melhor_tempo)
        {
            melhor_prioridade = item.prioridade;
            melhor_tempo = item.timestamp;
            melhor_idx = idx;
        }
    }

    let escolhido = fila.remove(melhor_idx);

    // Remove as notificações secundárias de baixa ou média prioridade restantes para não bombardear o usuário
    fila.retain(|item| item.prioridade >= PrioridadeNotificacao::Alta);

    escolhido.map(|item| item.payload)
}

/// Consome um comentário contextual amigável após o encerramento de um jogo/processo em tela cheia.
/// Respeita:
/// 1. Duração mínima de 20 minutos (1200 segundos) para considerar apto a comentários (sessões rápidas são ignoradas).
/// 2. Cooldown e probabilidade por frequência (AppConfig.frequencia_alertas):
///    - Discreto: Cooldown de 90 a 120 minutos (90 min); probabilidade de 15%.
///    - Equilibrado: Cooldown de 45 a 60 minutos (45 min); probabilidade de 35%.
///    - Frequente: Cooldown de 20 a 30 minutos (20 min); probabilidade de 50%.
pub fn consumir_comentario_contextual_pos_jogo(
    is_en: bool,
    frequencia: &str,
) -> Option<NotificacaoPayload> {
    let mut sessao_opt = ULTIMA_SESSAO_FINALIZADA.lock().ok()?;
    let sessao = sessao_opt.take()?;

    // 1. Duração Mínima de Sessão: pelo menos 20 minutos (1200 segundos)
    // Fechar um vídeo rápido de 5 minutos de YouTube ou jogo rápido deve ser completamente ignorado.
    if sessao.duracao.as_secs() < 20 * 60 {
        return None;
    }

    // 2. Cooldown e Probabilidade por Frequência
    let (cooldown_minutos, probabilidade_pct) = match frequencia.to_lowercase().trim() {
        "discreto" => (90u64, 15u32),
        "frequente" => (20u64, 50u32),
        _ => (45u64, 35u32), // equilibrado (padrão)
    };

    if let Ok(ultimo_guard) = ULTIMO_COMENTARIO_CONTEXTUAL.lock() {
        if let Some(ultimo) = *ultimo_guard {
            if ultimo.elapsed() < Duration::from_secs(cooldown_minutos * 60) {
                return None;
            }
        }
    }

    // Sorteio de probabilidade (0..99)
    let agora_nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();
    let sorteio = (agora_nanos % 100) as u32;

    if sorteio >= probabilidade_pct {
        return None;
    }

    // Atualiza o registro do último comentário contextual aceito e disparado
    if let Ok(mut ultimo_guard) = ULTIMO_COMENTARIO_CONTEXTUAL.lock() {
        *ultimo_guard = Some(Instant::now());
    }

    let nome_amigavel = formatar_nome_processo(&sessao.nome_app);

    let texto = if is_en {
        format!(
            "Hope you had a great time in {}! Let me know if you need anything.",
            nome_amigavel
        )
    } else {
        format!(
            "Espero que a sessão no {} tenha sido boa! Se precisar de algo, é só chamar.",
            nome_amigavel
        )
    };

    Some(NotificacaoPayload {
        texto,
        tipo: "feliz".to_string(),
        link: None,
        link_rotulo: None,
        pode_responder: false,
        tocar_som: true,
        acao: None,
        acao_rotulo: None,
        eh_reconsulta: false,
        timestamp: 0,
        hora_formatada: String::new(),
        comando_executar: None,
        comando_args: None,
    })
}

/// Formata nomes de processos populares para nomes amigáveis de exibição
fn formatar_nome_processo(raw: &str) -> String {
    let trim = raw.trim();
    let lower = trim.to_lowercase();

    if lower.contains("code") {
        "Visual Studio Code".to_string()
    } else if lower.contains("devenv") {
        "Visual Studio".to_string()
    } else if lower.contains("steam") {
        "Steam".to_string()
    } else if lower.contains("minecraft") || lower.contains("javaw") {
        "Minecraft".to_string()
    } else if lower.contains("valorant") {
        "Valorant".to_string()
    } else if lower.contains("csgo") || lower.contains("cs2") {
        "Counter-Strike".to_string()
    } else if lower.contains("dota") {
        "Dota 2".to_string()
    } else if lower.contains("leagueoflegends") || lower.contains("league of legends") {
        "League of Legends".to_string()
    } else if lower.contains("chrome") {
        "Google Chrome".to_string()
    } else if lower.contains("firefox") {
        "Firefox".to_string()
    } else if trim.is_empty() {
        "Jogo/Aplicativo".to_string()
    } else {
        trim.to_string()
    }
}

/// Retira a próxima notificação da fila (compatibilidade com chamadas simples)
#[allow(dead_code)]
pub fn desenfileirar_notificacao() -> Option<NotificacaoPayload> {
    if let Ok(mut fila) = FILA_NOTIFICACOES_INTELIGENTE.lock() {
        fila.pop_front().map(|i| i.payload)
    } else {
        None
    }
}

/// Retorna a quantidade de itens na fila.
pub fn tamanho_fila() -> usize {
    FILA_NOTIFICACOES_INTELIGENTE
        .lock()
        .map(|f| f.len())
        .unwrap_or(0)
}

/// Esvazia completamente a fila de notificações.
#[allow(dead_code)]
pub fn limpar_fila() {
    if let Ok(mut fila) = FILA_NOTIFICACOES_INTELIGENTE.lock() {
        fila.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    static TEST_MUTEX: Mutex<()> = Mutex::new(());

    fn criar_payload_mock(texto: &str, acao: Option<&str>) -> NotificacaoPayload {
        NotificacaoPayload {
            texto: texto.to_string(),
            tipo: "normal".to_string(),
            link: None,
            link_rotulo: None,
            pode_responder: false,
            tocar_som: false,
            acao: acao.map(|a| a.to_string()),
            acao_rotulo: None,
            eh_reconsulta: false,
            timestamp: 0,
            hora_formatada: String::new(),
            comando_executar: None,
            comando_args: None,
        }
    }

    #[test]
    fn test_fila_notificacoes_teto_e_purga() {
        let _guard = TEST_MUTEX.lock().unwrap();
        limpar_fila();
        assert_eq!(tamanho_fila(), 0);

        for i in 0..25 {
            enfileirar_notificacao(criar_payload_mock(&format!("Msg {}", i), None));
        }

        // Deve respeitar o teto de 20 itens
        assert_eq!(tamanho_fila(), 20);
        limpar_fila();
    }

    #[test]
    fn test_descarte_alerta_ram_quando_normalizado() {
        let _guard = TEST_MUTEX.lock().unwrap();
        limpar_fila();

        // Enfileira alerta de RAM alta
        let payload_ram = criar_payload_mock("Atenção: Uso de RAM em 88%", Some("otimizar_ram"));
        enfileirar_notificacao(payload_ram);

        // Enfileira lembrete de timer (prioridade alta)
        let payload_timer = criar_payload_mock("Lembrete: Reunião em 5 minutos", None);
        enfileirar_notificacao_inteligente(
            payload_timer,
            CategoriaNotificacao::Timer,
            PrioridadeNotificacao::Alta,
            3600,
        );

        // Se a RAM normalizou (ex: 60%), o alerta de RAM deve ser purgado e o Timer deve ser retornado
        let entregue = purgar_e_obter_proxima_notificacao(60.0).expect("Deveria retornar o timer");
        assert!(
            entregue.texto.contains("Lembrete"),
            "Esperava o lembrete de timer, mas recebeu: {}",
            entregue.texto
        );

        // Fila deve estar limpa agora
        assert_eq!(tamanho_fila(), 0);
    }

    #[test]
    fn test_descarte_curiosidades_acumuladas() {
        let _guard = TEST_MUTEX.lock().unwrap();
        limpar_fila();

        // Enfileira 3 curiosidades consecutivas acumuladas durante o jogo
        for i in 1..=3 {
            enfileirar_notificacao_inteligente(
                criar_payload_mock(&format!("Curiosidade {}", i), None),
                CategoriaNotificacao::Curiosidade,
                PrioridadeNotificacao::Baixa,
                300,
            );
        }

        assert_eq!(tamanho_fila(), 3);

        // Ao purgar, deve entregar apenas a curiosidade mais recente (Curiosidade 3) e descartar as antigas
        let entregue =
            purgar_e_obter_proxima_notificacao(70.0).expect("Deveria entregar a curiosidade recente");
        assert_eq!(entregue.texto, "Curiosidade 3");

        // Fila deve ter descartado as demais curiosidades antigas
        assert_eq!(tamanho_fila(), 0);
    }

    #[test]
    fn test_formatar_nome_processo() {
        assert_eq!(formatar_nome_processo("code.exe"), "Visual Studio Code");
        assert_eq!(formatar_nome_processo("javaw"), "Minecraft");
        assert_eq!(formatar_nome_processo("steam"), "Steam");
        assert_eq!(formatar_nome_processo("meu_jogo"), "meu_jogo");
    }

    #[test]
    fn test_comentario_contextual_ignora_sessao_curta_5min() {
        let _guard = TEST_MUTEX.lock().unwrap();
        // Simula sessão de apenas 5 minutos (300 segundos, como vídeo de YouTube ou jogo rápido)
        if let Ok(mut lock) = ULTIMA_SESSAO_FINALIZADA.lock() {
            *lock = Some(InfoSessaoFoco {
                nome_app: "chrome.exe".to_string(),
                titulo_janela: "YouTube".to_string(),
                timestamp_inicio: Instant::now() - Duration::from_secs(300),
                duracao: Duration::from_secs(300),
            });
        }

        // Não deve gerar comentário contextual, pois durou menos de 20 minutos
        let resultado = consumir_comentario_contextual_pos_jogo(false, "discreto");
        assert!(resultado.is_none(), "Sessão de 5 minutos deveria ser completamente ignorada");
    }

    #[test]
    fn test_comentario_contextual_cooldown_discreto() {
        let _guard = TEST_MUTEX.lock().unwrap();
        // Define último comentário recente (há 10 minutos)
        if let Ok(mut lock) = ULTIMO_COMENTARIO_CONTEXTUAL.lock() {
            *lock = Some(Instant::now() - Duration::from_secs(10 * 60));
        }

        // Simula sessão de 30 minutos (1800 segundos)
        if let Ok(mut lock) = ULTIMA_SESSAO_FINALIZADA.lock() {
            *lock = Some(InfoSessaoFoco {
                nome_app: "steam.exe".to_string(),
                titulo_janela: "Game".to_string(),
                timestamp_inicio: Instant::now() - Duration::from_secs(1800),
                duracao: Duration::from_secs(1800),
            });
        }

        // No modo discreto (cooldown 90 min), não deve disparar
        let resultado = consumir_comentario_contextual_pos_jogo(false, "discreto");
        assert!(resultado.is_none(), "Deveria estar em cooldown no modo discreto");
    }
}
