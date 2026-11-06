import { invoke } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";
import type { AppConfig, PayloadNotificacao } from "../types";

// Importações relativas de assets para empacotamento estático garantido pelo Vite
import miiNormal from "../assets/sprites/normal.png";
import miiFeliz from "../assets/sprites/feliz.png";
import miiExplicando from "../assets/sprites/explicando.png";
import miiBocejo from "../assets/sprites/bocejo.png";
import miiFrustrado from "../assets/sprites/frustrado.png";
import miiRaiva from "../assets/sprites/raiva.png";
import miiLambendo from "../assets/sprites/lambendo.png";
import miiAssustado from "../assets/sprites/assustado.png";

// Ícones vetoriais SVG limpos em substituição completa de emojis
const SVG_ICONS = {
    copy: `<svg viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path></svg>`,
    check: `<svg viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><polyline points="20 6 9 17 4 12"></polyline></svg>`,
    bubble: `<svg viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"></path></svg>`,
    link: `<svg viewBox="0 0 24 24" width="11" height="11" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"></path><path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"></path></svg>`,
};

const historicoAvatar = document.getElementById("historicoAvatar") as HTMLImageElement | null;
const historicoTitle = document.getElementById("historicoTitle");
const historicoSubtitle = document.getElementById("historicoSubtitle");
const btnLimpar = document.getElementById("btnLimpar");
const btnFechar = document.getElementById("btnFechar");
const historicoList = document.getElementById("historicoList");
const historicoEmpty = document.getElementById("historicoEmpty");
const emptyMessage = document.getElementById("emptyMessage");

let isEn = false;
let isMouseInside = false;
let ultimaAssinatura = "";

// Mapa base de avatares com suporte a aliases
const REACTION_AVATARS: Record<string, string> = {
    normal: miiNormal,
    feliz: miiFeliz,
    explicando: miiExplicando,
    bocejo: miiBocejo,
    frustrado: miiFrustrado,
    raiva: miiRaiva,
    lambendo: miiLambendo,
    assustado: miiAssustado,
    bravo: miiRaiva,
    irritado: miiRaiva,
    erro: miiFrustrado,
    tranquilo: miiFeliz,
    sucesso: miiFeliz,
};

// Resolução dinâmica adicional de assets compatível com Vite para janelas Tauri
const spriteModules = import.meta.glob<string>(
    ["../assets/sprites/*.png", "../assets/reacoes/*.png"],
    { eager: true, import: "default" }
);

for (const [path, url] of Object.entries(spriteModules)) {
    const fileName = path.split("/").pop()?.replace(/\.[^/.]+$/, "").toLowerCase() || "";
    if (fileName) {
        if (!REACTION_AVATARS[fileName]) {
            REACTION_AVATARS[fileName] = url;
        }
        if (fileName.startsWith("mii-")) {
            const alias = fileName.replace("mii-", "");
            if (!REACTION_AVATARS[alias]) {
                REACTION_AVATARS[alias] = url;
            }
        }
    }
}

const TEXTS = {
    pt: {
        title: "Mensagens da IA",
        subtitle: (count: number) => `${count} ${count === 1 ? 'mensagem recente' : 'mensagens recentes'}`,
        btnClearTitle: "Limpar histórico",
        btnCloseTitle: "Fechar",
        empty: "Nenhuma mensagem recente da IA ainda.",
        btnCopy: "Copiar",
        btnCopied: "Copiado!",
        btnBubble: "Ler no Balão",
        tagMii: "Mii",
    },
    en: {
        title: "Recent AI Messages",
        subtitle: (count: number) => `${count} recent ${count === 1 ? 'message' : 'messages'}`,
        btnClearTitle: "Clear history",
        btnCloseTitle: "Close",
        empty: "No recent AI messages yet.",
        btnCopy: "Copy",
        btnCopied: "Copied!",
        btnBubble: "Read in Bubble",
        tagMii: "Mii",
    },
};

function obterAvatarPorTipo(tipo: string): string {
    const limpo = (tipo || "normal").toLowerCase().trim();
    return REACTION_AVATARS[limpo] || REACTION_AVATARS.normal || miiNormal;
}

if (historicoAvatar) {
    historicoAvatar.src = obterAvatarPorTipo("feliz");
}

async function atualizarIdioma() {
    try {
        const config = await invoke<AppConfig>("carregar_configuracoes");
        isEn = config && config.idioma === "en";
    } catch {
        isEn = false;
    }
}

