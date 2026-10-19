import { invoke } from "@tauri-apps/api/core";
import { enable, disable, isEnabled } from '@tauri-apps/plugin-autostart';
import { openUrl } from '@tauri-apps/plugin-opener';

const userName = document.getElementById("userName") as HTMLInputElement;
const geminiApiKey = document.getElementById("geminiApiKey") as HTMLInputElement;
const btnToggleApiKey = document.getElementById("btnToggleApiKey") as HTMLButtonElement;
const folderPath = document.getElementById("folderPath") as HTMLInputElement;
const btnSalvar = document.getElementById("btnSalvar");
const checkAutostart = document.getElementById("checkAutostart") as HTMLInputElement;
const personalityButtons = document.querySelectorAll<HTMLButtonElement>(".personality-btn");
const personalityHint = document.getElementById("personalityHint");
const inputLimite = document.getElementById("limiteCaracteres") as HTMLInputElement;
const lengthButtons = document.querySelectorAll<HTMLButtonElement>(".length-btn");

// Armazena a personalidade selecionada
let personalidadeAtual = "espelho";

// Legendas explicativas para cada personalidade
const DESC_PERSONALIDADES: Record<string, string> = {
    espelho: "Adapta o tom ao usuário alternando estritamente entre Educado, Engraçado e Sério/Direto.",
    educado: "Extremamente educado, formal, paciente e gentil, como um mordomo atencioso.",
    engracado: "Engraçado e bem-humorado. Piadinhas leves, trocadilhos e respostas descontraídas.",
    direto: "100% sério, técnico, direto ao ponto e objetivo, sem enrolações.",
    hostil: "Agressivo, boca-suja, sem paciência e estressado. Sem papas na língua e sem filtros.",
    manuel_gomes: "Responde como o cantor Manuel Gomes (Caneta Azul), com sabedoria do interior e canções.",
    sarcastico: "Agressivo, boca-suja, sem paciência e estressado. Sem papas na língua e sem filtros.",
};

function selecionarPersonalidade(personalidade: string) {
    let persona = personalidade || "espelho";
    if (persona === "sarcastico") {
        persona = "hostil";
    }
    personalidadeAtual = persona;

    personalityButtons.forEach((btn) => {
        if (btn.getAttribute("data-personality") === personalidadeAtual) {
            btn.classList.add("active");
        } else {
            btn.classList.remove("active");
        }
    });

    if (personalityHint && DESC_PERSONALIDADES[personalidadeAtual]) {
        personalityHint.textContent = DESC_PERSONALIDADES[personalidadeAtual];
    }
}

// Configura listeners de clique nos botões de personalidade
personalityButtons.forEach((btn) => {
    btn.addEventListener("click", () => {
        const personality = btn.getAttribute("data-personality") || "espelho";
        selecionarPersonalidade(personality);
    });
});

// Ícones SVG para o botão de visualização da chave de API
const ICON_EYE = `<svg viewBox="0 0 24 24" width="16" height="16" stroke="currentColor" stroke-width="2" fill="none" stroke-linecap="round" stroke-linejoin="round"><path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"></path><circle cx="12" cy="12" r="3"></circle></svg>`;
const ICON_EYE_OFF = `<svg viewBox="0 0 24 24" width="16" height="16" stroke="currentColor" stroke-width="2" fill="none" stroke-linecap="round" stroke-linejoin="round"><path d="M17.94 17.94A10.07 10.07 0 0 1 12 20c-7 0-11-8-11-8a18.45 18.45 0 0 1 5.06-5.94M9.9 4.24A9.12 9.12 0 0 1 12 4c7 0 11 8 11 8a18.5 18.5 0 0 1-2.16 3.19m-6.72-1.07a3 3 0 1 1-4.24-4.24"></path><line x1="1" y1="1" x2="23" y2="23"></line></svg>`;

// Alterna visibilidade da chave de API
if (btnToggleApiKey && geminiApiKey) {
    btnToggleApiKey.innerHTML = ICON_EYE;
    btnToggleApiKey.addEventListener("click", () => {
        if (geminiApiKey.type === "password") {
            geminiApiKey.type = "text";
            btnToggleApiKey.innerHTML = ICON_EYE_OFF;
        } else {
            geminiApiKey.type = "password";
            btnToggleApiKey.innerHTML = ICON_EYE;
        }
    });
}

// Controle do limite de caracteres (30 a 500)
function atualizarLimiteVisual(valor: number, sincronizarInput: boolean = true) {
    const valorClamp = Math.max(30, Math.min(500, valor));
    if (sincronizarInput && inputLimite) {
        inputLimite.value = valorClamp.toString();
    }
    lengthButtons.forEach((btn) => {
        const btnValor = parseInt(btn.getAttribute("data-length") || "0", 10);
        if (btnValor === valorClamp) {
            btn.classList.add("active");
        } else {
            btn.classList.remove("active");
        }
    });
}

lengthButtons.forEach((btn) => {
    btn.addEventListener("click", () => {
        const valor = parseInt(btn.getAttribute("data-length") || "70", 10);
        atualizarLimiteVisual(valor, true);
    });
});

if (inputLimite) {
    inputLimite.addEventListener("input", () => {
        const valor = parseInt(inputLimite.value, 10);
        if (!isNaN(valor)) {
            // Se o valor corresponder exatamente a 70, 140 ou 250, ativa o botão; caso contrário, desativa todos sinalizando custom
            let encontrouPreset = false;
            lengthButtons.forEach((btn) => {
                const btnValor = parseInt(btn.getAttribute("data-length") || "0", 10);
                if (btnValor === valor) {
                    btn.classList.add("active");
                    encontrouPreset = true;
                } else {
                    btn.classList.remove("active");
                }
            });
            if (!encontrouPreset) {
                lengthButtons.forEach((btn) => btn.classList.remove("active"));
            }
        } else {
            lengthButtons.forEach((btn) => btn.classList.remove("active"));
        }
    });

    inputLimite.addEventListener("blur", () => {
        const valor = parseInt(inputLimite.value, 10) || 70;
        atualizarLimiteVisual(valor, true);
    });
}

async function carregarDadosDoPC() {
    try {
        // 1. Carrega as configurações salvas (nome, chave Gemini, personalidade e limite)
        const config: any = await invoke("carregar_configuracoes");
        if (config) {
            if (userName) userName.value = config.user_name || "";
            if (geminiApiKey) geminiApiKey.value = config.gemini_api_key || "";
            if (config.personalidade) {
                selecionarPersonalidade(config.personalidade);
            }
            if (config.limite_caracteres) {
                atualizarLimiteVisual(config.limite_caracteres);
            } else {
                atualizarLimiteVisual(70);
            }
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
        const limiteFinal = Math.max(30, Math.min(500, parseInt(inputLimite?.value || "70", 10) || 70));

        // Salva as configurações (nome, chave Gemini, personalidade e limite) no Rust
        try {
            await invoke("salvar_configuracoes", {
                config: {
                    user_name: userName?.value?.trim() || "Usuário",
                    gemini_api_key: geminiApiKey?.value?.trim() || "",
                    personalidade: personalidadeAtual,
                    limite_caracteres: limiteFinal
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