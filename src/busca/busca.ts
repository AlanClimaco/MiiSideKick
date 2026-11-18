import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { AppConfig, ResultadoBusca } from "../types";

const inputBusca = document.getElementById("inputBusca") as HTMLInputElement | null;
const btnBuscar = document.getElementById("btnBuscar") as HTMLButtonElement | null;
const btnFechar = document.getElementById("btnFechar") as HTMLButtonElement | null;
const buscaList = document.getElementById("buscaList") as HTMLDivElement | null;
const buscaEmpty = document.getElementById("buscaEmpty") as HTMLDivElement | null;
const emptyMessage = document.getElementById("emptyMessage") as HTMLParagraphElement | null;
const buscaLoading = document.getElementById("buscaLoading") as HTMLDivElement | null;
const loadingMessage = document.getElementById("loadingMessage") as HTMLSpanElement | null;
const buscaStatus = document.getElementById("buscaStatus") as HTMLSpanElement | null;
const checkFecharAoSair = document.getElementById("checkFecharAoSair") as HTMLInputElement | null;
const buscaTitle = document.getElementById("buscaTitle") as HTMLHeadingElement | null;
const buscaSubtitle = document.getElementById("buscaSubtitle") as HTMLSpanElement | null;
const labelFecharAoSair = document.getElementById("labelFecharAoSair") as HTMLSpanElement | null;

let isEn = false;

const TEXTS = {
    pt: {
        title: "Busca de Arquivos",
        subtitle: "Downloads, Documentos e Desktop",
        placeholder: "Digite o nome do arquivo...",
        statusReady: "Pronto para buscar",
        statusSearching: "Procurando arquivos...",
        statusFound: (n: number) => `${n} ${n === 1 ? 'arquivo encontrado' : 'arquivos encontrados'}`,
        statusNotFound: "Nenhum arquivo encontrado",
        emptyInitial: "Digite o nome ou extensão para localizar arquivos.",
        emptyNotFound: (term: string) => `Nenhum arquivo encontrado para "${term}".`,
        closeOnOpen: "Fechar ao abrir",
        searchBtnTitle: "Buscar",
        closeBtnTitle: "Fechar",
        openFolderTitle: "Clique para abrir no Explorador de Arquivos",
    },
    en: {
        title: "File Search",
        subtitle: "Downloads, Documents and Desktop",
        placeholder: "Type file name...",
        statusReady: "Ready to search",
        statusSearching: "Searching files...",
        statusFound: (n: number) => `${n} ${n === 1 ? 'file found' : 'files found'}`,
        statusNotFound: "No files found",
        emptyInitial: "Type a name or extension to find files.",
        emptyNotFound: (term: string) => `No files found for "${term}".`,
        closeOnOpen: "Close on open",
        searchBtnTitle: "Search",
        closeBtnTitle: "Close",
        openFolderTitle: "Click to open in File Explorer",
    },
};

async function atualizarIdioma() {
    try {
        const config = await invoke<AppConfig>("carregar_configuracoes");
        isEn = config && config.idioma === "en";
    } catch {
        isEn = false;
    }

    const t = isEn ? TEXTS.en : TEXTS.pt;
    if (buscaTitle) buscaTitle.textContent = t.title;
    if (buscaSubtitle) buscaSubtitle.textContent = t.subtitle;
    if (inputBusca) inputBusca.placeholder = t.placeholder;
    if (labelFecharAoSair) labelFecharAoSair.textContent = t.closeOnOpen;
    if (btnBuscar) {
        btnBuscar.title = t.searchBtnTitle;
        btnBuscar.setAttribute("aria-label", t.searchBtnTitle);
    }
    if (btnFechar) {
        btnFechar.title = t.closeBtnTitle;
        btnFechar.setAttribute("aria-label", t.closeBtnTitle);
    }
    if (loadingMessage) loadingMessage.textContent = t.statusSearching;
}

async function fecharJanela() {
    try {
        await invoke("esconder_busca");
    } catch (e) {
        console.error("Erro ao fechar busca:", e);
    }
}

