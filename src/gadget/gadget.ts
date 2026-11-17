import { invoke } from "@tauri-apps/api/core";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import type { TelemetriaRapida } from "../types";

const appWindow = getCurrentWebviewWindow();

const cpuVal = document.getElementById("cpuVal") as HTMLSpanElement | null;
const cpuBar = document.getElementById("cpuBar") as HTMLDivElement | null;
const ramVal = document.getElementById("ramVal") as HTMLSpanElement | null;
const ramBar = document.getElementById("ramBar") as HTMLDivElement | null;
const btnFechar = document.getElementById("btnFecharGadget") as HTMLButtonElement | null;
const btnAcaoLixo = document.getElementById("btnAcaoLixo") as HTMLButtonElement | null;
const btnAcaoOrganizar = document.getElementById("btnAcaoOrganizar") as HTMLButtonElement | null;
const btnAcaoOlharTela = document.getElementById("btnAcaoOlharTela") as HTMLButtonElement | null;
const gadgetStatus = document.getElementById("gadgetStatus") as HTMLDivElement | null;

let statusTimeout: number | null = null;

function exibirStatus(texto: string, duracaoMs = 2500) {
    if (!gadgetStatus) return;
    if (statusTimeout) clearTimeout(statusTimeout);
    gadgetStatus.textContent = texto;
    gadgetStatus.style.display = "block";
    statusTimeout = window.setTimeout(() => {
        gadgetStatus.style.display = "none";
    }, duracaoMs);
}

// 1. Atualização Contínua de Telemetria
async function atualizarTelemetria() {
    try {
        const dados = await invoke<TelemetriaRapida>("obter_telemetria_rapida");
        if (dados) {
            const cpuPct = Math.min(100, Math.max(0, Math.round(dados.cpu_uso)));
            const ramPct = Math.min(100, Math.max(0, Math.round(dados.ram_uso_pct)));

            if (cpuVal) cpuVal.textContent = `${cpuPct}%`;
            if (cpuBar) cpuBar.style.width = `${cpuPct}%`;

            if (ramVal) ramVal.textContent = `${ramPct}%`;
            if (ramBar) ramBar.style.width = `${ramPct}%`;
        }
    } catch (e) {
        console.error("Erro ao obter telemetria:", e);
    }
}

// Inicia polling a cada 2 segundos
atualizarTelemetria();
setInterval(atualizarTelemetria, 2000);

// 2. Fechar / Ocultar Gadget
if (btnFechar) {
    btnFechar.addEventListener("click", async () => {
        try {
            await invoke("alternar_gadget", { ativo: false });
        } catch (e) {
            console.error("Erro ao alternar gadget:", e);
        }
    });
}

// 3. Ações Rápidas
if (btnAcaoLixo) {
    btnAcaoLixo.addEventListener("click", async () => {
        exibirStatus("Limpando lixo...");
        try {
            await invoke("limpar_lixo_pc");
            exibirStatus("Lixo e cache limpos!");
            await atualizarTelemetria();
        } catch (e) {
            console.error("Erro ao limpar lixo:", e);
            exibirStatus("Erro ao limpar lixo.");
        }
    });
}

if (btnAcaoOrganizar) {
    btnAcaoOrganizar.addEventListener("click", async () => {
        exibirStatus("Organizando...");
        try {
            await invoke("organizar_downloads");
            exibirStatus("Downloads organizados!");
        } catch (e) {
            console.error("Erro ao organizar downloads:", e);
            exibirStatus("Erro ao organizar.");
        }
    });
}

if (btnAcaoOlharTela) {
    btnAcaoOlharTela.addEventListener("click", async () => {
        exibirStatus("Olhando tela...");
        try {
            await invoke("analisar_tela_com_ia");
            exibirStatus("Tela analisada!");
        } catch (e) {
            console.error("Erro ao analisar tela:", e);
            exibirStatus("Erro ao capturar tela.");
        }
    });
}

// 4. Persistência de Posição do Gadget na Área de Trabalho
let moveDebounceTimer: number | null = null;

async function persistirPosicao() {
    try {
        const pos = await appWindow.outerPosition();
        if (pos) {
            await invoke("salvar_posicao_gadget", { x: Math.round(pos.x), y: Math.round(pos.y) });
        }
    } catch (e) {
        console.error("Erro ao salvar posição:", e);
    }
}

// Ouvinte do evento nativo onMoved do Tauri v2
try {
    appWindow.onMoved(({ payload: pos }) => {
        if (moveDebounceTimer) clearTimeout(moveDebounceTimer);
        moveDebounceTimer = window.setTimeout(async () => {
            try {
                await invoke("salvar_posicao_gadget", { x: Math.round(pos.x), y: Math.round(pos.y) });
            } catch (err) {
                console.error("Erro ao salvar posição onMoved:", err);
            }
        }, 500);
    });
} catch (_) {}

// Fallback no evento pointerup após arrastar a janela
window.addEventListener("pointerup", () => {
    if (moveDebounceTimer) clearTimeout(moveDebounceTimer);
    moveDebounceTimer = window.setTimeout(persistirPosicao, 300);
});
