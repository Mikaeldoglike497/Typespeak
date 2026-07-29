!macro NSIS_HOOK_POSTINSTALL
  IfSilent typespeak_default_model_done
  MessageBox MB_YESNO|MB_ICONQUESTION "Download the default Whisper model now? It is 574 MB and stays on this computer." IDNO typespeak_default_model_done
  ExecShell "open" "$INSTDIR\typespeak.exe" "--download-default-model"
typespeak_default_model_done:
!macroend
