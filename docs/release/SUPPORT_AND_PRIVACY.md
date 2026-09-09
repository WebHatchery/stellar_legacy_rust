# Support and privacy

This is the current support draft for version 0.2.1. The support contact and
response commitment remain owner decisions tracked in [release QA](QA_AND_OPERATIONS.md).

## Saves and crash reports

Full-game Windows saves, Chronicle, preferences and local crash logs use
`%LOCALAPPDATA%\stellar_legacy`. The tutorial demo uses `stellar_legacy_demo`.
Open **Utilities → Help → OPEN SAVE FOLDER** in the Windows game. Preserve this
folder before replacing a build; the install ZIP is separate from saved progress.

Browser saves live in localStorage under the serving origin and have no Windows
folder. Moving to a different origin/profile or clearing site storage can make
those saves unavailable. Demo and full-game namespaces are separate even on the
same origin. There is no cloud synchronization.

The campaign loader accepts 0.1.0 and 0.2.0 through the current migration path,
validates 0.2.1 saves and rejects unsupported formats. Failed loads attempt to
preserve a quarantined slot and report the outcome. Do not delete all saves to
resolve a corrupt slot; preserve the original and restore a known-good copy.

For a Windows failure, useful evidence is `crash_log.txt`, the artifact SHA-256,
Windows/GPU/display details and the preceding action. Logs stay local unless the
player chooses to share them. Avoid collecting unrelated files or credentials.

## Troubleshooting

1. Extract the complete ZIP before launching. Keep `stellar_legacy.exe` and
   `assets.zip` together; check quarantine if either is missing.
2. For readability, use **Display & sound**, the scale/text Reset controls and
   an appropriate color scheme. Close settings remains visible while contents scroll.
3. For audio issues, lower Sound or disable Ambience using the visible controls.
4. Preserve the save directory/site storage before updating. Report the exact
   load error if a save is quarantined or cannot be recovered.

## Privacy statement for owner approval

The game stores progress and preferences locally. Its current dependency features
and runtime implement no accounts, analytics, telemetry, advertising, multiplayer
or game-operated online service. Windows crash logging is local; browser hosting
and storefront services have their own behavior outside this game executable.
No runtime generative-AI system is implemented. Confirm these statements against
the final distributed build before publishing them as policy.

Hardware minimums, final supported Windows versions, store-client save survival
and optional code signing are not established merely by development-machine
builds. Those remaining checks and decisions are listed once in release QA.
