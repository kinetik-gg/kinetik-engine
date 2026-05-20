# Basic FPS

## Purpose

Basic FPS proves Kinetik can describe and verify a minimal first-person
prototype loop with authored scene data and deterministic headless gameplay
helpers.

## Acceptance Target

M31: Basic FPS Prototype, issue #185.

## Engine Features

- Project-shaped first-party template layout.
- First-person player marker, camera, light, static primitive level geometry,
  and an interactable objective block.
- Headless input/controller/collision/interaction smoke covering move, look,
  interact, complete, and restart behavior.
- Editor play-mode lifecycle smoke without persisting runtime state.

## Not Covered

- Platform keyboard/mouse backend.
- OS pointer capture.
- Script-authored gameplay.
- GPU-backed editor viewport rendering.
- Rapier-backed dynamic physics.

## Human Verification

Review the template hierarchy and verification notes. Current playable evidence
is deterministic headless simulation over engine-owned input, controller,
collision, and interaction contracts.
