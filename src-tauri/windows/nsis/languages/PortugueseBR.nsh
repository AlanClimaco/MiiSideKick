; ===================================================================
; MiiSideKick - Arquivo de Idioma NSIS: Português do Brasil (PortugueseBR)
; ===================================================================

; Boas-vindas e Finalização do Instalador
LangString welcome_title ${LANG_PORTUGUESEBR} "Bem-vindo ao Instalador do MiiSideKick"
LangString welcome_text ${LANG_PORTUGUESEBR} "Este assistente instalará o MiiSideKick no seu computador.$\r$\n$\r$\nO MiiSideKick é o seu assistente de desktop com inteligência artificial, alertas proativos de manutenção e notificações interativas no estilo Frutiger Aero / Nintendo Wii.$\r$\n$\r$\nClique em Avançar para continuar."
LangString finish_title ${LANG_PORTUGUESEBR} "Instalação Concluída com Sucesso!"
LangString finish_text ${LANG_PORTUGUESEBR} "O MiiSideKick foi instalado no seu computador.$\r$\n$\r$\nO assistente permanecerá ativo na Bandeja do Sistema (System Tray). Você pode pressionar Ctrl + Shift + M a qualquer instante para falar com o Mii!$\r$\n$\r$\nClique em Concluir para finalizar o assistente."

; Transparência e Permissões
LangString transparency_header_title ${LANG_PORTUGUESEBR} "Transparência e Permissões do Sistema"
LangString transparency_header_subtitle ${LANG_PORTUGUESEBR} "Detalhes sobre o uso de IA, atalhos, memória RAM e privacidade"
LangString transparency_top ${LANG_PORTUGUESEBR} "Por favor, leia atentamente as informações sobre o funcionamento do MiiSideKick antes de prosseguir:"
LangString transparency_checkbox ${LANG_PORTUGUESEBR} "Eu li e concordo com os termos de transparência e permissões do sistema."

; Desinstalador
LangString un_welcome_title ${LANG_PORTUGUESEBR} "Desinstalação do MiiSideKick"
LangString un_welcome_text ${LANG_PORTUGUESEBR} "Este assistente removerá o MiiSideKick do seu computador.$\r$\n$\r$\nSentiremos sua falta! Clique em Avançar para prosseguir com a desinstalação."
LangString deleteAppData ${LANG_PORTUGUESEBR} "Remover também dados locais e chave de API (%APPDATA%\MiiSideKick)"
LangString un_finish_title ${LANG_PORTUGUESEBR} "Desinstalação Concluída"
LangString un_finish_text ${LANG_PORTUGUESEBR} "O MiiSideKick foi removido com sucesso do seu sistema."

; Mensagens operacionais do instalador (compatíveis com Tauri)
LangString addOrReinstall ${LANG_PORTUGUESEBR} "Adicionar/Reinstalar componentes"
LangString alreadyInstalled ${LANG_PORTUGUESEBR} "Já instalado"
LangString alreadyInstalledLong ${LANG_PORTUGUESEBR} "${PRODUCTNAME} ${VERSION} já está instalado no sistema. Selecione a operação desejada e clique em Avançar para continuar."
LangString appRunning ${LANG_PORTUGUESEBR} "O MiiSideKick está em execução! Por favor, feche o aplicativo na bandeja e tente novamente."
LangString appRunningOkKill ${LANG_PORTUGUESEBR} "O MiiSideKick está em execução em segundo plano!$\r$\nClique em OK para encerrá-lo automaticamente."
LangString chooseMaintenanceOption ${LANG_PORTUGUESEBR} "Escolha a opção de manutenção a realizar."
LangString chooseHowToInstall ${LANG_PORTUGUESEBR} "Escolha como deseja instalar ${PRODUCTNAME}."
LangString createDesktop ${LANG_PORTUGUESEBR} "Criar atalho na Área de Trabalho"
LangString dontUninstall ${LANG_PORTUGUESEBR} "Não desinstalar"
LangString dontUninstallDowngrade ${LANG_PORTUGUESEBR} "Não desinstalar (reversão de versão desativada neste instalador)"
LangString failedToKillApp ${LANG_PORTUGUESEBR} "Não foi possível finalizar o MiiSideKick automaticamente. Feche-o antes de tentar novamente."
LangString installingWebview2 ${LANG_PORTUGUESEBR} "Instalando o componente Microsoft Edge WebView2..."
LangString newerVersionInstalled ${LANG_PORTUGUESEBR} "Uma versão mais recente do ${PRODUCTNAME} já está instalada! Não é recomendado instalar uma versão antiga."
LangString olderOrUnknownVersionInstalled ${LANG_PORTUGUESEBR} "Uma versão anterior do ${PRODUCTNAME} foi encontrada. Deseja atualizá-la agora?"
LangString unableToUninstall ${LANG_PORTUGUESEBR} "Não foi possível desinstalar a versão existente."
LangString webview2AbortError ${LANG_PORTUGUESEBR} "Falha ao instalar o WebView2. O MiiSideKick necessita dele para exibir a interface."