async function renderizarHistorico(forcar: boolean = false) {
    await atualizarIdioma();
    const t = isEn ? TEXTS.en : TEXTS.pt;

    if (historicoAvatar) {
        historicoAvatar.src = obterAvatarPorTipo("feliz");
    }
    if (historicoTitle) historicoTitle.textContent = t.title;
    if (btnLimpar) {
        btnLimpar.title = t.btnClearTitle;
        btnLimpar.setAttribute("aria-label", t.btnClearTitle);
    }
    if (btnFechar) {
        btnFechar.title = t.btnCloseTitle;
        btnFechar.setAttribute("aria-label", t.btnCloseTitle);
    }
    if (emptyMessage) emptyMessage.textContent = t.empty;

    try {
        const items = await invoke<PayloadNotificacao[]>("obter_historico_ia");

        if (!items || items.length === 0) {
            ultimaAssinatura = "";
            if (historicoSubtitle) historicoSubtitle.textContent = t.subtitle(0);
            if (historicoList) historicoList.innerHTML = "";
            if (historicoEmpty) historicoEmpty.style.display = "flex";
            return;
        }

        const novaAssinatura = items.map((i) => `${i.tipo}:${i.timestamp || 0}:${i.texto}`).join("||");

        if (!forcar && novaAssinatura === ultimaAssinatura) {
            return;
        }
        ultimaAssinatura = novaAssinatura;

        if (historicoSubtitle) historicoSubtitle.textContent = t.subtitle(items.length);
        if (historicoEmpty) historicoEmpty.style.display = "none";
        if (!historicoList) return;

        historicoList.innerHTML = "";

        items.forEach((item, index) => {
            const card = document.createElement("div");
            card.className = "historico-item";

            // Top row: Avatar + Nome + Horário
            const topRow = document.createElement("div");
            topRow.className = "item-top-row";

            const badge = document.createElement("div");
            badge.className = "item-reaction-badge";

            const miniAvatar = document.createElement("img");
            miniAvatar.className = "item-mini-avatar";
            miniAvatar.src = obterAvatarPorTipo(item.tipo);
            miniAvatar.alt = item.tipo || "Mii";
            miniAvatar.onerror = () => {
                miniAvatar.src = REACTION_AVATARS.normal || miiNormal;
            };

            const tagText = document.createElement("span");
            tagText.textContent = t.tagMii;

            badge.appendChild(miniAvatar);
            badge.appendChild(tagText);
            topRow.appendChild(badge);

            const timeSpan = document.createElement("span");
            timeSpan.className = "item-time";
            const horaStr = item.timestamp
                ? new Date(item.timestamp).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" })
                : (item.hora_formatada || "");
            timeSpan.textContent = horaStr;
            topRow.appendChild(timeSpan);

            card.appendChild(topRow);

            // Text
            const textP = document.createElement("p");
            textP.className = "item-text";
            textP.textContent = item.texto;
            card.appendChild(textP);

            // Link opcional com ícone SVG limpo
            if (item.link) {
                const linkAnchor = document.createElement("span");
                linkAnchor.className = "item-link";
                linkAnchor.innerHTML = `${SVG_ICONS.link}<span></span>`;
                const linkSpan = linkAnchor.querySelector("span");
                if (linkSpan) {
                    linkSpan.textContent = item.link_rotulo || item.link;
                }
                linkAnchor.onclick = (e) => {
                    e.stopPropagation();
                    if (item.link) {
                        openUrl(item.link).catch(console.error);
                    }
                };
                card.appendChild(linkAnchor);
            }

            // Bottom actions row
            const bottomRow = document.createElement("div");
            bottomRow.className = "item-bottom-row";

            // Botão "Ler no Balão" (SVG limpo + texto)
            const bubbleBtn = document.createElement("button");
            bubbleBtn.type = "button";
            bubbleBtn.className = "btn-item-action";
            bubbleBtn.innerHTML = `${SVG_ICONS.bubble}<span>${t.btnBubble}</span>`;
            bubbleBtn.onclick = async (e) => {
                e.stopPropagation();
                try {
                    await invoke("reexibir_notificacao_ia", { index });
                } catch (err) {
                    console.error("Erro ao reexibir no balão:", err);
                }
            };
            bottomRow.appendChild(bubbleBtn);

            // Botão "Copiar" (SVG limpo + texto)
            const copyBtn = document.createElement("button");
            copyBtn.type = "button";
            copyBtn.className = "btn-item-action";
            copyBtn.innerHTML = `${SVG_ICONS.copy}<span>${t.btnCopy}</span>`;
            copyBtn.onclick = async (e) => {
                e.stopPropagation();
                try {
                    await invoke("copiar_item_clipboard", { texto: item.texto });
                    copyBtn.classList.add("copied");
                    copyBtn.innerHTML = `${SVG_ICONS.check}<span>${t.btnCopied}</span>`;
                    setTimeout(() => {
                        copyBtn.classList.remove("copied");
                        copyBtn.innerHTML = `${SVG_ICONS.copy}<span>${t.btnCopy}</span>`;
                    }, 1200);
                } catch (err) {
                    console.error("Erro ao copiar texto:", err);
                }
            };
            bottomRow.appendChild(copyBtn);

            card.appendChild(bottomRow);
            historicoList.appendChild(card);
        });
    } catch (e) {
        console.error("Erro ao obter histórico da IA:", e);
    }
}

btnLimpar?.addEventListener("click", async () => {
    try {
        await invoke("limpar_historico_ia");
        await renderizarHistorico(true);
    } catch (e) {
        console.error("Erro ao limpar histórico da IA:", e);
    }
});

btnFechar?.addEventListener("click", async () => {
    try {
        await invoke("esconder_historico");
    } catch (e) {
        console.error("Erro ao esconder janela de histórico:", e);
    }
});

document.addEventListener("mouseenter", () => {
    isMouseInside = true;
});

document.addEventListener("mouseleave", () => {
    isMouseInside = false;
});

window.addEventListener("blur", () => {
    if (!isMouseInside) {
        invoke("esconder_historico").catch(() => {});
    }
});

window.addEventListener("focus", () => {
    renderizarHistorico(true);
});

document.addEventListener("keydown", (e) => {
    if (e.key === "Escape") {
        invoke("esconder_historico").catch(() => {});
    }
});

setInterval(() => {
    if (!document.hidden) {
        renderizarHistorico(false);
    }
}, 2000);

renderizarHistorico(true);
