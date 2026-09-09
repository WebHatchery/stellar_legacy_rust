# Rights, credits and notices

This is an unresolved release record, not a license grant. Copyright holder,
project license and approved credit names require owner input; no rights or
public distribution approval are inferred from repository access.

## Sources to review

- Runtime dependencies: [generated inventory](THIRD_PARTY_LICENSES.md) and
  [upstream notice texts](THIRD_PARTY_NOTICES.txt).
- Bundled DejaVu font: [LICENSE-DejaVu.txt](../../assets/fonts/LICENSE-DejaVu.txt).
- Game assets/content: [asset provenance inventory](ASSET_PROVENANCE.csv).
- Title/store imagery and source-edit history: [media provenance](STORE_MEDIA_PROVENANCE.md).

The inventory script walks the Windows dependency graph from Cargo metadata and
its lockfile, and assembles locally available upstream license/notice texts.
It cannot establish project ownership or resolve missing notices. Review every
UNKNOWN, missing notice and distribution requirement for the final artifact.
Regeneration replaces inventory files; preserve verified provenance separately
before running it. Bundled notices retain their upstream text.

## Credits and disclosure

Supply approved design, writing, programming, art, publishing and other credit
names. Attribute dependencies and fonts according to their verified terms.
The title/store image source history includes AI-assisted edits; source rights,
prompts, model terms and subsequent edits need owner review. Authored event prose,
name data and other content also need creation-history review. Procedural runtime
silhouettes and synthesized audio are not evidence that all shipped content lacks
AI assistance.

No runtime generative-AI service is implemented. That fact is distinct from
pre-generated content provenance. Store disclosure, content-rating, privacy and
rights statements remain owner-approved release decisions recorded in
[release QA](QA_AND_OPERATIONS.md).
