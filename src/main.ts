import { invoke } from "@tauri-apps/api/core";
import { enable, disable, isEnabled } from '@tauri-apps/plugin-autostart';
import { openUrl } from '@tauri-apps/plugin-opener';
import type { AppConfig, UserInfo } from "./types";

const FORMSPREE_ENDPOINT = "https://formspree.io/f/xvkkyvbv";

// Elementos da interface
const userName = document.getElementById("userName") as HTMLInputElement;
const userInterests = document.getElementById("userInterests") as HTMLInputElement;
const labelUserInterests = document.getElementById("labelUserInterests");
const hintUserInterests = document.getElementById("hintUserInterests");
const geminiApiKey = document.getElementById("geminiApiKey") as HTMLInputElement;
const btnToggleApiKey = document.getElementById("btnToggleApiKey") as HTMLButtonElement;
const folderPath = document.getElementById("folderPath") as HTMLInputElement;
const btnSalvar = document.getElementById("btnSalvar");
const checkAutostart = document.getElementById("checkAutostart") as HTMLInputElement;
const personalityButtons = document.querySelectorAll<HTMLButtonElement>(".personality-btn");
const personalityHint = document.getElementById("personalityHint");
const inputLimite = document.getElementById("limiteCaracteres") as HTMLInputElement;
const lengthButtons = document.querySelectorAll<HTMLButtonElement>(".length-btn");
const languageButtons = document.querySelectorAll<HTMLButtonElement>(".lang-btn");

// Controles de alertas proativos (2 categorias)
const checkAlertaManutencao = document.getElementById("checkAlertaManutencao") as HTMLInputElement;
const checkAlertaInteracaoBemEstar = document.getElementById("checkAlertaInteracaoBemEstar") as HTMLInputElement;
const frequencyButtons = document.querySelectorAll<HTMLButtonElement>(".frequency-btn");
const frequencyHint = document.getElementById("frequencyHint");

// Elementos de Atalhos e Feedback
const keyModifier = document.getElementById("keyModifier");
const feedbackForm = document.getElementById("feedbackForm") as HTMLFormElement | null;
const feedbackName = document.getElementById("feedbackName") as HTMLInputElement | null;
const feedbackType = document.getElementById("feedbackType") as HTMLSelectElement | null;
const feedbackMessage = document.getElementById("feedbackMessage") as HTMLTextAreaElement | null;
const btnEnviarFeedback = document.getElementById("btnEnviarFeedback") as HTMLButtonElement | null;
const feedbackStatus = document.getElementById("feedbackStatus") as HTMLDivElement | null;

// Elementos de Atualização
const btnCheckUpdates = document.getElementById("btnCheckUpdates") as HTMLButtonElement | null;
const labelCheckUpdates = document.getElementById("labelCheckUpdates");
const updateStatusMsg = document.getElementById("updateStatusMsg") as HTMLDivElement | null;

interface UpdateCheckResult {
    tem_atualizacao: boolean;
    versao_atual: string;
    versao_nova?: string;
    url_release?: string;
    mensagem: string;
}

// Configuração do modificador de tecla de acordo com a plataforma (Mac vs Windows/Linux)
const isMac = typeof navigator !== "undefined" && navigator.userAgent.toLowerCase().includes("mac");
if (keyModifier) {
    keyModifier.textContent = isMac ? "Cmd" : "Ctrl";
}

// Estado da aplicação
let idiomaAtual = "pt";
let personalidadeAtual = "espelho";
let frequenciaAtual = "equilibrado";

interface Translations {
    windowTitle: string;
    headerTitle: string;
    headerSubtitle: string;
    sectionLanguage: string;
    sectionProfile: string;
    labelUserName: string;
    placeholderUserName: string;
    placeholderUserNameError: string;
    labelUserInterests: string;
    placeholderUserInterests: string;
    hintUserInterests: string;
    sectionPersonality: string;
    personalityNames: Record<string, string>;
    personalityHints: Record<string, string>;
    sectionApiKey: string;
    labelApiKey: string;
    placeholderApiKey: string;
    btnToggleKeyTitle: string;
    hintApiKey: string;
    sectionLength: string;
    labelPresetsLength: string;
    presetShort: string;
    presetMedium: string;
    presetLong: string;
    labelCustomLength: string;
    placeholderCustomLength: string;
    unitCharacters: string;
    hintLength: string;
    sectionProactive: string;
    checkMaintenance: string;
    checkWellBeing: string;
    labelFrequency: string;
    freqDiscreet: string;
    freqBalanced: string;
    freqFrequent: string;
    frequencyHints: Record<string, string>;
    sectionStartup: string;
    checkAutostart: string;
    sectionShortcuts: string;
    shortcutChatLabel: string;
    shortcutChatDesc: string;
    sectionFeedback: string;
    hintFeedback: string;
    labelFeedbackName: string;
    placeholderFeedbackName: string;
    labelFeedbackType: string;
    optSuggestion: string;
    optBug: string;
    optPraise: string;
    labelFeedbackMessage: string;
    placeholderFeedbackMessage: string;
    btnSendFeedback: string;
    btnSendingFeedback: string;
    feedbackSuccess: string;
    feedbackError: string;
    btnCheckUpdates: string;
    btnCheckingUpdates: string;
    tutorialLabel: string;
    tutorialStep1Alt: string;
    tutorialStep2Alt: string;
    tutorialDesc: string;
    btnSave: string;
    footerDevBy: string;
    tabGeneral: string;
    tabAI: string;
    tabAlerts: string;
    tabTools: string;
}

