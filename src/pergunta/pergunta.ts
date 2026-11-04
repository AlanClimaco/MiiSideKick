import { invoke } from "@tauri-apps/api/core";

const inputPergunta = document.getElementById("inputPergunta") as HTMLInputElement;
const btnEnviar = document.getElementById("btnEnviar");
const btnFechar = document.getElementById("btnFechar");

async function aplicarIdioma() {
    try {
        const config: any = await invoke("carregar_configuracoes");
        const isEn = config && config.idioma === "en";
        const titleEl = document.querySelector(".pergunta-title");
        if (titleEl) {
            titleEl.textContent = isEn ? "Chat with Mii" : "Conversar com o Mii";
        }
        if (inputPergunta) {
            inputPergunta.placeholder = isEn ? "Ask Mii anything..." : "Pergunte algo ao Mii...";
            inputPergunta.setAttribute("aria-label", isEn ? "Question for Mii" : "Pergunta para o Mii");
        }
        if (btnEnviar) {
            btnEnviar.title = isEn ? "Ask" : "Perguntar";
            btnEnviar.setAttribute("aria-label", isEn ? "Send question" : "Enviar pergunta");
        }
        if (btnFechar) {
            btnFechar.title = isEn ? "Close" : "Fechar";
            btnFechar.setAttribute("aria-label", isEn ? "Close chat" : "Fechar conversa");
        }
    } catch {}
}

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

aplicarIdioma();

window.addEventListener("focus", () => {
    aplicarIdioma();
    inputPergunta?.focus();
});
