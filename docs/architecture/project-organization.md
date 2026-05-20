# Project Organization

Kinetik projects are file-based workspaces on disk. A creator starts by picking
or creating a folder; Kinetik Studio scaffolds the project structure and keeps
the workspace coherent through editor commands.

The workspace must stay friendly to Git, code review, external tools, and
agentic development. Source content lives in normal folders. Stable metadata is
centralized into small manifest files by domain instead of emitted as noisy
sidecar files for every asset.

Default project workspace:

```text
MyGame/
  Kinetik.toml
  scenes/
    main.knscene
  prefabs/
    enemy.knprefab
  scripts/
    player.luau
  assets/
    models/
      tree.glb
    textures/
      grass.png
  project/
    assets.knmanifest
    instances.knmanifest
  .kinetik/
    cache/
    import/
    build/
```

- `Kinetik.toml`: TOML project identity, engine compatibility, and project settings.
- `scenes`: RON scene files with instance hierarchy and scene settings.
- `prefabs`: RON reusable serialized instance trees.
- `scripts`: Luau source files owned by the project.
- `assets`: human-owned source assets.
- `project`: stable project metadata and manifests, including asset identity.
- `.kinetik`: generated cache, import output, and build artifacts.

`.kinetik` is disposable and should be ignored by Git. The editor must be able
to rebuild it from source files, scene files, prefabs, project settings, and
manifests.

Kinetik-authored project source files are deterministic text. Project settings
and manifests use TOML. Scene and prefab instance trees use RON.

Default scene hierarchy:

```text
Game
  Workspace
  Prefabs
  Scripts
  UI
  Lighting
  Audio
  Physics
  Assets
  Packages
```

- `Workspace`: live visible/simulated objects.
- `Prefabs`: inactive clone-ready scene templates.
- `Scripts`: global runtime scripts/managers.
- `UI`: runtime UI roots/templates.
- `Lighting`: sky, sun, atmosphere, time of day.
- `Audio`: global buses/mixers.
- `Physics`: gravity, collision layers, solver settings.
- `Assets`: file-backed resources.
- `Packages`: reusable external/local modules/content.

These default entries are real serialized instances scaffolded by Kinetik
Studio. The Explorer panel lists them by default as normal workable instances so
users can discover scene capabilities without knowing a hidden settings system
exists.

Prefab instances use explicit override records for local customization. Package
workflows should reuse the same inspectable override principles when packages
contain prefab or scene instance trees.

Agents should prefer editor/MCP commands when Kinetik Studio is open. Direct file
edits are appropriate for closed-editor workflows, migrations, tests, and review
tools, but the file layout must remain deterministic and easy to inspect.

First-party repository templates use the same project layout and live under
`templates/`. `examples/` is reserved for illustrative samples that are not
template acceptance targets. See `docs/architecture/template-projects.md` for
the template README, verification, screenshot, and golden-file contract.
