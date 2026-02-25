# Changelog

## [1.0.0] - 2026-02-26

### Added
- UPM package structure (`package.json`, assembly definitions)
- `TaleNodeDialogueData` serializable data classes for dialogue JSON
- `TaleNodeDialogue` ScriptableObject wrapper with metadata
- `TaleNodeDialogueImporter` ScriptedImporter for `.talenode.json` files
- Custom inspector showing node stats, characters, variables, locale coverage
- GraphView-based read-only dialogue graph visualization
- BFS auto-layout (left-to-right from start nodes)
- Node colors matching the TaleNode desktop app
- Locale dropdown for previewing translations in graph and playtest
- In-editor playtest panel with dialogue log, choice buttons, variable watch
- Playtest syncs with graph view to highlight active node
- Search field to filter nodes by name or ID
