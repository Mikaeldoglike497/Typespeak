!macro TYPESPEAK_KILL_CURRENT_USER_PROCESS executable_name
  nsis_tauri_utils::FindProcessCurrentUser "${executable_name}"
  Pop $R0
  ${If} $R0 = 0
    nsis_tauri_utils::KillProcessCurrentUser "${executable_name}"
    Pop $R0
  ${EndIf}
!macroend

!macro TYPESPEAK_VERIFY_CURRENT_USER_PROCESS_STOPPED executable_name retry_label
  nsis_tauri_utils::FindProcessCurrentUser "${executable_name}"
  Pop $R0
  ${If} $R0 = 0
    MessageBox MB_RETRYCANCEL|MB_ICONSTOP "TypeSpeak could not close ${executable_name}. Choose Retry, or quit TypeSpeak from the system tray and try again." IDRETRY ${retry_label}
    Abort
  ${EndIf}
!macroend

!macro TYPESPEAK_STOP_BACKGROUND_SERVICES retry_label
${retry_label}:
  DetailPrint "Closing TypeSpeak background services..."
  !insertmacro TYPESPEAK_KILL_CURRENT_USER_PROCESS "typespeak.exe"
  Sleep 250
  !insertmacro TYPESPEAK_KILL_CURRENT_USER_PROCESS "whisper-server.exe"
  !insertmacro TYPESPEAK_KILL_CURRENT_USER_PROCESS "whisper-cli.exe"
  !insertmacro TYPESPEAK_KILL_CURRENT_USER_PROCESS "crispasr.exe"
  !insertmacro TYPESPEAK_KILL_CURRENT_USER_PROCESS "parakeet-cli.exe"
  Sleep 750
  !insertmacro TYPESPEAK_KILL_CURRENT_USER_PROCESS "typespeak.exe"
  !insertmacro TYPESPEAK_KILL_CURRENT_USER_PROCESS "whisper-server.exe"
  !insertmacro TYPESPEAK_KILL_CURRENT_USER_PROCESS "whisper-cli.exe"
  !insertmacro TYPESPEAK_KILL_CURRENT_USER_PROCESS "crispasr.exe"
  !insertmacro TYPESPEAK_KILL_CURRENT_USER_PROCESS "parakeet-cli.exe"
  Sleep 500
  !insertmacro TYPESPEAK_VERIFY_CURRENT_USER_PROCESS_STOPPED "typespeak.exe" ${retry_label}
  !insertmacro TYPESPEAK_VERIFY_CURRENT_USER_PROCESS_STOPPED "whisper-server.exe" ${retry_label}
  !insertmacro TYPESPEAK_VERIFY_CURRENT_USER_PROCESS_STOPPED "whisper-cli.exe" ${retry_label}
  !insertmacro TYPESPEAK_VERIFY_CURRENT_USER_PROCESS_STOPPED "crispasr.exe" ${retry_label}
  !insertmacro TYPESPEAK_VERIFY_CURRENT_USER_PROCESS_STOPPED "parakeet-cli.exe" ${retry_label}
!macroend

!macro NSIS_HOOK_PREINSTALL
  !insertmacro TYPESPEAK_STOP_BACKGROUND_SERVICES typespeak_preinstall_stop_retry
!macroend

!macro NSIS_HOOK_POSTINSTALL
  IfSilent typespeak_default_model_done
  MessageBox MB_YESNO|MB_ICONQUESTION "Download the default Whisper model now? It is 574 MB and stays on this computer." IDNO typespeak_default_model_done
  ExecShell "open" "$INSTDIR\typespeak.exe" "--download-default-model"
typespeak_default_model_done:
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  !insertmacro TYPESPEAK_STOP_BACKGROUND_SERVICES typespeak_preuninstall_stop_retry
!macroend
