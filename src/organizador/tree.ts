import { invoke } from "@tauri-apps/api/core";
import type { RegrasOrganizador } from "../types";

let regrasAtuais: RegrasOrganizador | null = null;
let idiomaAtual: string = "pt";

/**
 * Inicializa o componente de árvore do organizador de arquivos
 */
export async function inicializarArvoreOrganizador(idioma: string = "pt") {
    idiomaAtual = idioma;
    const container = document.getElementById("organizadorArvoreContainer");
    if (!container) return;

    try {
        regrasAtuais = await invoke<RegrasOrganizador>("obter_regras_organizador");
        renderizarArvore();
        configurarBotoesGlobais();
    } catch (e) {
        console.error("Erro ao carregar regras do organizador:", e);
        container.innerHTML = `<div class="aero-error">${idiomaAtual === "en" ? "Failed to load organization rules." : "Erro ao carregar regras do organizador."}</div>`;
    }
}

/**
 * Atualiza o idioma do componente de árvore
 */
export function atualizarIdiomaArvore(idioma: string) {
    idiomaAtual = idioma;
    if (regrasAtuais) {
        renderizarArvore();
    }
}

/**
 * Renderiza todos os nós e pastas de regras na árvore
 */
function renderizarArvore() {
    const container = document.getElementById("organizadorArvoreContainer");
    if (!container || !regrasAtuais) return;

    const isEn = idiomaAtual === "en";

    container.innerHTML = "";

    regrasAtuais.regras.forEach((regra) => {
        const pastaNode = document.createElement("div");
        pastaNode.className = "aero-tree-node";
        pastaNode.id = `regra-${regra.id}`;

        // Cabeçalho da pasta
        const header = document.createElement("div");
        header.className = "aero-tree-header";

        const icon = document.createElement("span");
        icon.className = "aero-folder-icon";
        icon.innerHTML = `<svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path></svg>`;

        const title = document.createElement("span");
        title.className = "aero-folder-name";
        title.textContent = regra.nome;

        const badgeContador = document.createElement("span");
        badgeContador.className = "aero-badge-count";
        badgeContador.textContent = `${regra.extensoes.length} ${isEn ? "ext" : "ext"}`;

        header.appendChild(icon);
        header.appendChild(title);
        header.appendChild(badgeContador);

        // Corpo com os chips de extensões
        const body = document.createElement("div");
        body.className = "aero-tree-body";

        const chipsContainer = document.createElement("div");
        chipsContainer.className = "aero-chips-container";

        regra.extensoes.forEach((ext) => {
            const chip = document.createElement("span");
            chip.className = "aero-chip";
            chip.innerHTML = `.${ext} <button type="button" class="btn-remove-chip" title="${isEn ? "Remove" : "Remover"}" aria-label="${isEn ? "Remove extension" : "Remover extensão"} ${ext}">×</button>`;

            chip.querySelector(".btn-remove-chip")?.addEventListener("click", () => {
                removerExtensao(regra.id, ext);
            });

            chipsContainer.appendChild(chip);
        });

        // Formulário rápido para adicionar nova extensão
        const addContainer = document.createElement("div");
        addContainer.className = "aero-add-chip-row";

        const inputAdd = document.createElement("input");
        inputAdd.type = "text";
        inputAdd.className = "aero-input-chip";
        inputAdd.placeholder = isEn ? "Add ext (e.g. zip)..." : "Nova ext (ex: zip)...";
        inputAdd.maxLength = 15;

        const btnAdd = document.createElement("button");
        btnAdd.type = "button";
        btnAdd.className = "btn-add-chip";
        btnAdd.textContent = "+";
        btnAdd.title = isEn ? "Add extension" : "Adicionar extensão";

        const executarAdicao = () => {
            const val = inputAdd.value.trim().toLowerCase().replace(/^\./, "");
            if (val) {
                adicionarExtensao(regra.id, val);
                inputAdd.value = "";
            }
        };

        inputAdd.addEventListener("keydown", (ev) => {
            if (ev.key === "Enter") {
                ev.preventDefault();
                executarAdicao();
            }
        });

        btnAdd.addEventListener("click", executarAdicao);

        addContainer.appendChild(inputAdd);
        addContainer.appendChild(btnAdd);

        body.appendChild(chipsContainer);
        body.appendChild(addContainer);

        pastaNode.appendChild(header);
        pastaNode.appendChild(body);
        container.appendChild(pastaNode);
    });
}

