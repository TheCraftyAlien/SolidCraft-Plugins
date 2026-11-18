# Timber, a SolidCraft plugin

Standard timber sections at any length, as solid bodies: on the origin, or placed parallel or at
90 degrees to a timber you have selected. Works with SolidCraft 0.43.

## Install

In SolidCraft open **Preferences › Plugins**, choose **Install plugin…** and pick
`timber-0.1.1.zip`. It asks for no folders or extra permissions.

## Use

**Plugins › Timber** (also a Timber button on the Plugins ribbon tab). To place against another
timber, **click that timber first**, then open Timber.

| Field | What it does |
|---|---|
| Size | 4x2 sawn 47×100, 4x2 CLS 38×89, 3x2 sawn 47×75, 3x2 CLS 38×63, 2x2 sawn 47×50, 2x2 planed 44×44, 6x2 47×150, 8x2 47×200, 3x3 75×75, 4x4 100×100, battens 25×50 and 25×38, or Custom |
| Inches | Ticked: plain numbers are inches, and the run's parameters are in inches |
| Length | `2400`, or with its own unit, converted whatever the Inches box says: `2400 mm`, `2.4 m`, `8'`, `96"`, `8' 6"`, `8ft 6in`, `1 1/2"`. 0 or empty = as long as the selected timber |
| Place | At the origin · Parallel to selected · At 90 degrees to selected |
| Where | Side · Other side · On top · Underneath · Far end · Near end (see below) |
| Along it from | Start · Middle · End: where the first piece goes along the selected timber (90°, on top/underneath/side) |
| How many / Spacing | A run of pieces, spacing centre to centre (`400`, `600 mm`, `16"`); 0 = touching. Typed like Length |
| Direction | Auto, X, Y, Z (upright). Parallel always follows the selected timber |
| Lie | On edge (like a joist) · Flat (like a batten) · Auto: as the selected one when parallel |
| Custom thickness / width | Used when Size is Custom; typed like Length (`22`, `1 1/2"`) |
| Material | Softwood, Treated softwood, Oak or None |
| Name | Empty = the size (4x2). Taken names get a number; several pieces are #1, #2… |

What **Where** means:

| | Parallel | At 90 degrees |
|---|---|---|
| Side / Other side | Beside it, Spacing apart: joists, studs, doubling up | Against its side: a rail on a post |
| On top / Underneath | Stacked on or under it | Crossing over or under it: joists on a bearer |
| Far end / Near end | In line, butted on its end | Across its end: a rim joist |

Examples:
- **Deck frame:** a 6x2 bearer at the origin (3000). Select it → 4x2, At 90°, On top, How many 8,
  Spacing 400 → joists. Select the bearer → Parallel, Side, Length 0, Spacing 1800 → second bearer.
- **Stud wall:** a 4x2 CLS upright (Direction Z). Select it → Parallel, Side, How many 6, Spacing 600.
- **Fence:** a 4x4 post upright → select it → 3x2, At 90°, Side, Along from End → top rail.

Each run's length, width and thickness are design parameters (`T_4x2_length`, …), so a whole run can
be lengthened or changed to another section in Parameters afterwards. Several pieces are grouped in
one history folder.

## Limits

- The dialog in 0.43 cannot pick a body, so it uses what is selected when you press Run.
- The selected timber is measured by its world bounding box: it must lie square to the X/Y/Z axes.
- Pieces are made in the active component; if that component has been moved, placed pieces are
  offset by the same amount. Work in the root component, or one that has not been moved.
- Changing a length parameter later grows each piece from its lowest corner.
- Standard sections stay in mm (the size they are sold as) even with Inches ticked. A metric value typed
  with Inches ticked (`1200 mm`) keeps its mm parameter, so nothing is rounded.
- Woodwork's Cut list includes these bodies (sizes are right), but its sheet estimate treats them as sheets.

## Build

    python build.py            # unit tests, wasm build, dist/timber-<version>.zip, end-to-end tests
    python build.py --no-e2e   # without solidcraft-cli

Needs Rust with `wasm32-unknown-unknown`, `wasm-tools`, and `solidcraft-cli` for the end-to-end tests.
