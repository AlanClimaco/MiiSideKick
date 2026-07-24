import { invoke } from "@tauri-apps/api/core";
import { enable, disable, isEnabled } from '@tauri-apps/plugin-autostart';

const userName = document.getElementById("userName") as HTMLInputElement;
const folderPath = document.getElementById("folderPath") as HTMLInputElement;
const btnSalvar = document.getElementById("btnSalvar");
const checkAutostart = document.getElementById("checkAutostart") as HTMLInputElement;

async function carregarDadosDoPC() {
    try {
        // 1. Pega os dados do Rust (como você já fazia)
        const info: any = await invoke("get_system_info");
        userName.value = info.name;
        if (folderPath) folderPath.value = info.downloads;
        
        // 2. Verifica se o Autostart já está ativado no Windows e marca a caixinha
        const autostartAtivo = await isEnabled();
        if (checkAutostart) checkAutostart.checked = autostartAtivo;

    } catch (e) {
        console.error("Erro ao carregar dados:", e);
        userName.placeholder = "Erro de conexão";
    }
}

// Executa imediatamente
carregarDadosDoPC();

if (btnSalvar) {
  btnSalvar.addEventListener("click", async () => {
    
    // 3. Aplica a configuração do Autostart escolhida pelo usuário
    try {
        if (checkAutostart && checkAutostart.checked) {
            await enable();
        } else {
            await disable();
        }
    } catch (error) {
        console.error("Erro ao configurar autostart:", error);
    }

    // Pede para o Rust esconder a janela e ir para a bandeja
    await invoke("esconder_janela");
  });
}