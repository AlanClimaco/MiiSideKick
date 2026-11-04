import { invoke } from "@tauri-apps/api/core";

async function aplicarIdioma() {
    try {
        const config: any = await invoke("carregar_configuracoes");
        const isEn = config && config.idioma === "en";
        const titleEl = document.querySelector(".timer-title");
        if (titleEl) {
            titleEl.textContent = isEn ? "New Reminder" : "Novo Lembrete";
        }
        const labelTempo = document.querySelector('label[for="minutos"]');
        if (labelTempo) {
            labelTempo.textContent = isEn ? "Time (in minutes):" : "Tempo (em minutos):";
        }
        const labelMsg = document.querySelector('label[for="mensagem"]');
        if (labelMsg) {
            labelMsg.textContent = isEn ? "Remind about what?" : "Avisar sobre o quê?";
        }
        const inputMsg = document.getElementById("mensagem") as HTMLInputElement;
        if (inputMsg) {
            inputMsg.placeholder = isEn ? "Take a break..." : "Comer cuscuz...";
        }
        const btnIniciar = document.getElementById("btnIniciar");
        if (btnIniciar) {
            btnIniciar.textContent = isEn ? "Start" : "Iniciar";
        }
        const btnFechar = document.getElementById("btnFechar");
        if (btnFechar) {
            btnFechar.textContent = isEn ? "Cancel" : "Cancelar";
        }
    } catch {}
}

document.getElementById("btnIniciar")?.addEventListener("click", async () => {
    const minutosStr = (document.getElementById("minutos") as HTMLInputElement).value;
    const config: any = await invoke("carregar_configuracoes").catch(() => null);
    const isEn = config && config.idioma === "en";
    const msgPadrao = isEn ? "Time is up!" : "O tempo acabou!";
    const mensagem = (document.getElementById("mensagem") as HTMLInputElement).value || msgPadrao;
    
    const minutos = parseInt(minutosStr, 10);
    
    if (minutos > 0) {
        await invoke("iniciar_timer", { minutos, mensagem });
        await invoke("esconder_timer");
    }
});

document.getElementById("btnFechar")?.addEventListener("click", async () => {
    await invoke("esconder_timer");
});

aplicarIdioma();
window.addEventListener("focus", () => {
    aplicarIdioma();
});
