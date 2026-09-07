# Topology Queries + Style Inheritance Helpers (proposal)

## Motivation

We keep re-implementing the same topology queries in multiple places:

- walk ancestors to find the “owner” component (e.g. a `RenderableComponent` above a `ColorComponent`)
- search descendants to find “targets” (e.g. all renderables under a subtree)
- resolve “effective” inherited values (e.g. color/opacity/cutout) with override rules

This duplication makes semantics easy to misread, and leads to widget boilerplate.

A concrete pain point that surfaced while building a button widget: **it’s not obvious where a `ColorComponent` must live in the topology for text glyphs to inherit it**. Today, some systems look for styles as *immediate children* of nodes, others also walk ancestors, and some treat the component node itself as the style carrier.

This doc proposes a small set of shared helpers to:

- centralize ancestor/descendant traversal
- standardize style resolution precedence
- make it explicit (in one place) what “attach this under that” means

No code changes in this document; this is a refactor plan.

---

## Current patterns (audit)

### 1) “Nearest ancestor wins” traversals

Examples:

- Render pass classification:
  - `RenderableSystem::inherited_background_for_renderable` (nearest `BackgroundComponent` ancestor)
  - `RenderableSystem::inherited_overlay_for_renderable` (nearest `OverlayComponent` ancestor)
  - File: `src/engine/ecs/system/renderable_system.rs`

- Raycastable opt-in:
  - `BvhSystem::renderable_is_raycastable`: check immediate child `RaycastableComponent`, else walk ancestors for a `RaycastableComponent` node.
  - File: `src/engine/ecs/system/bvh_system.rs`

- Signal routing:
  - `RxWorld::compute_scope_chain`: walk ancestors to dispatch scoped handlers.
  - File: `src/engine/ecs/signals/rx_world.rs`

### 2) “Immediate child style override” + ancestor fallback

Examples:

- `RenderableSystem::inherited_color_for_renderable`:
  - if the renderable has an immediate `ColorComponent` child, it wins
  - else walk ancestors and search for an immediate `ColorComponent` child on those ancestors
  - File: `src/engine/ecs/system/renderable_system.rs`

Analogous patterns exist for opacity and transparent-cutout.

### 3) “Register style component” wants either an owner or descendant propagation

Examples:

- `RenderableSystem::register_color` / `register_opacity` / `register_transparent_cutout`:
  - walk ancestors to find an owning `RenderableComponent`
  - else treat the component as a subtree style and apply to descendant renderables that don’t have explicit overrides
  - File: `src/engine/ecs/system/renderable_system.rs`

### 4) “Collect targets in subtree” helpers

Examples:

- `RxIntentExecutor` uses `collect_*_targets` helpers:
  - `collect_color_targets`, `collect_transform_targets`, `collect_raycast_targets`, …
  - These are variations of “if target is X, use it, else DFS subtree to find X (or first-X-per-branch)”.
  - File: `src/engine/ecs/signals/intent_executor.rs`

- `RxMutationExecutor`:
  - `collect_text_targets` duplicates the pattern.
  - File: `src/engine/ecs/signals/mutation_executor.rs`

### 5) Text styling inheritance is special-cased

- `TextSystem::register_text` inherits some properties from *immediate children of the `TextComponent` root*:
  - font atlas (`TextureComponent`)
  - filtering (`TextureFilteringComponent`)
  - emissive (`EmissiveComponent`)
  - raycastable (`RaycastableComponent`)
  - File: `src/engine/ecs/system/text_system.rs`

Color is not handled in `TextSystem`; color ultimately affects glyph renderables via renderable/color logic.

---

## What’s unclear today (semantics)

There are (at least) two different mental models:

1) **Component-as-node style model**
   - “If a `ColorComponent` is in the ancestry chain, it should apply to the subtree.”

2) **Component-attached-to-node style model**
   - “A style is an immediate child of some node; systems look for that immediate child while walking ancestors.”

Both exist in the codebase, but not consistently.

