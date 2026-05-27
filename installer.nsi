Unicode true
SetCompressor /SOLID lzma

!define APP_NAME "SimbaImportXMLCreator"
!define APP_VERSION "0.1.0"
!define APP_EXE "SimbaImportXMLCreator.exe"

Name "${APP_NAME}"
OutFile "${APP_NAME}-installer.exe"
InstallDir "$PROGRAMFILES64\${APP_NAME}"
RequestExecutionLevel admin

Section "Install"
    SetOutPath "$INSTDIR"
    File "target/x86_64-pc-windows-gnu/release/${APP_EXE}"

    ; Start menu shortcut
    CreateDirectory "$SMPROGRAMS\${APP_NAME}"
    CreateShortcut "$SMPROGRAMS\${APP_NAME}\${APP_NAME}.lnk" "$INSTDIR\${APP_EXE}"

    ; Desktop shortcut
    CreateShortcut "$DESKTOP\${APP_NAME}.lnk" "$INSTDIR\${APP_EXE}"

    ; Uninstaller
    WriteUninstaller "$INSTDIR\Uninstall.exe"
SectionEnd

Section "Uninstall"
    Delete "$INSTDIR\${APP_EXE}"
    Delete "$INSTDIR\Uninstall.exe"
    RMDir "$INSTDIR"
    Delete "$SMPROGRAMS\${APP_NAME}\${APP_NAME}.lnk"
    Delete "$DESKTOP\${APP_NAME}.lnk"
SectionEnd