/**
 * Remove uma extensão de uma categoria de regra
 */
function removerExtensao(regraId: string, extParaRemover: string) {
    if (!regrasAtuais) return;
    const regra = regrasAtuais.regras.find((r) => r.id === regraId);
    if (!regra) return;

    regra.extensoes = regra.extensoes.filter(
        (e) => e.toLowerCase() !== extParaRemover.toLowerCase()
    );
    renderizarArvore();
    destacarAlteracaoNaoSalva();
}

/**
 * Adiciona uma nova extensão a uma regra
 */
function adicionarExtensao(regraId: string, novaExt: string) {
    if (!regrasAtuais) return;
    const regra = regrasAtuais.regras.find((r) => r.id === regraId);
    if (!regra) return;

    const extLimpa = novaExt.trim().toLowerCase().replace(/^\./, "");
    if (!extLimpa) return;

    if (regra.extensoes.some((e) => e.toLowerCase() === extLimpa)) {
        return; // Já existe
    }

    regra.extensoes.push(extLimpa);
    renderizarArvore();
    destacarAlteracaoNaoSalva();
}

/**
 * Configura os botões de ação global (Salvar e Restaurar)
 */
function configurarBotoesGlobais() {
    const btnSalvar = document.getElementById("btnSalvarRegrasOrganizador");
    const btnRestaurar = document.getElementById("btnRestaurarRegrasOrganizador");

    btnSalvar?.addEventListener("click", async () => {
        if (!regrasAtuais) return;
        const isEn = idiomaAtual === "en";
        const statusEl = document.getElementById("statusRegrasOrganizador");

        try {
            await invoke("salvar_regras_organizador", { regras: regrasAtuais });
            if (statusEl) {
                statusEl.textContent = isEn ? "Rules saved successfully!" : "Regras salvas com sucesso!";
                statusEl.className = "aero-status-msg aero-status-success";
                statusEl.style.display = "block";
                setTimeout(() => {
                    statusEl.style.display = "none";
                }, 3000);
            }
        } catch (e) {
            console.error("Erro ao salvar regras:", e);
            if (statusEl) {
                statusEl.textContent = isEn ? "Error saving rules." : "Erro ao salvar regras.";
                statusEl.className = "aero-status-msg aero-status-error";
                statusEl.style.display = "block";
            }
        }
    });

    btnRestaurar?.addEventListener("click", async () => {
        const isEn = idiomaAtual === "en";
        const confirmMsg = isEn
            ? "Restore default organization rules?"
            : "Deseja restaurar as regras de organização de fábrica?";

        if (!window.confirm(confirmMsg)) return;

        try {
            regrasAtuais = await invoke<RegrasOrganizador>("restaurar_regras_padrao");
            renderizarArvore();
            const statusEl = document.getElementById("statusRegrasOrganizador");
            if (statusEl) {
                statusEl.textContent = isEn ? "Default rules restored!" : "Regras padrão restauradas!";
                statusEl.className = "aero-status-msg aero-status-success";
                statusEl.style.display = "block";
                setTimeout(() => {
                    statusEl.style.display = "none";
                }, 3000);
            }
        } catch (e) {
            console.error("Erro ao restaurar regras:", e);
        }
    });
}

function destacarAlteracaoNaoSalva() {
    const statusEl = document.getElementById("statusRegrasOrganizador");
    if (statusEl) {
        statusEl.textContent = idiomaAtual === "en" ? "Changes not saved yet." : "Modificações ainda não salvas.";
        statusEl.className = "aero-status-msg aero-status-pending";
        statusEl.style.display = "block";
    }
}
