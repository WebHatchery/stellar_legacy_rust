# Store-media provenance and editable masters

The delivery recipe is [store_media_recipe.py](../../scripts/store_media_recipe.py).
It reads masters under `docs/release/store_media/masters` and writes derivatives
under `delivery`. Game screenshots come from actual capture scenes, not image
generation. Existing deliveries are not certified against the latest UI: refresh
affected screenshots and review the final candidate before storefront use.

| Master | Source and transformation | AI assistance | Approval |
| --- | --- | --- | --- |
| `assets/title.png` | Existing repository key art | Unknown; owner review required | Human pending |
| `stellar_legacy_key_art_no_text.png` | Built-in image edit of `title.png`; removed only title/ornament for a text-free hero | Yes, 2026-08-26 | Human pending |
| `stellar_legacy_logo_transparent.png` | Built-in image edit/extraction of the established wordmark and compass mark | Yes, 2026-08-26 | Human pending |
| `stellar_legacy_icon_master.png` | Deterministic crop of the existing compass mark in `title.png` | Source status inherited | Human pending |
| `ui_*.png` screenshots | Deterministic in-game capture scenes from the retail code path | No generated replacement imagery | Human truthfulness review pending |

The recipe produces a text-free library hero and branded capsules with the game name/logo over
game art. No review quotes, awards, prices, discounts, or unsupported feature claims are
embedded. Delivery dimensions are encoded in the filenames and checked by the recipe.

Image-edit prompts:

- Logo: extract the existing gold two-line `STELLAR LEGACY` wordmark and compass-star to
  genuine transparency; preserve spelling, angular letterforms, proportions, and texture;
  remove scene, ship, tagline, and all other text.
- Hero: remove only the complete title/tagline/compass/rules from the existing key art and
  reconstruct continuous starfield; preserve ship, planet, moon, sunrise, lighting,
  composition, and color; add nothing.


Source provenance is retained even when a delivery is refreshed. Owner approval is
still pending; exact output sizes in the recipe are implementation settings, not
a claim that current storefront specifications have been reverified.