const TRANSLATIONS: Record<string, Translations> = {
    pt: {
        windowTitle: "MiiSideKick — Configurações (v1.6.0)",
        headerTitle: "Configurações",
        headerSubtitle: "Personalize seu assistente local.",
        sectionLanguage: "Idioma / Language",
        sectionProfile: "Perfil",
        labelUserName: "Como devo te chamar?",
        placeholderUserName: "Lendo do sistema...",
        placeholderUserNameError: "Erro de conexão",
        labelUserInterests: "Seus interesses e gostos (opcional)",
        placeholderUserInterests: "Ex: anime, xadrez, café, programação...",
        hintUserInterests: "O Mii pode usar seus gostos de vez em quando para fazer analogias ou puxar assunto de forma sutil.",
        sectionPersonality: "Personalidade",
        personalityNames: {
            espelho: "Espelho",
            educado: "Educado",
            engracado: "Engraçado",
            direto: "Sério & Direto",
            hostil: "Hostil",
        },
        personalityHints: {
            espelho: "Adapta o tom ao usuário alternando estritamente entre Educado, Engraçado e Sério/Direto.",
            educado: "Extremamente educado, formal, paciente e gentil, como um mordomo atencioso.",
            engracado: "Engraçado e bem-humorado. Piadinhas leves, trocadilhos e respostas descontraídas.",
            direto: "100% sério, técnico, direto ao ponto e objetivo, sem enrolações.",
            hostil: "Abertamente agressivo, boca-suja, estressado e sem paciência.",
            sarcastico: "Abertamente agressivo, boca-suja, estressado e sem paciência.",
        },
        sectionApiKey: "Chave de API do Gemini",
        labelApiKey: "Chave de acesso",
        placeholderApiKey: "Cole sua chave AIzaSy...",
        btnToggleKeyTitle: "Mostrar ou ocultar chave",
        hintApiKey: `Para conversar com o Mii, pegue sua chave gratuita no <a href="https://aistudio.google.com/app/apikey" target="_blank" rel="noreferrer">Google AI Studio</a>. Basta fazer login com sua conta Google e clicar em "Create API key".`,
        sectionLength: "Tamanho da Resposta",
        labelPresetsLength: "Tamanhos pré-definidos:",
        presetShort: "70 (Curto)",
        presetMedium: "140 (Médio)",
        presetLong: "250 (Longo)",
        labelCustomLength: "Ou defina um tamanho personalizado (30 a 500 caracteres):",
        placeholderCustomLength: "Ex: 180",
        unitCharacters: "caracteres",
        hintLength: "Limite máximo de caracteres que o Mii usará nas respostas.",
        sectionProactive: "Alertas Proativos",
        checkMaintenance: "<strong>Manutenção do Sistema</strong> (Lixeira cheia e acúmulo na pasta Downloads)",
        checkWellBeing: "<strong>Interação & Bem-Estar</strong> (Lembretes de saúde/água/postura e curiosidades de IA)",
        labelFrequency: "Frequência dos alertas:",
        freqDiscreet: "Discreto",
        freqBalanced: "Equilibrado",
        freqFrequent: "Frequente",
        frequencyHints: {
            discreto: "Intervalos maiores e alertas mais raros. Foco máximo nas suas tarefas.",
            equilibrado: "Intervalos balanceados e lembretes amigáveis ao longo do dia.",
            frequente: "Companhia mais ativa, com curiosidades e dicas com maior frequência.",
        },
        sectionStartup: "Inicialização do Sistema",
        checkAutostart: "Iniciar junto com o Windows",
        sectionShortcuts: "Atalhos do Teclado",
        shortcutChatLabel: "Conversa Rápida com o Mii",
        shortcutChatDesc: "Abre a caixa de diálogo instantânea a partir de qualquer aplicativo.",
        sectionFeedback: "Enviar Feedback",
        hintFeedback: "Tem uma sugestão, encontrou um problema ou quer elogiar o projeto? Deixe sua mensagem!",
        labelFeedbackName: "Seu Nome / Apelido (opcional)",
        placeholderFeedbackName: "Ex: Alex",
        labelFeedbackType: "Tipo de Feedback",
        optSuggestion: "Sugestão",
        optBug: "Problema / Bug",
        optPraise: "Elogio",
        labelFeedbackMessage: "Mensagem",
        placeholderFeedbackMessage: "Descreva sua mensagem aqui...",
        btnSendFeedback: "Enviar Feedback",
        btnSendingFeedback: "Enviando...",
        feedbackSuccess: "Feedback enviado com sucesso! ✓",
        feedbackError: "Não foi possível enviar o feedback. Tente novamente mais tarde.",
        btnCheckUpdates: "Verificar Atualizações",
        btnCheckingUpdates: "Verificando...",
        tutorialLabel: "Onde me encontrar depois de salvar?",
        tutorialStep1Alt: "Passo 1: encontre o Mii na bandeja do sistema",
        tutorialStep2Alt: "Passo 2: abra o menu de comandos do Mii",
        tutorialDesc: "O aplicativo ficará ativo na bandeja do sistema.<br>Clique com o botão direito no ícone da bandeja para acessar os comandos.",
        btnSave: "Salvar e Ocultar",
        footerDevBy: "Desenvolvido por",
        tabGeneral: "Geral & Perfil",
        tabAI: "Personalidade & IA",
        tabAlerts: "Alertas & Sistema",
        tabTools: "Ferramentas & Feedback",
    },
    en: {
        windowTitle: "MiiSideKick — Settings (v1.6.0)",
        headerTitle: "Settings",
        headerSubtitle: "Customize your desktop companion.",
        sectionLanguage: "Language / Idioma",
        sectionProfile: "Profile",
        labelUserName: "What should I call you?",
        placeholderUserName: "Reading from system...",
        placeholderUserNameError: "Connection error",
        labelUserInterests: "Your interests and hobbies (optional)",
        placeholderUserInterests: "E.g. anime, chess, coffee, coding...",
        hintUserInterests: "Mii might occasionally bring up your interests in subtle analogies or casual conversation.",
        sectionPersonality: "Personality",
        personalityNames: {
            espelho: "Mirror",
            educado: "Polite",
            engracado: "Funny",
            direto: "Serious & Direct",
            hostil: "Hostile",
        },
        personalityHints: {
            espelho: "Adapts the tone to the user strictly alternating between Polite, Funny, and Serious/Direct.",
            educado: "Extremely polite, formal, patient, and courteous, like an attentive butler.",
            engracado: "Funny and witty. Lighthearted jokes, puns, and playful responses.",
            direto: "100% serious, technical, straight to the point, and objective.",
            hostil: "Openly aggressive, foul-mouthed, stressed out, and impatient.",
            sarcastico: "Openly aggressive, foul-mouthed, stressed out, and impatient.",
        },
        sectionApiKey: "Gemini API Key",
        labelApiKey: "Access key",
        placeholderApiKey: "Paste your AIzaSy... key",
        btnToggleKeyTitle: "Show or hide key",
        hintApiKey: `To chat with Mii, get your free key at <a href="https://aistudio.google.com/app/apikey" target="_blank" rel="noreferrer">Google AI Studio</a>. Simply log in with your Google account and click "Create API key".`,
        sectionLength: "Response Length",
        labelPresetsLength: "Preset lengths:",
        presetShort: "70 (Short)",
        presetMedium: "140 (Medium)",
        presetLong: "250 (Long)",
        labelCustomLength: "Or set a custom length (30 to 500 characters):",
        placeholderCustomLength: "E.g. 180",
        unitCharacters: "characters",
        hintLength: "Maximum character limit Mii will use in responses.",
        sectionProactive: "Proactive Alerts",
        checkMaintenance: "<strong>System Maintenance</strong> (Full Recycle Bin and Downloads accumulation)",
        checkWellBeing: "<strong>Interaction & Well-Being</strong> (Health/water/posture reminders and AI trivia)",
        labelFrequency: "Alert frequency:",
        freqDiscreet: "Discreet",
        freqBalanced: "Balanced",
        freqFrequent: "Frequent",
        frequencyHints: {
            discreto: "Longer intervals and fewer alerts. Maximum focus on your tasks.",
            equilibrado: "Balanced intervals and friendly reminders throughout the day.",
            frequente: "More active companion, with more frequent trivia and tips.",
        },
        sectionStartup: "System Startup",
        checkAutostart: "Start with Windows",
        sectionShortcuts: "Keyboard Shortcuts",
        shortcutChatLabel: "Quick Chat with Mii",
        shortcutChatDesc: "Opens the instant chat dialog from any application.",
        sectionFeedback: "Send Feedback",
        hintFeedback: "Have a suggestion, found a bug, or want to praise the project? Leave your message!",
        labelFeedbackName: "Your Name / Nickname (optional)",
        placeholderFeedbackName: "E.g. Alex",
        labelFeedbackType: "Feedback Type",
        optSuggestion: "Suggestion",
        optBug: "Bug / Issue",
        optPraise: "Praise",
        labelFeedbackMessage: "Message",
        placeholderFeedbackMessage: "Describe your message here...",
        btnSendFeedback: "Send Feedback",
        btnSendingFeedback: "Sending...",
        feedbackSuccess: "Feedback sent successfully! ✓",
        feedbackError: "Could not send feedback. Please try again later.",
        btnCheckUpdates: "Check for Updates",
        btnCheckingUpdates: "Checking...",
        tutorialLabel: "Where to find me after saving?",
        tutorialStep1Alt: "Step 1: find Mii in the system tray",
        tutorialStep2Alt: "Step 2: open Mii's command menu",
        tutorialDesc: "The application will stay active in the system tray.<br>Right-click the tray icon to access commands.",
        btnSave: "Save and Hide",
        footerDevBy: "Developed by",
        tabGeneral: "General & Profile",
        tabAI: "Personality & AI",
        tabAlerts: "Alerts & System",
        tabTools: "Tools & Feedback",
    },
};

