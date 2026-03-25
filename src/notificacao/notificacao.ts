import { invoke } from "@tauri-apps/api/core";

// Relógios globais para cancelar se o usuário clicar no X
let hideTimeout: number;
let fadeTimeout: number;

(window as any).atualizarInterface = function(texto: string, tipo: string) {
    clearTimeout(hideTimeout);
    clearTimeout(fadeTimeout);

    document.getElementById("mensagem")!.innerText = texto;
    const miiImg = document.getElementById("miiAvatar") as HTMLImageElement;

    if (tipo === "frustrado" || tipo === "erro") {
        miiImg.src = "/src/assets/reacoes/mii-frustrado.png";
    } else if (tipo === "tranquilo") {
        miiImg.src = "/src/assets/reacoes/mii-tranquilo.png";
    } else if (tipo === "assustado") {
        miiImg.src = "/src/assets/reacoes/mii-assustado.png";
    } else {
        miiImg.src = "/src/assets/reacoes/mii-normal.png";
    }

    document.body.classList.remove("fade-out");

    fadeTimeout = window.setTimeout(() => {
        document.body.classList.add("fade-out");
        hideTimeout = window.setTimeout(async () => {
            await invoke("esconder_popup");
        }, 450);
    }, 4000);
};

// Botão X
document.getElementById("fecharPopup")?.addEventListener("click", async () => {
    clearTimeout(hideTimeout);
    clearTimeout(fadeTimeout);
    
    // Anima o body inteiro para o X sumir junto
    document.body.classList.add("fade-out");
    setTimeout(async () => {
        await invoke("esconder_popup");
    }, 450);
});

// Ao iniciar, vê se já tem alguma mensagem
async function carregarNotificacao() {
    try {
        const dados: any = await invoke("pegar_dados_notificacao");
        if (dados && dados.texto) {
            (window as any).atualizarInterface(dados.texto, dados.tipo);
        }
    } catch (e) {
        console.error("Erro ao carregar dados iniciais:", e);
    }
}

carregarNotificacao();