This is why it’s easy for a widget author to do something that *looks* correct topologically, but doesn’t produce the expected rendering.

---

## Proposal: a shared topology-query helper module

Introduce an internal helper module (name bikeshed):

- `src/engine/ecs/topology_query.rs` (or `src/engine/ecs/topology/mod.rs`)

The helpers should be pure queries over `(world, start_component)` and avoid allocations where possible.

### A) Generic traversal helpers

- `ancestors_inclusive(world, start) -> impl Iterator<Item = ComponentId>`
  - yields `start`, then its parent, … up to root

- `descendants_dfs(world, start) -> impl Iterator<Item = ComponentId>`
  - DFS traversal over subtree

- `descendants_bfs(world, start) -> impl Iterator<Item = ComponentId>`
  - BFS traversal over subtree

### B) Component lookup helpers

- `immediate_child_component<T>(world, node) -> Option<ComponentId>`
  - finds the first child of `node` that is a `T`

- `nearest_ancestor_with_component<T>(world, start) -> Option<ComponentId>`
  - walks `ancestors_inclusive` and returns the first node that is a `T`

- `nearest_ancestor_with_immediate_child_component<T>(world, start) -> Option<ComponentId>`
  - walks ancestors and returns the first ancestor whose **immediate child** is a `T`

These three cover almost all of the repeated `children_of(...).find_map(...)` + `while let Some(parent)` patterns.

### C) “Effective style” resolution helpers

Standardize a single precedence rule for inheritable properties.

For renderable styles like color/opacity/cutout, a reasonable rule is:

1. **Renderable-local override**: immediate style child of the renderable node
2. **Nearest in ancestry**: first match while walking upward, accepting either:
   - a style component *node itself* (component-as-node model)
   - or an immediate child style component of an ancestor (component-attached-to-node model)

Expressed as a helper:

- `effective_style_component<T>(world, renderable) -> Option<ComponentId>`

Where `T` might be `ColorComponent`, `OpacityComponent`, `TransparentCutoutComponent`, `RaycastableComponent` (for enable/disable), etc.

This would directly address the “why doesn’t my text pick up color?” confusion by making it explicit that a style can be supplied either as:

- `ancestor_node -> (ColorComponent child)`
- `... -> ColorComponent node -> ...`

…and still work, with clear precedence.

### D) “Register style component” propagation helper

Several register functions share the same shape:

- If the component has an owning `RenderableComponent` ancestor, apply it there
- Else apply it to all descendant renderables, but do not clobber explicit per-renderable overrides

Propose:

- `apply_style_to_descendant_renderables_if_no_override(world, style_component_id, style_kind)`

Or, in more composable terms:

- `collect_descendant_renderables_without_override<T>(world, root) -> Vec<ComponentId>`

This unifies the BFS logic currently repeated in `register_color`, `register_opacity`, `register_transparent_cutout`.

---

## Proposal: document the widget-authoring conventions

Separately from helper functions, add (or expand) a doc section (likely in the existing widget/gizmo docs) to clearly state:

- “Renderable-local override”: attach style components as **immediate children of the renderable** for explicit override.
- “Group style”: attach style either:
  - as a child of a group node that sits above the subtree, or
  - as a node in the ancestry chain (if we standardize the inclusive-ancestor lookup)

This makes it much easier to build widgets without guesswork.

---

## Benefits

- Less duplicated traversal code across systems.
- Fewer subtle inconsistencies (especially around style inheritance).
- Clearer mental model for widget authors (less boilerplate; fewer “why didn’t this apply?” moments).

---

## Suggested next steps (future PRs)

1. Implement the helper module (pure queries + small iterators).
2. Refactor one system at a time:
   - Start with `RenderableSystem` style resolution (color/opacity/cutout)
   - Then BVH raycastable checks
   - Then intent/mutation `collect_*_targets` helpers
3. Update docs to reflect the standardized precedence.

(Again: this file is only a proposal; no behavior is changed yet.)