function selecionarIdioma(idioma: string) {
    const lang = (idioma || "pt").toLowerCase().trim() === "en" ? "en" : "pt";
    idiomaAtual = lang;

    // Atualiza botões do seletor
    languageButtons.forEach((btn) => {
        if (btn.getAttribute("data-lang") === idiomaAtual) {
            btn.classList.add("active");
        } else {
            btn.classList.remove("active");
        }
    });

    const t = TRANSLATIONS[idiomaAtual];
    if (!t) return;

    // Atualiza título da página e atributo lang
    document.title = t.windowTitle;
    document.documentElement.lang = idiomaAtual === "en" ? "en" : "pt-BR";

    // Cabeçalho
    const headerTitle = document.getElementById("headerTitle");
    if (headerTitle) headerTitle.textContent = t.headerTitle;

    const headerSubtitle = document.getElementById("headerSubtitle");
    if (headerSubtitle) headerSubtitle.textContent = t.headerSubtitle;

    // Abas de Configuração
    const tabBtnGeral = document.getElementById("tabBtnGeral");
    if (tabBtnGeral) tabBtnGeral.textContent = t.tabGeneral;
    const tabBtnIA = document.getElementById("tabBtnIA");
    if (tabBtnIA) tabBtnIA.textContent = t.tabAI;
    const tabBtnSistema = document.getElementById("tabBtnSistema");
    if (tabBtnSistema) tabBtnSistema.textContent = t.tabAlerts;
    const tabBtnFerramentas = document.getElementById("tabBtnFerramentas");
    if (tabBtnFerramentas) tabBtnFerramentas.textContent = t.tabTools;

    // Seção Idioma
    const sectionLanguageTitle = document.getElementById("sectionLanguageTitle");
    if (sectionLanguageTitle) sectionLanguageTitle.textContent = t.sectionLanguage;

    // Seção Perfil
    const sectionProfileTitle = document.getElementById("sectionProfileTitle");
    if (sectionProfileTitle) sectionProfileTitle.textContent = t.sectionProfile;

    const labelUserName = document.getElementById("labelUserName");
    if (labelUserName) labelUserName.textContent = t.labelUserName;

    if (userName && (!userName.value || userName.placeholder === TRANSLATIONS.pt.placeholderUserName || userName.placeholder === TRANSLATIONS.en.placeholderUserName)) {
        userName.placeholder = t.placeholderUserName;
    }

    if (labelUserInterests) labelUserInterests.textContent = t.labelUserInterests;
    if (userInterests) userInterests.placeholder = t.placeholderUserInterests;
    if (hintUserInterests) hintUserInterests.textContent = t.hintUserInterests;

    // Seção Personalidade
    const sectionPersonalityTitle = document.getElementById("sectionPersonalityTitle");
    if (sectionPersonalityTitle) sectionPersonalityTitle.textContent = t.sectionPersonality;

    personalityButtons.forEach((btn) => {
        const pKey = btn.getAttribute("data-personality");
        if (pKey && t.personalityNames[pKey]) {
            btn.textContent = t.personalityNames[pKey];
        }
    });

    if (personalityHint && t.personalityHints[personalidadeAtual]) {
        personalityHint.textContent = t.personalityHints[personalidadeAtual];
    }

    // Seção Chave de API
    const sectionApiKeyTitle = document.getElementById("sectionApiKeyTitle");
    if (sectionApiKeyTitle) sectionApiKeyTitle.textContent = t.sectionApiKey;

    const labelApiKey = document.getElementById("labelApiKey");
    if (labelApiKey) labelApiKey.textContent = t.labelApiKey;

    if (geminiApiKey) geminiApiKey.placeholder = t.placeholderApiKey;

    if (btnToggleApiKey) {
        btnToggleApiKey.title = t.btnToggleKeyTitle;
        btnToggleApiKey.setAttribute("aria-label", t.btnToggleKeyTitle);
    }

    const hintApiKey = document.getElementById("hintApiKey");
    if (hintApiKey) hintApiKey.innerHTML = t.hintApiKey;

    // Seção Tamanho da Resposta
    const sectionLengthTitle = document.getElementById("sectionLengthTitle");
    if (sectionLengthTitle) sectionLengthTitle.textContent = t.sectionLength;

    const labelPresetsLength = document.getElementById("labelPresetsLength");
    if (labelPresetsLength) labelPresetsLength.textContent = t.labelPresetsLength;

    lengthButtons.forEach((btn) => {
        const len = btn.getAttribute("data-length");
        if (len === "70") btn.textContent = t.presetShort;
        else if (len === "140") btn.textContent = t.presetMedium;
        else if (len === "250") btn.textContent = t.presetLong;
    });

    const labelCustomLength = document.getElementById("labelCustomLength");
    if (labelCustomLength) labelCustomLength.textContent = t.labelCustomLength;

    if (inputLimite) inputLimite.placeholder = t.placeholderCustomLength;

    const unitCharacters = document.getElementById("unitCharacters");
    if (unitCharacters) unitCharacters.textContent = t.unitCharacters;

    const hintLength = document.getElementById("hintLength");
    if (hintLength) hintLength.textContent = t.hintLength;

    // Seção Alertas Proativos
    const sectionProactiveTitle = document.getElementById("sectionProactiveTitle");
    if (sectionProactiveTitle) sectionProactiveTitle.textContent = t.sectionProactive;

    const spanMaintenance = document.getElementById("spanMaintenance");
    if (spanMaintenance) spanMaintenance.innerHTML = t.checkMaintenance;

    const spanWellBeing = document.getElementById("spanWellBeing");
    if (spanWellBeing) spanWellBeing.innerHTML = t.checkWellBeing;

    const labelFrequency = document.getElementById("labelFrequency");
    if (labelFrequency) labelFrequency.textContent = t.labelFrequency;

    frequencyButtons.forEach((btn) => {
        const fKey = btn.getAttribute("data-frequency");
        if (fKey === "discreto") btn.textContent = t.freqDiscreet;
        else if (fKey === "equilibrado") btn.textContent = t.freqBalanced;
        else if (fKey === "frequente") btn.textContent = t.freqFrequent;
    });

    if (frequencyHint && t.frequencyHints[frequenciaAtual]) {
        frequencyHint.textContent = t.frequencyHints[frequenciaAtual];
    }

    // Seção Inicialização
    const sectionStartupTitle = document.getElementById("sectionStartupTitle");
    if (sectionStartupTitle) sectionStartupTitle.textContent = t.sectionStartup;

    const spanAutostart = document.getElementById("spanAutostart");
    if (spanAutostart) spanAutostart.textContent = t.checkAutostart;

    // Seção Atalhos
    const sectionShortcutsTitle = document.getElementById("sectionShortcutsTitle");
    if (sectionShortcutsTitle) sectionShortcutsTitle.textContent = t.sectionShortcuts;

    const shortcutChatLabel = document.getElementById("shortcutChatLabel");
    if (shortcutChatLabel) shortcutChatLabel.textContent = t.shortcutChatLabel;

    const shortcutChatDesc = document.getElementById("shortcutChatDesc");
    if (shortcutChatDesc) shortcutChatDesc.textContent = t.shortcutChatDesc;

    // Seção Tutorial
    const tutorialLabel = document.getElementById("tutorialLabel");
    if (tutorialLabel) tutorialLabel.textContent = t.tutorialLabel;

    const tutorialImg1 = document.getElementById("tutorialImg1") as HTMLImageElement;
    if (tutorialImg1) tutorialImg1.alt = t.tutorialStep1Alt;

    const tutorialImg2 = document.getElementById("tutorialImg2") as HTMLImageElement;
    if (tutorialImg2) tutorialImg2.alt = t.tutorialStep2Alt;

    const tutorialDesc = document.getElementById("tutorialDesc");
    if (tutorialDesc) tutorialDesc.innerHTML = t.tutorialDesc;

    // Seção Feedback
    const sectionFeedbackTitle = document.getElementById("sectionFeedbackTitle");
    if (sectionFeedbackTitle) sectionFeedbackTitle.textContent = t.sectionFeedback;

    const hintFeedback = document.getElementById("hintFeedback");
    if (hintFeedback) hintFeedback.textContent = t.hintFeedback;

    const labelFeedbackName = document.getElementById("labelFeedbackName");
    if (labelFeedbackName) labelFeedbackName.textContent = t.labelFeedbackName;

    if (feedbackName) feedbackName.placeholder = t.placeholderFeedbackName;

    const labelFeedbackType = document.getElementById("labelFeedbackType");
    if (labelFeedbackType) labelFeedbackType.textContent = t.labelFeedbackType;

    const optSuggestion = document.getElementById("optSuggestion");
    if (optSuggestion) optSuggestion.textContent = t.optSuggestion;

    const optBug = document.getElementById("optBug");
    if (optBug) optBug.textContent = t.optBug;

    const optPraise = document.getElementById("optPraise");
    if (optPraise) optPraise.textContent = t.optPraise;

    const labelFeedbackMessage = document.getElementById("labelFeedbackMessage");
    if (labelFeedbackMessage) labelFeedbackMessage.textContent = t.labelFeedbackMessage;

    if (feedbackMessage) feedbackMessage.placeholder = t.placeholderFeedbackMessage;

    if (btnEnviarFeedback && !btnEnviarFeedback.disabled) {
        btnEnviarFeedback.textContent = t.btnSendFeedback;
    }

    // Botão de Atualização
    if (labelCheckUpdates && (!btnCheckUpdates || !btnCheckUpdates.disabled)) {
        labelCheckUpdates.textContent = t.btnCheckUpdates;
    }

    // Botão Salvar
    if (btnSalvar) btnSalvar.textContent = t.btnSave;

    // Rodapé
    const footerText = document.getElementById("footerText");
    if (footerText) {
        footerText.innerHTML = `${t.footerDevBy} <a href="https://alanclimaco.github.io/Portfolio/" target="_blank" class="footer-link">Alan Clímaco</a>`;
        const link = footerText.querySelector('a.footer-link');
        if (link) {
            link.addEventListener('click', (e) => {
                e.preventDefault();
                const url = (link as HTMLAnchorElement).href;
                if (url && url !== "XXX" && !url.endsWith("/XXX")) {
                    openUrl(url).catch(() => window.open(url, '_blank'));
                }
            });
        }
    }
}

