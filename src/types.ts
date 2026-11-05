export interface AppConfig {
    user_name: string;
    gemini_api_key: string;
    personalidade: string;
    limite_caracteres: number;
    alerta_manutencao: boolean;
    alerta_interacao_bem_estar: boolean;
    frequencia_alertas: string;
    idioma: string;
    gostos_usuario?: string;
}

export interface UserInfo {
    name: string;
    downloads: string;
}

export interface PayloadNotificacao {
    texto: string;
    tipo: string;
    link?: string | null;
    link_rotulo?: string | null;
    pode_responder?: boolean;
    tocar_som?: boolean;
    acao?: string | null;
    acao_rotulo?: string | null;
    eh_reconsulta?: boolean;
    timestamp?: number;
    hora_formatada?: string;
}

export interface ClipboardItem {
    id: string;
    texto: string;
    tamanho: number;
    timestamp: number;
    hora_formatada: string;
}

declare global {
    interface Window {
        atualizarInterface?: (
            textoOuPayload: string | PayloadNotificacao,
            tipo?: string,
            link?: string | null,
            linkRotulo?: string | null,
            podeResponder?: boolean,
            tocarSom?: boolean,
            acao?: string | null,
            acaoRotulo?: string | null,
            ehReconsulta?: boolean
        ) => void;
    }
}
