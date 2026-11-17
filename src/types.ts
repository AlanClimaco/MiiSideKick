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
    gadget_ativo?: boolean;
    gadget_x?: number;
    gadget_y?: number;
    avatar_modo?: string; // "classico", "nostalgico", "custom"
}

export interface RegraPasta {
    id: string;
    nome: string;
    destino_chave: string;
    subpasta?: string | null;
    extensoes: string[];
}

export interface RegrasOrganizador {
    versao: number;
    regras: RegraPasta[];
}

export interface TelemetriaRapida {
    cpu_uso: number;
    ram_uso_pct: number;
    ram_usada_mb: number;
    ram_total_mb: number;
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
