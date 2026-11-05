import { invoke } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";
import type { AppConfig, PayloadNotificacao } from "../types";

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

const REACTION_AVATARS: Record<string, string> = {
    normal: "/src/assets/reacoes/mii-normal.png",
    feliz: "/src/assets/reacoes/mii-feliz.png",
    explicando: "/src/assets/reacoes/mii-explicando.png",
    bocejo: "/src/assets/reacoes/mii-bocejo.png",
    frustrado: "/src/assets/reacoes/mii-frustrado.png",
    raiva: "/src/assets/reacoes/mii-raiva.png",
    lambendo: "/src/assets/reacoes/mii-lambendo.png",
    assustado: "/src/assets/reacoes/mii-assustado.png",
};

const TEXTS = {
    pt: {
        title: "Mensagens da IA",
        subtitle: (count: number) => `${count} ${count === 1 ? 'mensagem recente' : 'mensagens recentes'}`,
        btnClearTitle: "Limpar histórico",
        btnCloseTitle: "Fechar",
        empty: "Nenhuma mensagem recente da IA ainda.",
        btnCopy: "Copiar",
        btnCopied: "Copiado! ✓",
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
        btnCopied: "Copied! ✓",
        btnBubble: "Read in Bubble",
        tagMii: "Mii",
    },
};

async function atualizarIdioma() {
    try {
        const config = await invoke<AppConfig>("carregar_configuracoes");
        isEn = config && config.idioma === "en";
    } catch {
        isEn = false;
    }
}

function obterAvatarPorTipo(tipo: string): string {
    const limpo = (tipo || "").toLowerCase().trim();
    return REACTION_AVATARS[limpo] || REACTION_AVATARS.normal;
}

async function renderizarHistorico(forcar: boolean = false) {
    await atualizarIdioma();
    const t = isEn ? TEXTS.en : TEXTS.pt;

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

            // Link opcional
            if (item.link) {
                const linkAnchor = document.createElement("span");
                linkAnchor.className = "item-link";
                linkAnchor.textContent = `🔗 ${item.link_rotulo || item.link}`;
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

            // Botão "Ler no Balão"
            const bubbleBtn = document.createElement("button");
            bubbleBtn.type = "button";
            bubbleBtn.className = "btn-item-action";
            bubbleBtn.innerHTML = `💬 <span>${t.btnBubble}</span>`;
            bubbleBtn.onclick = async (e) => {
                e.stopPropagation();
                try {
                    await invoke("reexibir_notificacao_ia", { index });
                } catch (err) {
                    console.error("Erro ao reexibir no balão:", err);
                }
            };
            bottomRow.appendChild(bubbleBtn);

            // Botão "Copiar"
            const copyBtn = document.createElement("button");
            copyBtn.type = "button";
            copyBtn.className = "btn-item-action";
            copyBtn.innerHTML = `📋 <span>${t.btnCopy}</span>`;
            copyBtn.onclick = async (e) => {
                e.stopPropagation();
                try {
                    await invoke("copiar_item_clipboard", { texto: item.texto });
                    copyBtn.classList.add("copied");
                    copyBtn.innerHTML = `<span>${t.btnCopied}</span>`;
                    setTimeout(() => {
                        copyBtn.classList.remove("copied");
                        copyBtn.innerHTML = `📋 <span>${t.btnCopy}</span>`;
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
