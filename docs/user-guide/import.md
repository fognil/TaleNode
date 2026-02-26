# Import Formats

TaleNode can import dialogue graphs from four external formats. All imports are accessible from the **File** menu.

!!! warning
    Importing replaces the current graph. The operation supports undo — press ++ctrl+z++ to revert if needed.

---

## Yarn Spinner

**Menu**: File > Import from Yarn...

Imports a Yarn Spinner `.yarn` file.

### What gets imported

| Yarn concept | TaleNode equivalent |
|---|---|
| Node (title/body) | Dialogue node per line |
| `<<jump NodeName>>` | Connection to target node |
| `-> Option text` | Choice node with options |
| `<<set $var to value>>` | Variable + Event node (SetVariable) |
| `<<if $var ...>>` | Condition node |
| Speaker: text | Character + Dialogue node with speaker |

### Supported syntax

- Node headers (`title:`, `---`, `===`)
- Dialogue lines with optional `Speaker:` prefix
- Shortcut options (`->`)
- Jump commands (`<<jump>>`)
- Variable declarations and set commands
- Basic if/else conditions

### Limitations

- Inline expressions (e.g., `{$var + 1}`) are imported as literal text
- Functions and custom commands are not mapped

---

## Chat Mapper

**Menu**: File > Import from Chat Mapper...

Imports a Chat Mapper JSON export file.

### What gets imported

| Chat Mapper concept | TaleNode equivalent |
|---|---|
| Conversation | Dialogue graph |
| Dialogue entry | Dialogue node |
| Actor | Character |
| Link | Connection |
| Variable | Variable |
| Hub / group entry | Branching structure |

### Requirements

- The file must be a valid Chat Mapper JSON export (contains `Assets` > `Conversations`, `Actors`, `Variables`)
- Root entry is used as the starting point for each conversation

---

## articy:draft

**Menu**: File > Import from articy...

Imports an articy:draft JSON export file.

### What gets imported

| articy concept | TaleNode equivalent |
|---|---|
| DialogueFragment | Dialogue node (text, speaker from entity reference) |
| Hub | Branching point (multiple outputs) |
| Connection | Connection between nodes |
| Entity (character) | Character |
| Variable set | Variables |
| Input/Output pins | Ports |

### Requirements

- Export from articy:draft as JSON format
- The file must contain a `Packages` section with `Models` entries
- Supported model types: `DialogueFragment`, `Hub`, `FlowFragment`, `Condition`, `Instruction`

### Limitations

- articy scripts and complex expressions are imported as text, not evaluated
- Nested flow fragments are flattened

---

## Ink

**Menu**: File > Import from Ink...

Imports an Inkle Ink `.ink` file.

### What gets imported

| Ink concept | TaleNode equivalent |
|---|---|
| Knot (`=== knot_name ===`) | Group of Dialogue nodes |
| Dialogue line | Dialogue node |
| `Speaker:` prefix | Character + speaker assignment |
| `* Choice text` | Choice node with options |
| `-> divert` | Connection to target knot |
| `VAR name = value` | Variable (Bool, Int, Float, or Text) |
| `~ name = value` | Event node (SetVariable) |

### Supported syntax

- Knots and stitches as organizational units
- Dialogue lines with optional speaker prefix (`Guard: Halt!`)
- Choices (`*` and `+` prefixes) with optional conditions
- Diverts (`-> knot_name`)
- Global variable declarations (`VAR`)
- Variable assignment (`~ var = value`)
- Tags (`# tag`)

### Limitations

- Tunnels (`->->`) are not supported
- External functions (`EXTERNAL`) are skipped with a warning
- Inline logic (`{expression}`) is preserved as literal text
- Threads are not supported

---

## CLI Import

You can import files from the command line without opening the GUI:

```bash
# Convert Yarn file to .talenode
talenode import yarn dialogue.yarn -o project.talenode

# Convert Ink file, output to stdout
talenode import ink story.ink

# List available import formats
talenode import --list
```

Supported formats: `yarn`, `ink`, `articy`, `chatmapper`.

---

## After Import

After importing from any format:

1. Review the generated graph in the canvas — nodes are auto-positioned
2. Check the **Validation** panel for any warnings (disconnected nodes, missing Start, etc.)
3. Review **Characters** and **Variables** in the left panel — they are extracted from the imported file
4. Save the project as `.talenode` to preserve your work