async function executarBusca() {
    const termo = (inputBusca?.value || "").trim();
    const t = isEn ? TEXTS.en : TEXTS.pt;

    if (!termo) {
        if (buscaList) buscaList.innerHTML = "";
        if (buscaEmpty) {
            buscaEmpty.style.display = "flex";
            if (emptyMessage) emptyMessage.textContent = t.emptyInitial;
        }
        if (buscaStatus) buscaStatus.textContent = t.statusReady;
        return;
    }

    if (buscaList) buscaList.style.display = "none";
    if (buscaEmpty) buscaEmpty.style.display = "none";
    if (buscaLoading) buscaLoading.style.display = "flex";
    if (buscaStatus) buscaStatus.textContent = t.statusSearching;

    try {
        const resultados = await invoke<ResultadoBusca[]>("procurar_arquivos_locais", { termo });

        if (buscaLoading) buscaLoading.style.display = "none";

        if (!resultados || resultados.length === 0) {
            if (buscaList) {
                buscaList.innerHTML = "";
                buscaList.style.display = "none";
            }
            if (buscaEmpty) {
                buscaEmpty.style.display = "flex";
                if (emptyMessage) emptyMessage.textContent = t.emptyNotFound(termo);
            }
            if (buscaStatus) buscaStatus.textContent = t.statusNotFound;
            return;
        }

        if (buscaEmpty) buscaEmpty.style.display = "none";
        if (buscaStatus) buscaStatus.textContent = t.statusFound(resultados.length);

        if (!buscaList) return;
        buscaList.innerHTML = "";
        buscaList.style.display = "flex";

        resultados.forEach((item) => {
            const card = document.createElement("div");
            card.className = "resultado-card";
            card.title = `${t.openFolderTitle}: ${item.caminho}`;

            // Header com Ícone e Nome do Arquivo
            const header = document.createElement("div");
            header.className = "resultado-header";

            const iconSvg = document.createElementNS("http://www.w3.org/2000/svg", "svg");
            iconSvg.setAttribute("class", "resultado-icon");
            iconSvg.setAttribute("viewBox", "0 0 24 24");
            iconSvg.setAttribute("fill", "none");
            iconSvg.setAttribute("stroke", "currentColor");
            iconSvg.setAttribute("stroke-width", "2");
            iconSvg.setAttribute("stroke-linecap", "round");
            iconSvg.setAttribute("stroke-linejoin", "round");

            const path1 = document.createElementNS("http://www.w3.org/2000/svg", "path");
            path1.setAttribute("d", "M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z");
            const poly = document.createElementNS("http://www.w3.org/2000/svg", "polyline");
            poly.setAttribute("points", "14 2 14 8 20 8");
            iconSvg.appendChild(path1);
            iconSvg.appendChild(poly);

            const nomeSpan = document.createElement("span");
            nomeSpan.className = "resultado-nome";
            nomeSpan.textContent = item.nome;

            header.appendChild(iconSvg);
            header.appendChild(nomeSpan);
            card.appendChild(header);

            // Caminho da Pasta
            const caminhoSpan = document.createElement("div");
            caminhoSpan.className = "resultado-caminho";
            caminhoSpan.textContent = item.caminho;
            card.appendChild(caminhoSpan);

            // Metadados: Tamanho e Data
            const metaDiv = document.createElement("div");
            metaDiv.className = "resultado-meta";

            const tamanhoSpan = document.createElement("span");
            tamanhoSpan.className = "resultado-tamanho";
            tamanhoSpan.textContent = item.tamanho_formatado;

            const dataSpan = document.createElement("span");
            dataSpan.className = "resultado-data";
            dataSpan.textContent = item.data_modificacao;

            metaDiv.appendChild(tamanhoSpan);
            metaDiv.appendChild(dataSpan);
            card.appendChild(metaDiv);

            // Clique: Abre pasta com o arquivo focado
            card.addEventListener("click", async () => {
                try {
                    await invoke("abrir_pasta_com_arquivo", { caminho: item.caminho });
                    if (checkFecharAoSair?.checked) {
                        await fecharJanela();
                    }
                } catch (err) {
                    console.error("Erro ao abrir pasta com arquivo:", err);
                }
            });

            buscaList.appendChild(card);
        });
    } catch (e) {
        console.error("Erro ao procurar arquivos locais:", e);
        if (buscaLoading) buscaLoading.style.display = "none";
        if (buscaEmpty) {
            buscaEmpty.style.display = "flex";
            if (emptyMessage) emptyMessage.textContent = isEn ? "Error searching files." : "Erro ao pesquisar arquivos.";
        }
    }
}

// Botão Buscar
btnBuscar?.addEventListener("click", () => {
    executarBusca();
});

// Tecla Enter no Campo de Busca
inputBusca?.addEventListener("keydown", (e: KeyboardEvent) => {
    if (e.key === "Enter") {
        e.preventDefault();
        executarBusca();
    }
});

// Botão Fechar (X)
btnFechar?.addEventListener("click", () => {
    fecharJanela();
});

// Tecla Escape para Fechar
window.addEventListener("keydown", (e: KeyboardEvent) => {
    if (e.key === "Escape") {
        e.preventDefault();
        fecharJanela();
    }
});

async function aplicarTermoEBuscar(termo: string) {
    if (!termo || !termo.trim()) return;
    if (inputBusca) {
        inputBusca.value = termo.trim();
    }
    await executarBusca();
}

async function checarTermoPrePesquisa() {
    try {
        const termo = await invoke<string | null>("obter_termo_pre_pesquisa");
        if (termo && termo.trim()) {
            await aplicarTermoEBuscar(termo);
        }
    } catch (e) {
        console.error("Erro ao obter termo pré-pesquisa:", e);
    }
}

// Escuta evento emitido pelo backend com termo de busca
listen<string>("termo_busca", async (event) => {
    if (event.payload && typeof event.payload === "string") {
        await aplicarTermoEBuscar(event.payload);
    }
});

// Atualiza e foca ao carregar e ganhar foco
window.addEventListener("focus", () => {
    atualizarIdioma();
    checarTermoPrePesquisa();
    if (inputBusca) {
        inputBusca.focus();
        inputBusca.select();
    }
});

window.addEventListener("DOMContentLoaded", () => {
    atualizarIdioma();
    checarTermoPrePesquisa();
    if (inputBusca) {
        inputBusca.focus();
    }
});

atualizarIdioma();
checarTermoPrePesquisa();
if (inputBusca) {
    inputBusca.focus();
}
