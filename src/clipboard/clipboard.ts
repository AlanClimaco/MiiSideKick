import { invoke } from "@tauri-apps/api/core";
import type { AppConfig, ClipboardItem } from "../types";

const clipboardTitle = document.getElementById("clipboardTitle");
const clipboardSubtitle = document.getElementById("clipboardSubtitle");
const btnLimpar = document.getElementById("btnLimpar");
const btnFechar = document.getElementById("btnFechar");
const clipboardList = document.getElementById("clipboardList");
const clipboardEmpty = document.getElementById("clipboardEmpty");
const emptyMessage = document.getElementById("emptyMessage");

let isEn = false;
let isMouseInside = false;
let ultimaAssinatura = "";

const TEXTS = {
    pt: {
        title: "Área de Transferência",
        subtitle: (count: number) => `${count} ${count === 1 ? 'item recente' : 'itens recentes'}`,
        btnClearTitle: "Limpar histórico",
        btnCloseTitle: "Fechar",
        empty: "Nenhum texto copiado ainda.",
        btnCopy: "Copiar",
        btnCopied: "Copiado! ✓",
        showMore: "Ver mais",
        showLess: "Ver menos",
        chars: "caracteres",
    },
    en: {
        title: "Clipboard History",
        subtitle: (count: number) => `${count} recent ${count === 1 ? 'item' : 'items'}`,
        btnClearTitle: "Clear history",
        btnCloseTitle: "Close",
        empty: "No copied text yet.",
        btnCopy: "Copy",
        btnCopied: "Copied! ✓",
        showMore: "Show more",
        showLess: "Show less",
        chars: "chars",
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

async function renderizarHistorico(forcar: boolean = false) {
    await atualizarIdioma();
    const t = isEn ? TEXTS.en : TEXTS.pt;

    if (clipboardTitle) clipboardTitle.textContent = t.title;
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
        const items = await invoke<ClipboardItem[]>("obter_historico_clipboard");

        if (!items || items.length === 0) {
            ultimaAssinatura = "";
            if (clipboardSubtitle) clipboardSubtitle.textContent = t.subtitle(0);
            if (clipboardList) clipboardList.innerHTML = "";
            if (clipboardEmpty) clipboardEmpty.style.display = "flex";
            return;
        }

        const novaAssinatura = items.map((i) => `${i.id || ""}:${i.texto}`).join("||");

        // Se a lista não mudou, evita reconstruir o DOM e resetar o scroll
        if (!forcar && novaAssinatura === ultimaAssinatura) {
            return;
        }

        // Se o usuário estiver interagindo com o mouse dentro da janela, posterga a recriação do DOM
        if (!forcar && isMouseInside) {
            return;
        }

        ultimaAssinatura = novaAssinatura;

        if (clipboardEmpty) clipboardEmpty.style.display = "none";
        if (clipboardSubtitle) clipboardSubtitle.textContent = t.subtitle(items.length);

        if (!clipboardList) return;
        clipboardList.innerHTML = "";

        items.forEach((item) => {
            const card = document.createElement("div");
            card.className = "clipboard-item-card";

            const contentEl = document.createElement("div");
            contentEl.className = "item-content";
            contentEl.textContent = item.texto;
            card.appendChild(contentEl);

            const textoLongo = item.texto.length > 90 || item.texto.includes("\n");
            if (textoLongo) {
                const toggleBtn = document.createElement("button");
                toggleBtn.type = "button";
                toggleBtn.className = "btn-toggle-expand";
                toggleBtn.textContent = t.showMore;
                toggleBtn.onclick = (e) => {
                    e.stopPropagation();
                    const expandido = contentEl.classList.toggle("expanded");
                    toggleBtn.textContent = expandido ? t.showLess : t.showMore;
                };
                card.appendChild(toggleBtn);
            }

            const footer = document.createElement("div");
            footer.className = "item-footer";

            const meta = document.createElement("div");
            meta.className = "item-meta";

            const horaFormatada = item.timestamp
                ? new Date(item.timestamp).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" })
                : item.hora_formatada;

            meta.textContent = `${horaFormatada} • ${item.texto.length} ${t.chars}`;
            footer.appendChild(meta);

            const copyBtn = document.createElement("button");
            copyBtn.type = "button";
            copyBtn.className = "btn-copy";
            copyBtn.innerHTML = `<svg viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path></svg> <span>${t.btnCopy}</span>`;

            copyBtn.onclick = async (e) => {
                e.stopPropagation();
                try {
                    await invoke("copiar_item_clipboard", { texto: item.texto });
                    copyBtn.classList.add("copied");
                    copyBtn.innerHTML = `<span>${t.btnCopied}</span>`;
                    setTimeout(() => {
                        copyBtn.classList.remove("copied");
                        copyBtn.innerHTML = `<svg viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path></svg> <span>${t.btnCopy}</span>`;
                    }, 1200);
                } catch (err) {
                    console.error("Erro ao copiar item:", err);
                }
            };

            footer.appendChild(copyBtn);
            card.appendChild(footer);

            clipboardList.appendChild(card);
        });
    } catch (e) {
        console.error("Erro ao obter histórico do clipboard:", e);
    }
}

btnLimpar?.addEventListener("click", async () => {
    try {
        await invoke("limpar_historico_clipboard");
        await renderizarHistorico(true);
    } catch (e) {
        console.error("Erro ao limpar clipboard:", e);
    }
});

btnFechar?.addEventListener("click", async () => {
    try {
        await invoke("esconder_clipboard");
    } catch (e) {
        console.error("Erro ao esconder janela do clipboard:", e);
    }
});

// Manipuladores de mouse para pausar fechamento por blur durante interação
document.addEventListener("mouseenter", () => {
    isMouseInside = true;
});

document.addEventListener("mouseleave", () => {
    isMouseInside = false;
});

window.addEventListener("blur", () => {
    // Só fecha a janela se o cursor não estiver interagindo ativamente com o conteúdo
    if (!isMouseInside) {
        invoke("esconder_clipboard").catch(() => {});
    }
});

window.addEventListener("focus", () => {
    renderizarHistorico(true);
});

// Atualização periódica de 1.5s enquanto aberta
setInterval(() => {
    if (!document.hidden) {
        renderizarHistorico(false);
    }
}, 1500);

renderizarHistorico(true);
