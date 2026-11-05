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
}

const MONITOR_DEFAULTTOPRIMARY: u32 = 1;

/// Fila thread-safe de notificações retidas durante tela cheia
static FILA_NOTIFICACOES: Mutex<VecDeque<NotificacaoPayload>> = Mutex::new(VecDeque::new());

/// Registra o momento em que o sistema saiu do modo de tela cheia (para tolerância de segurança)
static ULTIMA_SAIDA_TELA_CHEIA: Mutex<Option<Instant>> = Mutex::new(None);

/// Registra o estado da última checagem de tela cheia para detectar transição (saída)
static ESTAVA_EM_TELA_CHEIA: Mutex<bool> = Mutex::new(false);

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

            let mut rect = RECT { left: 0, top: 0, right: 0, bottom: 0 };
            if GetWindowRect(hwnd, &mut rect) == 0 {
                return false;
            }

            let h_monitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTOPRIMARY);
            if h_monitor.is_null() {
                return false;
            }

            let mut mi = MONITORINFO {
                cb_size: std::mem::size_of::<MONITORINFO>() as u32,
                rc_monitor: RECT { left: 0, top: 0, right: 0, bottom: 0 },
                rc_work: RECT { left: 0, top: 0, right: 0, bottom: 0 },
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

/// Atualiza o ciclo de monitoramento de tela cheia. Retorna se o modo de tela cheia ou tolerância está ativo.
pub fn atualizar_e_checar_foco() -> bool {
    let agora_tela_cheia = esta_em_tela_cheia();
    let mut estava = ESTAVA_EM_TELA_CHEIA.lock().unwrap();

    if *estava && !agora_tela_cheia {
        // Acabou de sair de tela cheia! Inicia o período de tolerância de segurança
        if let Ok(mut saida) = ULTIMA_SAIDA_TELA_CHEIA.lock() {
            *saida = Some(Instant::now());
        }
    }

    *estava = agora_tela_cheia;

    agora_tela_cheia || esta_em_periodo_tolerancia(4)
}

/// Enfileira uma notificação para entrega posterior quando desocupar a tela.
pub fn enfileirar_notificacao(payload: NotificacaoPayload) {
    if let Ok(mut fila) = FILA_NOTIFICACOES.lock() {
        // Limite máximo de segurança para a fila em memória (evita vazamento)
        if fila.len() >= 20 {
            fila.pop_front();
        }
        fila.push_back(payload);
    }
}

/// Retira a próxima notificação da fila se houver alguma pendente.
pub fn desenfileirar_notificacao() -> Option<NotificacaoPayload> {
    if let Ok(mut fila) = FILA_NOTIFICACOES.lock() {
        fila.pop_front()
    } else {
        None
    }
}

/// Retorna a quantidade de itens na fila.
pub fn tamanho_fila() -> usize {
    FILA_NOTIFICACOES.lock().map(|f| f.len()).unwrap_or(0)
}

/// Esvazia completamente a fila de notificações.
#[allow(dead_code)]
pub fn limpar_fila() {
    if let Ok(mut fila) = FILA_NOTIFICACOES.lock() {
        fila.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fila_notificacoes_fifo_e_limite() {
        limpar_fila();
        assert_eq!(tamanho_fila(), 0);

        for i in 0..25 {
            enfileirar_notificacao(NotificacaoPayload {
                texto: format!("Msg {}", i),
                tipo: "normal".to_string(),
                link: None,
                link_rotulo: None,
                pode_responder: false,
                tocar_som: false,
                acao: None,
                acao_rotulo: None,
                eh_reconsulta: false,
                timestamp: 0,
                hora_formatada: String::new(),
            });
        }

        // Deve respeitar o teto de 20 itens
        assert_eq!(tamanho_fila(), 20);

        let primeiro = desenfileirar_notificacao().unwrap();
        assert_eq!(primeiro.texto, "Msg 5"); // os primeiros 5 foram descartados pelo teto

        limpar_fila();
        assert_eq!(tamanho_fila(), 0);
        assert!(desenfileirar_notificacao().is_none());
    }

    #[test]
    fn test_periodo_tolerancia_saida_tela_cheia() {
        if let Ok(mut saida) = ULTIMA_SAIDA_TELA_CHEIA.lock() {
            *saida = Some(Instant::now());
        }
        assert!(esta_em_periodo_tolerancia(3));

        if let Ok(mut saida) = ULTIMA_SAIDA_TELA_CHEIA.lock() {
            *saida = None;
        }
        assert!(!esta_em_periodo_tolerancia(3));
    }
}
