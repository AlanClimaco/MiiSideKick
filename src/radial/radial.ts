import { invoke } from "@tauri-apps/api/core";
import type { AppConfig } from "../types";

// Botões das 9 opções + fechar
const btnOrganizar = document.getElementById("btnOrganizar");
const btnOtimizarRAM = document.getElementById("btnOtimizarRAM");
const btnLimparLixo = document.getElementById("btnLimparLixo");
const btnTimer = document.getElementById("btnTimer");
const btnSetup = document.getElementById("btnSetup");
const btnPergunta = document.getElementById("btnPergunta");
const btnClipboard = document.getElementById("btnClipboard");
const btnHistoricoIA = document.getElementById("btnHistoricoIA");
const btnBusca = document.getElementById("btnBusca");
const btnFecharRadial = document.getElementById("btnFecharRadial");

// Rótulos de texto
const labelOrganizar = document.getElementById("labelOrganizar");
const labelOtimizarRAM = document.getElementById("labelOtimizarRAM");
const labelLimparLixo = document.getElementById("labelLimparLixo");
const labelTimer = document.getElementById("labelTimer");
const labelSetup = document.getElementById("labelSetup");
const labelPergunta = document.getElementById("labelPergunta");
const labelClipboard = document.getElementById("labelClipboard");
const labelHistoricoIA = document.getElementById("labelHistoricoIA");
const labelBusca = document.getElementById("labelBusca");

async function aplicarIdioma() {
    try {
        const config = await invoke<AppConfig>("carregar_configuracoes");
        const isEn = config && config.idioma === "en";

        if (labelOrganizar) {
            labelOrganizar.textContent = isEn ? "Organize Downloads" : "Organizar Downloads";
        }
        if (labelOtimizarRAM) {
            labelOtimizarRAM.textContent = isEn ? "Optimize RAM" : "Otimizar RAM";
        }
        if (labelLimparLixo) {
            labelLimparLixo.textContent = isEn ? "Empty Recycle Bin" : "Esvaziar Lixeira";
        }
        if (labelTimer) {
            labelTimer.textContent = isEn ? "New Reminder" : "Novo Lembrete";
        }
        if (labelSetup) {
            labelSetup.textContent = isEn ? "Settings" : "Configurações";
        }
        if (labelPergunta) {
            labelPergunta.textContent = isEn ? "Chat with Mii" : "Conversar com Mii";
        }
        if (labelClipboard) {
            labelClipboard.textContent = isEn ? "Clipboard History" : "Histórico Copiar";
        }
        if (labelHistoricoIA) {
            labelHistoricoIA.textContent = isEn ? "AI History" : "Histórico da IA";
        }
        if (labelBusca) {
            labelBusca.textContent = isEn ? "Search Files" : "Buscar Arquivos";
        }
        if (btnBusca) {
            btnBusca.title = isEn ? "Search Files" : "Buscar Arquivos";
            btnBusca.setAttribute("aria-label", isEn ? "Search files on PC" : "Buscar arquivos no PC");
        }
        if (btnFecharRadial) {
            btnFecharRadial.title = isEn ? "Close Menu" : "Fechar Menu";
            btnFecharRadial.setAttribute("aria-label", isEn ? "Close radial menu" : "Fechar menu radial");
        }
    } catch (e) {
        console.error("Erro ao aplicar idioma no menu radial:", e);
    }
}

async function fecharMenu() {
    try {
        await invoke("esconder_radial");
    } catch (e) {
        console.error("Erro ao fechar menu radial:", e);
    }
}

// 1. Norte: Organizar Downloads
btnOrganizar?.addEventListener("click", async () => {
    fecharMenu();
    await invoke("organizar_downloads");
});

// 2. Nordeste: Otimizar RAM
btnOtimizarRAM?.addEventListener("click", async () => {
    fecharMenu();
    await invoke("limpar_cache_ram");
});

// 3. Leste: Esvaziar Lixeira
btnLimparLixo?.addEventListener("click", async () => {
    fecharMenu();
    await invoke("limpar_lixo_pc");
});

// 4. Sudeste: Novo Timer / Lembrete
btnTimer?.addEventListener("click", async () => {
    fecharMenu();
    await invoke("abrir_timer");
});

// 5. Sul: Configurações / Setup
btnSetup?.addEventListener("click", async () => {
    fecharMenu();
    await invoke("abrir_configuracoes");
});

// 6. Sudoeste: Conversar com o Mii
btnPergunta?.addEventListener("click", async () => {
    fecharMenu();
    await invoke("abrir_pergunta");
});

// 7. Oeste: Histórico do Clipboard
btnClipboard?.addEventListener("click", async () => {
    fecharMenu();
    await invoke("abrir_clipboard");
});

// 8. Noroeste: Histórico da IA
btnHistoricoIA?.addEventListener("click", async () => {
    fecharMenu();
    await invoke("abrir_historico");
});

// 9. Noroeste-Alto: Buscar Arquivos no PC
btnBusca?.addEventListener("click", async () => {
    fecharMenu();
    await invoke("abrir_busca");
});

// Centro: Botão Fechar (X)
btnFecharRadial?.addEventListener("click", () => {
    fecharMenu();
});

// Fechar com tecla Escape
window.addEventListener("keydown", (e: KeyboardEvent) => {
    if (e.key === "Escape") {
        e.preventDefault();
        fecharMenu();
    }
});

// Fechar ao perder o foco (clicar fora)
window.addEventListener("blur", () => {
    fecharMenu();
});

aplicarIdioma();

window.addEventListener("focus", () => {
    aplicarIdioma();
});
