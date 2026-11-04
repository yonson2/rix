# [108] App Store header image

**Status:** done
**Milestone:** M5
**Depends on:** [051]
**Created:** 2026-11-04
**Closed:** 2026-11-04

## Problem
Since iOS 27, App Store Connect has "Header and Search Results" creative assets under
Product Page Information. They are optional: without them the App Store shows the
screenshots. A header makes the product page look finished, and Pedro asked for one
before the 1.0.1 review.

## Approach
`tools/store-art.py` draws the header (3840 x 1646, 21:9, no alpha) with the store art:
the headline and the subtitle in the game font on the left, the `2-game` shot on a tilted
phone with the glowing edge on the right, over a soft purple light. The layout follows the
example in Pedro's message (headline left, tilted phone right). No search results image:
the three captioned screenshots show there without one.

## Verification
`just store-art` writes `target/store-art/app-store/header.png`, 3840 x 1646, RGB; look at
it: the text does not touch the phone.

## Notes
- Specs (read 2026-11-04):
  https://developer.apple.com/help/app-store-connect/reference/app-information/creative-assets-specifications/
  Header 21:9 3840 x 1646 (or 16:9 5244 x 2950 PNG, which also covers search results);
  search results 3:2, 1920 x 1280 to 3840 x 2560; images without alpha. Third-party guides
  (aso.dev/app-store-connect/placements): optional, shown on iOS 27 and iPadOS 27 or later;
  Apple reviews each asset, with a version or alone through the Asset Library.
- No 16:9 universal image: its crops for the two placements are not published, and the
  search card is better with the screenshots.
- 2026-11-04: `just store-art` passed: every store seed still works after the [106] physics
  change. The game shots moved a little (enemy places); the uploaded screenshots stay valid.
- Result: header.png 3840 x 1646 RGB, 2.4 MB; checked at 1600 px wide.
