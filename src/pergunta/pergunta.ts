import { invoke } from "@tauri-apps/api/core";

const inputPergunta = document.getElementById("inputPergunta") as HTMLInputElement;
const btnEnviar = document.getElementById("btnEnviar");
const btnFechar = document.getElementById("btnFechar");

async function enviarPergunta() {
    if (!inputPergunta) return;
    const texto = inputPergunta.value.trim();
    if (!texto) return;

    // Dispara a pergunta no backend Rust
    await invoke("perguntar_ao_mii", { pergunta: texto });

    // Limpa o campo e fecha a janela
    inputPergunta.value = "";
    await invoke("esconder_pergunta");
}

inputPergunta?.addEventListener("keydown", async (event: KeyboardEvent) => {
    if (event.key === "Enter") {
        event.preventDefault();
        await enviarPergunta();
    }
});

btnEnviar?.addEventListener("click", async () => {
    await enviarPergunta();
});

btnFechar?.addEventListener("click", async () => {
    if (inputPergunta) inputPergunta.value = "";
    await invoke("esconder_pergunta");
});

window.addEventListener("focus", () => {
    inputPergunta?.focus();
});
