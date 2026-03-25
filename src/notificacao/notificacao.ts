import { invoke } from "@tauri-apps/api/core";

import miiNormal from '../assets/reacoes/mii-normal.png';
import miiFrustrado from '../assets/reacoes/mii-frustrado.png';
import miiTranquilo from '../assets/reacoes/mii-tranquilo.png';
import miiAssustado from '../assets/reacoes/mii-assustado.png';

let hideTimeout: number;
let fadeTimeout: number;

(window as any).atualizarInterface = function(texto: string, tipo: string) {
    clearTimeout(hideTimeout);
    clearTimeout(fadeTimeout);

    document.getElementById("mensagem")!.innerText = texto;
    const miiImg = document.getElementById("miiAvatar") as HTMLImageElement;

    if (tipo === "frustrado" || tipo === "erro") {
        miiImg.src = miiFrustrado;
    } else if (tipo === "tranquilo" || tipo === "sucesso") {
        miiImg.src = miiTranquilo;
    } else if (tipo === "assustado") {
        miiImg.src = miiAssustado; 
    } else {
        miiImg.src = miiNormal;
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
    
    document.body.classList.add("fade-out");
    setTimeout(async () => {
        await invoke("esconder_popup");
    }, 450);
});

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