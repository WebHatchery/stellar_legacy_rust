# Support, troubleshooting, privacy, and known issues

Support contact and response commitment: **OWNER INPUT REQUIRED**.

## Saves and crash reports

On Windows, saves, display preferences, the Chronicle, and crash logs live in
`%LOCALAPPDATA%\stellar_legacy`. Open HELP and select **OPEN SAVE FOLDER**. Keep that
folder when reinstalling or moving the portable game. A corrupt save is never an
instruction to delete the folder; preserve it for support and restore a known-good copy.

If the game closes unexpectedly, attach `crash_log.txt`, the release SHA-256 from the
manifest, Windows version, GPU, display resolution/DPI, and the action immediately before
the failure. Do not include unrelated personal files.

## Troubleshooting

1. Extract the entire ZIP before launching; do not run the executable from inside it.
2. Keep `stellar_legacy.exe` and `assets.zip` together.
3. Confirm antivirus quarantine did not remove either file.
4. Try muted audio from DISPLAY if an audio device changes or fails.
5. Preserve `%LOCALAPPDATA%\stellar_legacy` before replacing a build.

## Privacy approval draft

The scoped Windows build stores game progress and settings locally. It provides no
accounts, analytics, telemetry, advertising, multiplayer, or game-operated network
service. Crash logs are local files and are shared only if the player chooses to send
them to support. The owner must verify the final binary and approve this statement before
publication.

## Known issues

- Minimum/recommended Windows hardware remains unmeasured.
- Code signing is not active unless the owner supplies and authorises a certificate.
- Store-client install/update/uninstall and save-survival tests require human accounts
  and clean machines.