// Configura listeners nos botões de idioma
languageButtons.forEach((btn) => {
    btn.addEventListener("click", () => {
        const lang = btn.getAttribute("data-lang") || "pt";
        selecionarIdioma(lang);
    });
});

function selecionarPersonalidade(personalidade: string) {
    let persona = personalidade || "espelho";
    if (persona === "sarcastico") {
        persona = "hostil";
    } else if (persona === "manuel_gomes") {
        persona = "espelho";
    }
    personalidadeAtual = persona;

    personalityButtons.forEach((btn) => {
        if (btn.getAttribute("data-personality") === personalidadeAtual) {
            btn.classList.add("active");
        } else {
            btn.classList.remove("active");
        }
    });

    const t = TRANSLATIONS[idiomaAtual];
    if (personalityHint && t?.personalityHints[personalidadeAtual]) {
        personalityHint.textContent = t.personalityHints[personalidadeAtual];
    }
}

// Configura listeners de clique nos botões de personalidade
personalityButtons.forEach((btn) => {
    btn.addEventListener("click", () => {
        const personality = btn.getAttribute("data-personality") || "espelho";
        selecionarPersonalidade(personality);
    });
});

function selecionarFrequencia(freq: string) {
    const f = (freq || "equilibrado").toLowerCase().trim();
    if (f === "discreto" || f === "frequente") {
        frequenciaAtual = f;
    } else {
        frequenciaAtual = "equilibrado";
    }

    frequencyButtons.forEach((btn) => {
        if (btn.getAttribute("data-frequency") === frequenciaAtual) {
            btn.classList.add("active");
        } else {
            btn.classList.remove("active");
        }
    });

    const t = TRANSLATIONS[idiomaAtual];
    if (frequencyHint && t?.frequencyHints[frequenciaAtual]) {
        frequencyHint.textContent = t.frequencyHints[frequenciaAtual];
    }
}

