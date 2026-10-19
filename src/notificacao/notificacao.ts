import { invoke } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";

// Importação das 7 expressões visuais oficiais para garantir empacotamento pelo bundler
import miiNormal from '../assets/reacoes/mii-normal.png';
import miiFeliz from '../assets/reacoes/mii-feliz.png';
import miiFrustrado from '../assets/reacoes/mii-frustrado.png';
import miiAssustado from '../assets/reacoes/mii-assustado.png';
import miiExplicando from '../assets/reacoes/mii-explicando.png';
import miiBocejo from '../assets/reacoes/mii-bocejo.png';
import miiLambendo from '../assets/reacoes/mii-lambendo.png';
import bellSound from '../assets/audio/bell.wav';

// Mapa das 7 reações suportadas com suporte a aliases
const mapaAssetsReacoes: Record<string, string> = {
    normal: miiNormal,
    feliz: miiFeliz,
    frustrado: miiFrustrado,
    assustado: miiAssustado,
    explicando: miiExplicando,
    bocejo: miiBocejo,
    lambendo: miiLambendo,
    erro: miiFrustrado,
    tranquilo: miiFeliz,
    sucesso: miiFeliz,
};

/**
 * Resolve dinamicamente a imagem da reação apontando para `/src/assets/reacoes/mii-{reacao}.png`
 * e utilizando o asset empacotado pelo Vite.
 */
function obterImagemReacao(reacao: string): string {
    const nomeLimpo = (reacao || "normal").toLowerCase().trim();
    if (mapaAssetsReacoes[nomeLimpo]) {
        return mapaAssetsReacoes[nomeLimpo];
    }
    // Aponta dinamicamente para o caminho da pasta de assets
    return `/src/assets/reacoes/mii-${nomeLimpo}.png`;
}

let hideTimeout: number;
let fadeTimeout: number;

function pausarTimerFechamento() {
    clearTimeout(hideTimeout);
    clearTimeout(fadeTimeout);
    document.body.classList.remove("fade-out");
}

function iniciarTimerFechamento(ms: number = 3500) {
    pausarTimerFechamento();
    fadeTimeout = window.setTimeout(() => {
        document.body.classList.add("fade-out");
        hideTimeout = window.setTimeout(async () => {
            await invoke("esconder_popup");
        }, 450);
    }, ms);
}

function tocarSomNotificacao() {
    const audio = new Audio(bellSound);
    audio.play().catch(() => {});
}

interface PayloadNotificacao {
    texto: string;
    tipo: string;
    link?: string | null;
    link_rotulo?: string | null;
    pode_responder?: boolean;
    tocar_som?: boolean;
}

(window as any).atualizarInterface = function(
    textoOuPayload: string | PayloadNotificacao,
    tipo?: string,
    link?: string | null,
    linkRotulo?: string | null,
    podeResponder?: boolean,
    tocarSom?: boolean
) {
    let texto: string;
    let tipoFinal = "normal";
    let linkFinal: string | null | undefined = null;
    let linkRotuloFinal: string | null | undefined = null;
    let podeResponderFinal = false;
    let deveTocarSom = false;

    if (typeof textoOuPayload === "object" && textoOuPayload !== null) {
        texto = textoOuPayload.texto;
        tipoFinal = textoOuPayload.tipo || "normal";
        linkFinal = textoOuPayload.link;
        linkRotuloFinal = textoOuPayload.link_rotulo;
        podeResponderFinal = textoOuPayload.pode_responder === true;
        deveTocarSom = textoOuPayload.tocar_som === true;
    } else {
        texto = textoOuPayload;
        tipoFinal = tipo || "normal";
        linkFinal = link;
        linkRotuloFinal = linkRotulo;
        podeResponderFinal = podeResponder === true;
        deveTocarSom = tocarSom === true;
    }

    pausarTimerFechamento();

    if (deveTocarSom) {
        tocarSomNotificacao();
    }

    if (btnResponder) {
        btnResponder.style.display = podeResponderFinal ? "inline-flex" : "none";
    }

    document.getElementById("mensagem")!.innerText = texto;
    const miiImg = document.getElementById("miiAvatar") as HTMLImageElement;

    // Garante que caso a imagem solicitada não seja encontrada, o fallback carregue a imagem mii-normal.png
    miiImg.onerror = () => {
        miiImg.onerror = null;
        miiImg.src = miiNormal;
    };

    // Troca de imagem do avatar apontando dinamicamente para a reação correspondente
    miiImg.src = obterImagemReacao(tipoFinal) || miiNormal;

    // Gerencia link de referência externa ou autoria
    const refContainer = document.getElementById("referenciaContainer");
    const btnLink = document.getElementById("btnLinkReferencia");
    const labelLink = document.getElementById("labelLinkReferencia");

    if (linkFinal && linkFinal.trim().startsWith("http")) {
        if (refContainer && btnLink && labelLink) {
            labelLink.innerText = linkRotuloFinal || "Acessar Fonte";
            btnLink.onclick = (e) => {
                e.stopPropagation();
                openUrl(linkFinal!).catch(() => window.open(linkFinal!, "_blank"));
            };
            refContainer.style.display = "flex";
        }
    } else if (refContainer) {
        refContainer.style.display = "none";
    }

    document.body.classList.remove("fade-out");

    iniciarTimerFechamento(4000);
};

// Pausar desaparecimento quando o mouse estiver sobre a notificação
const container = document.getElementById("container");
if (container) {
    container.addEventListener("mouseenter", () => {
        pausarTimerFechamento();
    });

    container.addEventListener("mouseleave", () => {
        iniciarTimerFechamento(3500);
    });
}

// Botão X
document.getElementById("fecharPopup")?.addEventListener("click", async () => {
    pausarTimerFechamento();
    document.body.classList.add("fade-out");
    setTimeout(async () => {
        await invoke("esconder_popup");
    }, 450);
});

// Botão Responder - Abre a janela de conversa sem sobreposição
const btnResponder = document.getElementById("btnResponder");
if (btnResponder) {
    btnResponder.addEventListener("click", async () => {
        clearTimeout(hideTimeout);
        clearTimeout(fadeTimeout);
        pausarTimerFechamento();
        await invoke("abrir_pergunta");
    });
}

async function carregarNotificacao() {
    try {
        const dados: any = await invoke("pegar_dados_notificacao");
        if (dados && dados.texto) {
            (window as any).atualizarInterface(dados);
        }
    } catch (e) {
        console.error("Erro ao carregar dados iniciais:", e);
    }
}

carregarNotificacao();
