import { invoke } from "@tauri-apps/api/core";

document.getElementById("btnIniciar")?.addEventListener("click", async () => {
    const minutosStr = (document.getElementById("minutos") as HTMLInputElement).value;
    const mensagem = (document.getElementById("mensagem") as HTMLInputElement).value || "O tempo acabou!";
    
    const minutos = parseInt(minutosStr, 10);
    
    if (minutos > 0) {
        await invoke("iniciar_timer", { minutos, mensagem });
        await invoke("esconder_timer");
    }
});

document.getElementById("btnFechar")?.addEventListener("click", async () => {
    await invoke("esconder_timer");
});