frequencyButtons.forEach((btn) => {
    btn.addEventListener("click", () => {
        const freq = btn.getAttribute("data-frequency") || "equilibrado";
        selecionarFrequencia(freq);
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
        // 1. Carrega as configurações salvas (nome, chave Gemini, personalidade, limite, gatilhos, gostos e idioma)
        const config = await invoke<AppConfig>("carregar_configuracoes");
        if (config) {
            if (config.idioma) {
                selecionarIdioma(config.idioma);
            } else {
                selecionarIdioma("pt");
            }

            if (userName) userName.value = config.user_name || "";
            if (userInterests) userInterests.value = config.gostos_usuario || "";
            if (geminiApiKey) geminiApiKey.value = config.gemini_api_key || "";
            if (config.personalidade) {
                selecionarPersonalidade(config.personalidade);
            }
            if (config.limite_caracteres) {
                atualizarLimiteVisual(config.limite_caracteres);
            } else {
                atualizarLimiteVisual(70);
            }

            // Alertas proativos (2 categorias)
            if (checkAlertaManutencao) {
                checkAlertaManutencao.checked = config.alerta_manutencao !== false;
            }
            if (checkAlertaInteracaoBemEstar) {
                checkAlertaInteracaoBemEstar.checked = config.alerta_interacao_bem_estar !== false;
            }
            if (config.frequencia_alertas) {
                selecionarFrequencia(config.frequencia_alertas);
            }
        } else {
            selecionarIdioma("pt");
        }

        // Se ainda não tiver nome preenchido, busca do sistema
        if (userName && !userName.value) {
            const info = await invoke<UserInfo>("get_system_info");
            userName.value = info.name;
        }

        if (folderPath) {
            const info = await invoke<UserInfo>("get_system_info");
            folderPath.value = info.downloads;
        }
        
        // 2. Verifica se o Autostart já está ativado no sistema e marca a caixinha
        const autostartAtivo = await isEnabled();
        if (checkAutostart) checkAutostart.checked = autostartAtivo;

    } catch (e) {
        console.error("Erro ao carregar dados:", e);
        if (userName) userName.placeholder = TRANSLATIONS[idiomaAtual]?.placeholderUserNameError || "Erro de conexão";
    }
}

// Intercepta links externos para abrir no navegador padrão com o plugin-opener
document.querySelectorAll('a[href^="http"], a.footer-link').forEach(link => {
    link.addEventListener('click', (e) => {
        e.preventDefault();
        const url = (link as HTMLAnchorElement).href;
        if (url && url !== "XXX" && !url.endsWith("/XXX")) {
            openUrl(url).catch(() => window.open(url, '_blank'));
        }
    });
});

// Configura o envio assíncrono do formulário de feedback via Formspree
if (feedbackForm) {
    feedbackForm.addEventListener("submit", async (e) => {
        e.preventDefault();
        const t = TRANSLATIONS[idiomaAtual];
        const nome = feedbackName?.value?.trim() || "";
        const tipo = feedbackType?.value || "sugestao";
        const mensagem = feedbackMessage?.value?.trim() || "";

        if (!mensagem) return;

        if (btnEnviarFeedback) {
            btnEnviarFeedback.disabled = true;
            btnEnviarFeedback.textContent = t.btnSendingFeedback;
        }

        if (feedbackStatus) {
            feedbackStatus.style.display = "none";
            feedbackStatus.className = "feedback-status";
        }

        try {
            const response = await fetch(FORMSPREE_ENDPOINT, {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                    Accept: "application/json",
                },
                body: JSON.stringify({
                    name: nome,
                    type: tipo,
                    message: mensagem,
                }),
            });

            if (!response.ok) {
                throw new Error(`Erro ${response.status}`);
            }

            if (feedbackMessage) feedbackMessage.value = "";
            if (feedbackName) feedbackName.value = "";
            if (feedbackStatus) {
                feedbackStatus.textContent = t.feedbackSuccess;
                feedbackStatus.className = "feedback-status sucesso";
                feedbackStatus.style.display = "block";
                setTimeout(() => {
                    if (feedbackStatus) feedbackStatus.style.display = "none";
                }, 5000);
            }
        } catch (err) {
            console.error("Erro ao enviar feedback:", err);
            if (feedbackStatus) {
                feedbackStatus.textContent = t.feedbackError;
                feedbackStatus.className = "feedback-status erro";
                feedbackStatus.style.display = "block";
            }
        } finally {
            if (btnEnviarFeedback) {
                btnEnviarFeedback.disabled = false;
                btnEnviarFeedback.textContent = t.btnSendFeedback;
            }
        }
    });
}

// Configura a verificação manual de atualizações
if (btnCheckUpdates) {
    btnCheckUpdates.addEventListener("click", async () => {
        const t = TRANSLATIONS[idiomaAtual];
        btnCheckUpdates.disabled = true;
        btnCheckUpdates.classList.add("checking");
        if (labelCheckUpdates) labelCheckUpdates.textContent = t.btnCheckingUpdates;

        if (updateStatusMsg) {
            updateStatusMsg.style.display = "none";
            updateStatusMsg.className = "update-status-msg";
        }

        try {
            const result = await invoke<UpdateCheckResult>("verificar_atualizacoes_manual");
            if (updateStatusMsg) {
                updateStatusMsg.textContent = result.mensagem;
                updateStatusMsg.className = result.tem_atualizacao
                    ? "update-status-msg available"
                    : "update-status-msg latest";
                updateStatusMsg.style.display = "block";
            }
        } catch (err) {
            console.error("Erro ao verificar atualizações:", err);
            if (updateStatusMsg) {
                updateStatusMsg.textContent = String(err);
                updateStatusMsg.className = "update-status-msg error";
                updateStatusMsg.style.display = "block";
            }
        } finally {
            btnCheckUpdates.disabled = false;
            btnCheckUpdates.classList.remove("checking");
            if (labelCheckUpdates) labelCheckUpdates.textContent = t.btnCheckUpdates;
        }
    });
}

// Configura a navegação por abas horizontais (Windows 7 Property Sheet)
function configurarAbas() {
    const tabBtns = document.querySelectorAll<HTMLButtonElement>(".setup-tabs .tab-btn");
    const tabPanes = document.querySelectorAll<HTMLElement>(".setup-tab-viewport .tab-pane");

    tabBtns.forEach((btn) => {
        btn.addEventListener("click", () => {
            const targetId = btn.getAttribute("data-tab");
            if (!targetId) return;

            tabBtns.forEach((b) => {
                b.classList.remove("active");
                b.setAttribute("aria-selected", "false");
            });
            tabPanes.forEach((p) => {
                p.classList.remove("active");
            });

            btn.classList.add("active");
            btn.setAttribute("aria-selected", "true");

            const targetPane = document.getElementById(targetId);
            if (targetPane) {
                targetPane.classList.add("active");
            }
        });
    });
}

// Configura os cards retráteis (Accordions), se presentes
function configurarAccordions() {
    const accordions = document.querySelectorAll<HTMLElement>(".accordion-section");
    accordions.forEach((sec) => {
        const toggleBtn = sec.querySelector<HTMLButtonElement>(".accordion-toggle");
        const content = sec.querySelector<HTMLElement>(".accordion-body");
        if (toggleBtn && content) {
            toggleBtn.addEventListener("click", () => {
                const isOpen = sec.classList.toggle("is-open");
                toggleBtn.setAttribute("aria-expanded", String(isOpen));
                content.style.display = isOpen ? "block" : "none";
            });
        }
    });
}

// Executa imediatamente
configurarAbas();
configurarAccordions();
carregarDadosDoPC();

if (btnSalvar) {
    btnSalvar.addEventListener("click", async () => {
        const limiteFinal = Math.max(30, Math.min(500, parseInt(inputLimite?.value || "70", 10) || 70));
        const defaultName = idiomaAtual === "en" ? "User" : "Usuário";

        // Salva as configurações completas no Rust
        try {
            await invoke("salvar_configuracoes", {
                config: {
                    user_name: userName?.value?.trim() || defaultName,
                    gemini_api_key: geminiApiKey?.value?.trim() || "",
                    personalidade: personalidadeAtual,
                    limite_caracteres: limiteFinal,
                    alerta_manutencao: checkAlertaManutencao ? checkAlertaManutencao.checked : true,
                    alerta_interacao_bem_estar: checkAlertaInteracaoBemEstar ? checkAlertaInteracaoBemEstar.checked : true,
                    frequencia_alertas: frequenciaAtual,
                    idioma: idiomaAtual,
                    gostos_usuario: userInterests?.value?.trim() || "",
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
