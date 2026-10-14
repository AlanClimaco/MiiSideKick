import { invoke } from "@tauri-apps/api/core";
import { enable, disable, isEnabled } from '@tauri-apps/plugin-autostart';
import { openUrl } from '@tauri-apps/plugin-opener';

const userName = document.getElementById("userName") as HTMLInputElement;
const geminiApiKey = document.getElementById("geminiApiKey") as HTMLInputElement;
const folderPath = document.getElementById("folderPath") as HTMLInputElement;
const btnSalvar = document.getElementById("btnSalvar");
const checkAutostart = document.getElementById("checkAutostart") as HTMLInputElement;

async function carregarDadosDoPC() {
    try {
        // 1. Carrega as configurações salvas (nome e chave Gemini)
        const config: any = await invoke("carregar_configuracoes");
        if (config) {
            if (userName) userName.value = config.user_name || "";
            if (geminiApiKey) geminiApiKey.value = config.gemini_api_key || "";
        }

        // Se ainda não tiver nome preenchido, busca do sistema
        if (userName && !userName.value) {
            const info: any = await invoke("get_system_info");
            userName.value = info.name;
        }

        if (folderPath) {
            const info: any = await invoke("get_system_info");
            folderPath.value = info.downloads;
        }
        
        // 2. Verifica se o Autostart já está ativado no sistema e marca a caixinha
        const autostartAtivo = await isEnabled();
        if (checkAutostart) checkAutostart.checked = autostartAtivo;

    } catch (e) {
        console.error("Erro ao carregar dados:", e);
        if (userName) userName.placeholder = "Erro de conexão";
    }
}

// Intercepta links externos para abrir no navegador padrão com o plugin-opener
document.querySelectorAll('a[href^="http"]').forEach(link => {
    link.addEventListener('click', (e) => {
        e.preventDefault();
        const url = (link as HTMLAnchorElement).href;
        openUrl(url).catch(() => window.open(url, '_blank'));
    });
});

// Executa imediatamente
carregarDadosDoPC();

if (btnSalvar) {
  btnSalvar.addEventListener("click", async () => {
    // Salva as configurações (nome e chave Gemini) no Rust
    try {
        await invoke("salvar_configuracoes", {
            config: {
                user_name: userName?.value?.trim() || "Usuário",
                gemini_api_key: geminiApiKey?.value?.trim() || ""
            }
        });
    } catch (error) {
        console.error("Erro ao salvar configurações:", error);
    }
    
    // Aplica a configuração do Autostart escolhida pelo usuário
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