import { invoke } from "@tauri-apps/api/core";

const userName = document.getElementById("userName") as HTMLInputElement;
const folderPath = document.getElementById("folderPath") as HTMLInputElement;
const btnSalvar = document.getElementById("btnSalvar");

async function carregarDadosDoPC() {
    try {
        const info: any = await invoke("get_system_info");
        userName.value = info.name;
        folderPath.value = info.downloads;
    } catch (e) {
        console.error("Erro ao pegar dados:", e);
        userName.placeholder = "Erro de conexão";
    }
}

// Executa imediatamente
carregarDadosDoPC();

if (btnSalvar) {
  btnSalvar.addEventListener("click", async () => {

    // Pede para o Rust esconder a janela e ir para a bandeja

    await invoke("esconder_janela");
  });
}