# Interactive SceneGraph Evolution Handback

**Date**: 2026-05-15
**Scope**: petalTongue Interactive SceneGraph Pipeline (6 phases)
**Status**: COMPLETE — all 6 phases implemented and verified

## What Was Done

The petalTongue SceneGraph pipeline was evolved from static rendering to
fully interactive data exploration. This directly supports the Anderson QS
visualization vision from baseCamp papers and wetSpring experiments.

### Phase 1: Semantic data_id

Domain identity (gene names, feature labels, categories) now flows from
`DataBindingCompiler` through `compile_geometry` to `Primitive::data_id`.
Tooltips show meaningful names ("thrA", "lacZ-alpha") instead of "pt-0".

Files: `petal-tongue-scene/src/data_binding/mod.rs`,
`petal-tongue-scene/src/compiler/geometry.rs`

### Phase 2: Click-to-Select

New `SceneInteractionState` tracks selection and hover across frames.
`SceneWidget` upgraded from `Sense::hover()` to `Sense::click_and_drag()`.
Both tiled and expanded views support click-to-toggle-selection with
visual highlight overlays and a detail strip.

Files: `petal-tongue-ui/src/scene_interaction.rs` (new),
`petal-tongue-ui/src/scene_bridge/mod.rs`,
`petal-tongue-ui/src/scene_viewer.rs`

### Phase 3: ViewCamera (Pan/Zoom)

`ViewCamera` struct provides scroll-wheel zoom (cursor-centered) and
ctrl+drag/middle-drag pan. Hit-map queries inverse-transformed through
camera. Fit-to-view reset button.

Files: `petal-tongue-ui/src/scene_interaction.rs`,
`petal-tongue-ui/src/scene_viewer.rs`

### Phase 4: IPC Interaction Bridge

Scene click-selections fire `InteractionApplyRequest` with semantic
`data_id` targets and binding key as `grammar_id`. `apply_interaction`
sets `perspective_id` on outbound events. Springs can subscribe and react.

Files: `petal-tongue-ui/src/scene_viewer.rs`,
`petal-tongue-ipc/src/visualization_handler/interaction.rs`,
`petal-tongue-ui/src/app/panels.rs`

### Phase 5: Data-Driven Animation

`CompiledBinding` stores `prev_scene` and `source_binding`. Stream
updates stash the old scene before recompilation. Viewer renders
crossfade transitions using `Easing::EaseOut` over 350ms.

Files: `petal-tongue-ipc/src/visualization_handler/state/types.rs`,
`petal-tongue-ipc/src/visualization_handler/state/stream_handler.rs`,
`petal-tongue-ui/src/scene_viewer.rs`

### Phase 6: Parameter Controls

Collapsible controls strip in expanded view with combo boxes for geometry
type, coordinate system, and X/Y scale types. Changes recompile locally
through the full `DataBindingCompiler` -> `GrammarCompiler` pipeline.

Files: `petal-tongue-ui/src/scene_viewer.rs`

## Verification

- Full workspace `cargo check` passes (petalTongue, lithoSpore, headless)
- 222 tests pass, 0 failures
- No linter errors on modified files

## Upstream Impact

- **lithoSpore**: No code changes required. Existing `litho visualize`
  dashboards automatically benefit from interactive features.
- **foundation**: No changes. Validation data visualized through lithoSpore
  adapters inherits full interactivity.
- **petalTongue upstream**: `CompiledBinding` struct gained two new fields
  (`prev_scene`, `source_binding`). All construction sites updated.

## Open Items for primalPing

- `#[allow(dead_code)]` on 6 functions in `domain_charts/mod.rs` and
  `scene_paint.rs` — retained as reference implementations during
  progressive SceneGraph convergence. Remove when convergence is complete.
- Camera transform not yet applied to the SceneGraph paint pass itself
  (currently uses inverse on hit-map queries only). Full camera-in-paint
  integration is a follow-up.
