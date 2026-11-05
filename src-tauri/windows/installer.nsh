; ===================================================================
; MiiSideKick - Custom NSIS Lifecycle Hooks (installer.nsh)
; ===================================================================

!define MIISIDEKICK_WINDOWS_DIR "${__FILEDIR__}"
!define MIISIDEKICK_ICONS_DIR "${__FILEDIR__}\..\icons"

; Configura o bitmap lateral para a desinstalação (Mii com reação frustrada)
!ifndef MUI_UNWELCOMEFINISHPAGE_BITMAP
  !define MUI_UNWELCOMEFINISHPAGE_BITMAP "${MIISIDEKICK_ICONS_DIR}\nsis-sidebar-uninstall.bmp"
!endif

; -------------------------------------------------------------------
; Hook de Pré-Instalação: finaliza o MiiSideKick se estiver rodando
; -------------------------------------------------------------------
!macro NSIS_HOOK_PREINSTALL
  DetailPrint "Verificando se o MiiSideKick está ativo na bandeja do sistema..."
  nsExec::Exec 'taskkill /F /IM MiiSideKick.exe'
!macroend

; -------------------------------------------------------------------
; Hook de Pré-Desinstalação: finaliza o processo antes de remover arquivos
; -------------------------------------------------------------------
!macro NSIS_HOOK_PREUNINSTALL
  DetailPrint "Encerrando MiiSideKick para liberação de arquivos..."
  nsExec::Exec 'taskkill /F /IM MiiSideKick.exe'
!macroend

; -------------------------------------------------------------------
; Hook de Pós-Desinstalação: Limpeza Inteligente de AppData
; -------------------------------------------------------------------
!macro NSIS_HOOK_POSTUNINSTALL
  ; Verifica se a caixa de seleção de limpeza de dados locais foi marcada pelo usuário
  ${If} $DeleteAppDataCheckboxState = 1
    DetailPrint "Realizando expurgo inteligente de configurações e chave de API em AppData..."
    SetShellVarContext current

    ; Remove o diretório de dados em Roaming (%APPDATA%\MiiSideKick)
    RMDir /r "$APPDATA\MiiSideKick"

    ; Remove o diretório de cache/WebView2 em Local (%LOCALAPPDATA%\MiiSideKick)
    RMDir /r "$LOCALAPPDATA\MiiSideKick"

    ; Limpeza de fallback para identificadores do bundle
    RMDir /r "$APPDATA\com.alã.assistente"
    RMDir /r "$LOCALAPPDATA\com.alã.assistente"
    RMDir /r "$APPDATA\com.ala.assistente"
    RMDir /r "$LOCALAPPDATA\com.ala.assistente"

    ; Limpeza de chaves de registro residuais
    DeleteRegKey HKCU "Software\MiiSideKick"
    DeleteRegKey HKCU "Software\com.alã.assistente"
    DeleteRegKey HKCU "Software\com.ala.assistente"
  ${EndIf}
!macroend
