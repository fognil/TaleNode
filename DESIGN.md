# TaleNode - Design Document
## Node-Based Dialogue & Quest Editor

---

## 1. Tổng quan sản phẩm

**TaleNode** là công cụ visual editor cho phép game developer tạo cốt truyện,
hội thoại và quest bằng cách kéo thả các node trên canvas và nối chúng lại.

**Output:** File `.json` chuẩn, import thẳng vào Unity / Godot / bất kỳ engine nào.

**Target users:** Indie game devs, narrative designers, visual novel creators.

**Giá bán mục tiêu:** $10 - $15 trên itch.io / gumroad.

---

## 2. Core Features

### 2.1 Node Editor (Canvas chính)
- Grid canvas vô hạn, zoom in/out, pan (kéo canvas)
- Kéo thả node từ sidebar hoặc right-click context menu
- Nối dây (wire) giữa các output → input port
- Multi-select nodes (kéo vùng chọn, Ctrl+Click)
- Copy / Paste / Delete nodes
- Undo / Redo (Ctrl+Z / Ctrl+Shift+Z)
- Minimap ở góc để overview toàn bộ graph
- Snap-to-grid (tuỳ chọn)

### 2.2 Các loại Node

```
┌─────────────────────────────────────────────────────────┐
│  NODE TYPE        │ MÔ TẢ                    │ MÀU SẮC │
├─────────────────────────────────────────────────────────┤
│  Start            │ Điểm bắt đầu conversation │ 🟢 Xanh lá│
│  Dialogue         │ NPC nói một câu thoại      │ 🔵 Xanh   │
│  Choice           │ Người chơi chọn đáp án     │ 🟡 Vàng   │
│  Condition        │ Rẽ nhánh theo điều kiện    │ 🟠 Cam    │
│  Event            │ Trigger game event          │ 🟣 Tím    │
│  Random           │ Random chọn nhánh           │ ⚪ Xám    │
│  End              │ Kết thúc conversation       │ 🔴 Đỏ     │
│  SubGraph         │ Chứa sub-dialogue graph     │ 🔷 Cyan   │
└─────────────────────────────────────────────────────────┘
```

#### Start Node
```
┌──────────────┐
│  ▶ START     │
│              │
│  ID: start_1 │──→ (output)
│              │
└──────────────┘
```
- Mỗi conversation có đúng 1 Start node
- Chỉ có output port, không có input

#### Dialogue Node
```
         ┌───────────────────────────┐
(input)──│  💬 DIALOGUE              │
         │                           │
         │  Speaker: [NPC Name    ]  │
         │  ┌─────────────────────┐  │
         │  │ "Xin chào, ngươi   │  │
         │  │  muốn gì?"         │  │──→ (output)
         │  └─────────────────────┘  │
         │                           │
         │  Portrait: [avatar.png ]  │
         │  Emotion:  [neutral ▼  ]  │
         │  Audio:    [clip_01.wav]  │
         └───────────────────────────┘
```
- **Speaker**: Tên nhân vật nói
- **Text**: Nội dung câu thoại (multi-line)
- **Portrait** (tuỳ chọn): Path đến ảnh avatar
- **Emotion** (tuỳ chọn): neutral, happy, sad, angry, surprised...
- **Audio** (tuỳ chọn): Path đến voice clip
- 1 input, 1 output

#### Choice Node
```
         ┌──────────────────────────────┐
(input)──│  ❓ CHOICE                    │
         │                              │
         │  Prompt: "Ngươi chọn gì?"   │
         │                              │
         │  [1] "Giúp đỡ"         ──→  │──→ output_1
         │  [2] "Từ chối"         ──→  │──→ output_2
         │  [3] "Hỏi thêm"       ──→  │──→ output_3
         │                              │
         │  [+ Thêm lựa chọn]          │
         └──────────────────────────────┘
```
- 1 input, N outputs (mỗi lựa chọn = 1 output port)
- Số lượng choice dynamic (thêm / xoá được)
- Mỗi choice có thể gắn condition (ẩn/hiện tuỳ game state)

#### Condition Node
```
         ┌─────────────────────────────┐
(input)──│  ⚙️ CONDITION                │
         │                             │
         │  Variable: [has_sword    ]  │
         │  Operator: [==  ▼       ]  │
         │  Value:    [true        ]  │
         │                             │
         │  ✅ True            ──→    │──→ output_true
         │  ❌ False           ──→    │──→ output_false
         └─────────────────────────────┘
```
- Check game variable → rẽ nhánh True / False
- Operators: ==, !=, >, <, >=, <=, contains

#### Event Node
```
         ┌─────────────────────────────┐
(input)──│  ⚡ EVENT                     │
         │                             │
         │  Action: [set_variable ▼]   │
         │  Key:    [quest_started ]   │──→ (output)
         │  Value:  [true          ]   │
         │                             │
         │  [+ Thêm action]           │
         └─────────────────────────────┘
```
- Trigger side-effect: set variable, give item, play animation...
- Nhiều action trong 1 event node
- Actions: set_variable, add_item, remove_item, play_sound, custom

#### Random Node
```
         ┌──────────────────────────┐
(input)──│  🎲 RANDOM               │
         │                          │
         │  [1] Weight: 50%   ──→  │──→ output_1
         │  [2] Weight: 30%   ──→  │──→ output_2
         │  [3] Weight: 20%   ──→  │──→ output_3
         └──────────────────────────┘
```
- Random chọn 1 nhánh theo weight

#### End Node
```
         ┌──────────────┐
(input)──│  ⏹ END       │
         │              │
         │  Tag: [good] │
         └──────────────┘
```
- Tag kết thúc: good_ending, bad_ending, neutral, continue...

#### SubGraph Node
```
         ┌─────────────────────────┐
(input)──│  📦 SUBGRAPH            │
         │                         │
         │  Name: [Side Quest    ] │──→ (output)
         │  Nodes: 5  Conns: 4    │
         │                         │
         │  Double-click to enter   │
         └─────────────────────────┘
```
- Chứa một DialogueGraph con bên trong (nested graph)
- Double-click để vào sub-graph, breadcrumb navigation để quay lại
- 1 input, 1 output
- Copy/paste sẽ regenerate tất cả ID bên trong

### 2.3 Variable System
- Panel bên trái: quản lý danh sách variables
- Mỗi variable: name, type (bool / int / float / string), default value
- Condition & Event nodes tham chiếu đến variables này
- Variables xuất ra trong JSON để game engine đọc

### 2.4 Character Database
- Panel quản lý nhân vật
- Mỗi character: id, name, color (màu tên trong dialogue), portrait_path
- Dialogue node chọn speaker từ danh sách này

### 2.5 File Operations

**Project:**
- **Save/Load** project file `.talenode` (format nội bộ, giữ layout node)
- **Auto-save** mỗi 60 giây (khi đã có save path)
- **New Project** (có confirmation dialog tránh mất dữ liệu)

**Export:**
- **Export JSON** — format chuẩn cho game engine (chỉ data, không layout)
- **Export XML** — XML serialization format
- **Export Godot Plugin** — runtime plugin cho Godot 4.x (`addons/talenode/`)
- **Export Unity Plugin** — runtime plugin cho Unity (`TaleNode/`)
- **Export Unreal Plugin** — runtime plugin cho Unreal Engine (`TaleNode/`)
- **Export Voice Script (CSV)** — bảng thoại cho voice actor
- **Export Locale CSV** — bảng dịch cho translator, import ngược lại
- **Export Analytics** — báo cáo thống kê graph (CSV hoặc text)

**Import:**
- **Import from Yarn** — import file Yarn Spinner (.yarn)
- **Import from Chat Mapper** — import file Chat Mapper (.json)
- **Import from articy** — import file articy:draft (.xml)

### 2.6 Localization System

Game devs cần ship hội thoại đa ngôn ngữ. TaleNode hỗ trợ localization trực tiếp trong tool.

**Concepts:**
- **Default locale**: Ngôn ngữ gốc (mặc định "en"), text lưu trực tiếp trong node fields
- **Extra locales**: Các ngôn ngữ bổ sung (ví dụ: "fr", "ja", "de"), lưu trong `LocaleSettings.translations`
- **String key**: Mỗi text field có stable key dựa trên UUID node (8 ký tự đầu): `dlg_{uuid8}`, `choice_{uuid8}`, `opt_{uuid8}_{index}`

**Data Model:**
```rust
struct LocaleSettings {
    default_locale: String,           // "en"
    extra_locales: Vec<String>,       // ["fr", "ja", "de"]
    translations: HashMap<String, HashMap<String, String>>,  // key → { locale → text }
}
```
- `DialogueGraph.locale: LocaleSettings` — field mới, `#[serde(default)]` cho backward compat
- Default text giữ nguyên trong `DialogueData.text`, `ChoiceData.prompt`, `ChoiceOption.text`
- Translation lưu trong `locale.translations["dlg_7de3cb62"]["fr"] = "Bonjour"`

**UI — Inspector Locale Switcher:**
- Combo box "Locale:" ở đầu Inspector (chỉ hiện khi có extra locales)
- Chọn "Default (en)": edit node text bình thường
- Chọn locale khác: hiện `[fr]` translation field bên dưới mỗi text field
- Field chưa dịch hiện placeholder "(untranslated)"

**UI — Localization Panel (Bottom tab):**
- Quản lý locale: Add/Remove locale
- Bảng translation: Key | Type | Default Text | [1 column mỗi locale]
- Inline editing cho translation cells
- Filter: All / Untranslated only, filter theo locale
- Progress bar mỗi locale: "fr: 45/120 (37%)"
- Buttons: Export CSV, Import CSV
- Navigate: Click "Go" để select node trên canvas

**CSV Format (cho translator):**
```
key,type,en,fr,ja
dlg_7de3cb62,dialogue,"Hello, traveler!","Bonjour, voyageur!","こんにちは！"
choice_4968a99b,prompt,"What will you do?","Que voulez-vous faire?",""
opt_4968a99b_0,option,"Fight","Combattre","戦う"
```

**JSON Export:**
Khi có extra locales, JSON export thêm 3 field:
```json
{
  "default_locale": "en",
  "locales": ["en", "fr", "ja"],
  "strings": {
    "dlg_1": { "en": "Hello!", "fr": "Bonjour!", "ja": "こんにちは！" }
  }
}
```
Dùng `skip_serializing_if` — field bị ẩn khi không có extra locales.

### 2.7 UX Nâng cao

**Search & Navigation:**
- **Search & Replace**: Tìm node theo text/speaker/tag, replace hàng loạt (Cmd+F / Cmd+H)
- **Zoom to Fit**: Fit toàn bộ graph vào viewport (phím F)
- **Node Tooltip**: Hover node hiện preview nội dung đầy đủ
- **Minimap**: Overview toàn bộ graph ở góc canvas

**Organization:**
- **Bookmark / Tag**: Gắn tag cho node, filter theo tag
- **Group**: Nhóm các node lại, tô màu nền, đặt tên
- **Template Library**: Lưu/load tổ hợp node thành template (có builtin templates)
- **SubGraph Navigation**: Nest dialogue graph, breadcrumb navigation

**Review & Collaboration:**
- **Comments Panel**: Thêm comment cho từng node, filter theo review status
- **Review Status**: Đánh dấu Draft / Needs Review / Approved cho mỗi node
- **Version History**: Lưu snapshot phiên bản, so sánh diff giữa 2 version, restore

**Playtest & Validation:**
- **Playtest Mode**: Chạy thử hội thoại ngay trong tool, node đang chạy glow xanh
- **Validation Panel**: Cảnh báo node không kết nối, dead-end, text trống
- **Analytics Panel**: Thống kê graph (node count, word count, path analysis)

**Editor:**
- **Script Editor**: Dual view — visual graph + text-based Yarn script, bidirectional sync
- **Audio Manager**: Batch assign audio clip, browse audio files
- **Confirmation Dialogs**: Xác nhận trước khi New Project hoặc Restore Version
- **Tooltips**: Tất cả button nhỏ đều có hover tooltip giải thích
- **Port Hover Feedback**: Port phóng to + glow khi hover, cursor thay đổi
- **Status Bar**: Hiện thông báo lỗi (đỏ) / thành công (xanh) cho file I/O
- **Themes**: Dark mode / Light mode
- **Collapsible Panels**: Left panel sections (Variables, Characters, Groups) thu gọn được

---

## 3. UI Layout

```
┌──────────────────────────────────────────────────────────────────┐
│  Menu Bar: [File] [Edit] [View] [Export] [Help]                  │
├────────────┬─────────────────────────────────────┬───────────────┤
│            │                                     │               │
│  LEFT      │        CENTER                       │  RIGHT        │
│  PANEL     │        CANVAS                       │  PANEL        │
│            │                                     │               │
│ ┌────────┐ │    ┌───┐     ┌───┐                  │ ┌───────────┐ │
│ │Chars   │ │    │ S ├────→│ D │                  │ │ Inspector │ │
│ │- NPC1  │ │    └───┘     └─┬─┘                  │ │           │ │
│ │- NPC2  │ │                │                    │ │ Speaker:  │ │
│ │        │ │           ┌────┴────┐               │ │ [NPC1   ] │ │
│ ├────────┤ │           │ Choice  │               │ │           │ │
│ │Vars    │ │           └┬──┬──┬──┘               │ │ Text:     │ │
│ │- hp    │ │            │  │  │                  │ │ [........]│ │
│ │- gold  │ │           ┌┘  │  └┐                 │ │ [........]│ │
│ │- quest │ │          ┌┴┐ ┌┴┐ ┌┴┐                │ │           │ │
│ │        │ │          │D│ │D│ │D│                │ │ Emotion:  │ │
│ ├────────┤ │          └┬┘ └┬┘ └─┘                │ │ [neutral] │ │
│ │Search  │ │           │   │                     │ │           │ │
│ │[     ] │ │          ┌┴┐ ┌┴┐                    │ │ Audio:    │ │
│ │        │ │          │E│ │E│                    │ │ [       ] │ │
│ └────────┘ │          └─┘ └─┘                    │ └───────────┘ │
│            │                                     │               │
│            │                          [Minimap]  │               │
├────────────┴─────────────────────────────────────┴───────────────┤
│  Status Bar: Nodes: 12 | Connections: 15 | Saved ✓ | Zoom: 100% │
└──────────────────────────────────────────────────────────────────┘
```

- **Menu Bar**: File (New/Open/Save/Export/Import), Edit (Undo/Redo/Select/Find), View (Panel toggles/Theme)
- **Search Bar** (toggle): Find & Replace, navigate matches
- **Breadcrumb Bar** (khi trong sub-graph): Back button + path navigation
- **Left Panel** (toggle): Variables, Characters, Groups — mỗi section collapsible
- **Center Canvas**: Vùng chính kéo thả node, grid background, minimap
- **Right Panel — Inspector** (khi chọn 1 node): Edit node properties, tags, review status
- **Right Panel — Script Editor** (toggle): Dual Yarn text editor, bidirectional sync
- **Bottom Panels** (toggle): Comments, Bookmarks, Analytics, Version History, Template Library, Validation, Playtest, Localization
- **Status Bar**: Node/connection count, zoom level, error/success messages (color-coded)

---

## 4. Kiến trúc Code (Rust + egui)

### 4.1 Tech Stack
- **Rust** (stable)
- **eframe** + **egui**: UI framework
- **egui_extras**: Bổ sung widgets
- **serde** + **serde_json**: Serialization
- **uuid**: Unique ID cho mỗi node
- **rfd** (Rusty File Dialogs): Open/Save file dialog

> **Lưu ý về egui_node_graph**: Thư viện này đã ngừng maintain từ 2023.
> Chúng ta sẽ tự viết node graph editor trên egui. Điều này cho phép:
> - Kiểm soát hoàn toàn UX/UI
> - Không phụ thuộc dependency chết
> - Tuỳ biến sâu cho dialogue editor

### 4.2 Project Structure

```
talenode/
├── Cargo.toml
├── DESIGN.md                    # Design doc (file này)
├── Makefile                     # Build/release scripts
├── src/
│   ├── main.rs                  # Entry point, setup eframe
│   │
│   ├── app/                     # Application logic (TaleNodeApp)
│   │   ├── mod.rs               # App state, keyboard shortcuts, panel layout
│   │   ├── canvas.rs            # Canvas interaction, node hit-test, drag, tooltip
│   │   ├── confirm.rs           # Confirmation dialog (PendingAction)
│   │   ├── context_menu.rs      # Right-click context menu
│   │   ├── file_io.rs           # Save/Open/Export/Import handlers
│   │   ├── file_io_locale.rs   # Locale CSV export/import handlers
│   │   ├── menu.rs              # Top menu bar (File/Edit/View)
│   │   ├── panel_handlers.rs    # Bottom panel action handlers
│   │   ├── panels.rs            # Panel layout + status bar rendering
│   │   ├── search.rs            # Search & Replace logic
│   │   ├── subgraph_nav.rs      # SubGraph enter/exit navigation
│   │   └── templates.rs         # Template insert/save/delete logic
│   │
│   ├── model/                   # Data structures (KHÔNG phụ thuộc UI)
│   │   ├── mod.rs
│   │   ├── node.rs              # Node struct, constructors, port management
│   │   ├── node_types.rs        # NodeType enum, DialogueData, ChoiceData...
│   │   ├── connection.rs        # Connection (wire) giữa 2 port
│   │   ├── port.rs              # Port, PortId, PortDirection
│   │   ├── graph.rs             # DialogueGraph — nodes + connections + tags
│   │   ├── graph_diff.rs        # GraphDiff — compare 2 graphs
│   │   ├── group.rs             # NodeGroup (visual grouping)
│   │   ├── locale.rs            # LocaleSettings, TranslatableString
│   │   ├── variable.rs          # Variable system
│   │   ├── character.rs         # Character database
│   │   ├── project.rs           # Project metadata, version snapshots
│   │   ├── review.rs            # ReviewStatus, NodeComment
│   │   ├── template.rs          # NodeTemplate, TemplateLibrary
│   │   ├── version.rs           # Version snapshot helpers
│   │   └── example_project.rs   # Built-in example project generator
│   │
│   ├── ui/                      # UI layer (egui rendering)
│   │   ├── mod.rs
│   │   ├── canvas.rs            # CanvasState — pan, zoom, grid, zoom-to-fit
│   │   ├── node_widget.rs       # Node rendering, ports, border, hover effects
│   │   ├── node_body.rs         # Node body content (type-specific text preview)
│   │   ├── connection_renderer.rs  # Bezier curves cho wires
│   │   ├── inspector.rs         # Right panel — node properties, tags, review, locale switcher
│   │   ├── inspector_widgets.rs # Reusable inspector widgets (dialogue, condition, locale...)
│   │   ├── left_panel.rs        # Left panel — variables, characters, groups
│   │   ├── locale_panel.rs      # Localization panel — translation table, progress, CSV
│   │   ├── playtest.rs          # Playtest panel + runtime
│   │   ├── script_panel.rs      # Dual script editor (Yarn text)
│   │   ├── comments_panel.rs    # Comments panel + review filter
│   │   ├── bookmark_panel.rs    # Bookmark/tag panel
│   │   ├── version_panel.rs     # Version history panel + diff viewer
│   │   ├── template_panel.rs    # Template library panel
│   │   ├── analytics_panel.rs   # Analytics dashboard
│   │   └── audio_manager.rs     # Audio clip batch assignment
│   │
│   ├── actions/                 # Undo/redo system
│   │   ├── mod.rs
│   │   └── history.rs           # UndoHistory (snapshot-based)
│   │
│   ├── export/                  # Export logic (KHÔNG import egui)
│   │   ├── mod.rs
│   │   ├── json_export.rs       # Export → JSON cho game engine
│   │   ├── json_export_helpers.rs # JSON export utility functions + string table builder
│   │   ├── json_export_types.rs # Exported struct definitions (ExportedDialogue, etc.)
│   │   ├── flatten.rs           # Flatten graph (resolve SubGraph nesting)
│   │   ├── locale_export.rs     # Locale CSV export/import
│   │   ├── xml_export.rs        # Export → XML
│   │   ├── yarn_export.rs       # Export → Yarn Spinner format
│   │   ├── voice_export.rs      # Export → Voice script CSV
│   │   ├── analytics_export.rs  # Export → Analytics CSV/text
│   │   └── plugin_export.rs     # Export runtime plugins (Godot/Unity/Unreal)
│   │
│   ├── import/                  # Import logic (KHÔNG import egui)
│   │   ├── mod.rs
│   │   ├── yarn_import.rs       # Import Yarn Spinner files
│   │   ├── yarn_build.rs        # Yarn → DialogueGraph builder
│   │   ├── chatmapper_import.rs # Import Chat Mapper files
│   │   ├── chatmapper_parse.rs  # Chat Mapper JSON parser
│   │   ├── articy_import.rs     # Import articy:draft files
│   │   └── articy_parse.rs      # articy XML parser
│   │
│   ├── scripting/               # Expression engine
│   │   ├── mod.rs               # Public API + tests
│   │   ├── expr.rs              # Expression AST + parser
│   │   ├── expr_tokenizer.rs    # Tokenizer for expressions
│   │   ├── eval.rs              # AST evaluator
│   │   └── interpolate.rs       # Text interpolation ({var}, {if}...{/if})
│   │
│   └── validation/              # Graph validation (KHÔNG import egui)
│       ├── mod.rs
│       ├── validator.rs         # Check lỗi: disconnected, dead-end, empty text
│       └── analytics.rs         # Graph analytics (stats, path analysis)
│
├── plugins/                     # Runtime plugins cho game engines
│   ├── godot/addons/talenode/   # Godot 4.x addon
│   │   ├── plugin.cfg
│   │   ├── talenode_expr.gd     # Expression engine (GDScript)
│   │   └── talenode_runner.gd   # Dialogue runner (GDScript)
│   │
│   ├── unity/TaleNode/          # Unity plugin
│   │   ├── TaleNodeExpression.cs # Expression engine (C#)
│   │   └── TaleNodeRunner.cs    # Dialogue runner (C#)
│   │
│   └── unreal/TaleNode/         # Unreal Engine plugin
│       ├── TaleNodeRunner.h     # Runner header
│       ├── TaleNodeRunner.cpp   # Dialogue runner (C++)
│       ├── TaleNodeProcess.cpp  # Node processing
│       └── TaleNodeValue.cpp    # Value type system
│
├── docs/                        # Documentation (mkdocs)
├── examples/                    # Example project files
└── .claude/                     # AI coding rules (CLAUDE.md)
```

### 4.3 Core Data Model

```rust
// === Node ===
struct Node {
    id: Uuid,
    node_type: NodeType,
    position: [f32; 2],      // Vị trí trên canvas (KHÔNG dùng egui::Vec2)
    inputs: Vec<Port>,        // Input ports
    outputs: Vec<Port>,       // Output ports
}

enum NodeType {
    Start,
    Dialogue(DialogueData),
    Choice(ChoiceData),
    Condition(ConditionData),
    Event(EventData),
    Random(RandomData),
    End(EndData),
    SubGraph(SubGraphData),   // Nested dialogue graph
}

struct DialogueData {
    speaker_id: Option<Uuid>,     // ref → Character
    text: String,
    portrait_override: Option<String>,
    emotion: String,
    audio_clip: Option<String>,
    metadata: HashMap<String, String>,  // custom data
}

struct ChoiceData {
    prompt: String,                     // Câu hỏi hiển thị
    choices: Vec<ChoiceOption>,
}

struct ChoiceOption {
    id: Uuid,
    text: String,
    condition: Option<ConditionExpr>,   // Ẩn nếu condition false
}

struct ConditionData {
    variable_name: String,
    operator: CompareOp,
    value: VariableValue,
}

enum CompareOp { Eq, Neq, Gt, Lt, Gte, Lte, Contains }

struct EventData {
    actions: Vec<EventAction>,
}

struct EventAction {
    action_type: EventActionType,
    key: String,
    value: VariableValue,
}

enum EventActionType {
    SetVariable,
    AddItem,
    RemoveItem,
    PlaySound,
    Custom(String),
}

struct RandomData {
    branches: Vec<RandomBranch>,
}

struct RandomBranch {
    id: Uuid,
    weight: f32,    // 0.0 - 1.0
}

struct EndData {
    tag: String,    // good_ending, bad_ending...
}

struct SubGraphData {
    name: String,
    child_graph: DialogueGraph,  // Nested graph
}

// === Connection ===
struct Connection {
    id: Uuid,
    from_node: Uuid,
    from_port: PortId,
    to_node: Uuid,
    to_port: PortId,
}

// === Graph (trung tâm) ===
struct DialogueGraph {
    nodes: HashMap<Uuid, Node>,
    connections: Vec<Connection>,
    variables: Vec<Variable>,
    characters: Vec<Character>,
    groups: Vec<NodeGroup>,           // Visual grouping
    node_tags: HashMap<Uuid, Vec<String>>,  // Bookmarks/tags
    review_statuses: HashMap<Uuid, ReviewStatus>,
    comments: Vec<NodeComment>,
    locale: LocaleSettings,           // Localization (serde default)
}

// === Localization ===
struct LocaleSettings {
    default_locale: String,                                    // "en"
    extra_locales: Vec<String>,                                // ["fr", "ja"]
    translations: HashMap<String, HashMap<String, String>>,    // key → { locale → text }
}

// === Group ===
struct NodeGroup {
    id: Uuid,
    name: String,
    color: [u8; 4],           // RGBA
    node_ids: Vec<Uuid>,
}

// === Review ===
enum ReviewStatus { Draft, NeedsReview, Approved }
struct NodeComment {
    id: Uuid,
    node_id: Uuid,
    text: String,
    timestamp: String,
}

// === Template ===
struct NodeTemplate {
    id: Uuid,
    name: String,
    description: String,
    nodes: Vec<Node>,
    connections: Vec<Connection>,
    is_builtin: bool,
}

// === Version ===
struct VersionSnapshot {
    id: usize,
    description: String,
    timestamp: String,
    graph: DialogueGraph,
}

// === Variable ===
struct Variable {
    id: Uuid,
    name: String,
    var_type: VariableType,
    default_value: VariableValue,
}

enum VariableType { Bool, Int, Float, Text }
enum VariableValue { Bool(bool), Int(i64), Float(f64), Text(String) }

// === Character ===
struct Character {
    id: Uuid,
    name: String,
    color: [u8; 4],          // RGBA (không dùng egui::Color32 trong model)
    portrait_path: String,
}
```

### 4.4 Undo / Redo System

Sử dụng **snapshot-based undo** — lưu toàn bộ `DialogueGraph` clone trước mỗi thay đổi.

```rust
struct UndoHistory {
    undo_stack: Vec<DialogueGraph>,  // Snapshot trước thay đổi
    redo_stack: Vec<DialogueGraph>,  // Snapshot để redo
    max_history: usize,               // Giới hạn 50 snapshots
}
```

**Flow:**
1. Trước khi mutate graph → `history.save_snapshot(&graph)` (clone toàn bộ graph)
2. Ctrl+Z → swap current graph với top of undo_stack, push current vào redo_stack
3. Ctrl+Shift+Z → swap current graph với top of redo_stack, push current vào undo_stack

**Khi nào snapshot:**
- Trước khi drag node (snapshot 1 lần khi bắt đầu drag, không phải mỗi frame)
- Trước khi edit field trong inspector (khi text field gained_focus)
- Trước khi add/remove node, connection, variable, character
- Trước khi apply import hoặc restore version

### 4.5 Render Pipeline (Canvas)

```
Mỗi frame egui:
  1. Handle input (pan, zoom, click, drag)
  2. Transform: screen coords ↔ canvas coords (dựa trên pan_offset + zoom)
  3. Render grid background
  4. Render groups (colored rectangles behind nodes)
  5. Render connections (bezier curves)
  6. Detect port hover (hit-test all ports under cursor)
  7. Render nodes (with selection border, search match glow, port hover, playtest glow)
  8. Render node tooltip (if hovering node and not dragging)
  9. Render dragging wire preview (nếu đang kéo từ port)
  10. Render box selection overlay (nếu đang kéo chọn)
  11. Render minimap overlay
  12. Handle node interaction (click, drag, port click → tạo wire)
  13. Handle context menu (right-click → add node, group, template)
```

**Zoom/Pan:**
```rust
struct CanvasState {
    pan_offset: Vec2,     // Dịch chuyển canvas
    zoom: f32,            // 0.25 → 4.0
}

// Screen → Canvas
fn screen_to_canvas(screen_pos: Pos2, state: &CanvasState) -> Pos2 {
    ((screen_pos - state.pan_offset) / state.zoom).to_pos2()
}
```

**Bezier Wires:**
```
  Output port (P1)
      │
      ╰──╮
          │   Control points cách P1, P4
          │   một khoảng = |P4.x - P1.x| * 0.5
          ╰──╮
              │
        Input port (P4)
```

---

## 5. Export JSON Format

Đây là format output cho game engine:

```json
{
  "version": "1.0",
  "name": "main_quest_dialogue",
  "variables": [
    {
      "name": "has_sword",
      "type": "bool",
      "default": false
    },
    {
      "name": "gold",
      "type": "int",
      "default": 0
    }
  ],
  "characters": [
    {
      "id": "npc_elder",
      "name": "Village Elder",
      "portrait": "portraits/elder.png",
      "color": "#4A90D9"
    }
  ],
  "nodes": [
    {
      "id": "start_1",
      "type": "start",
      "next": "dlg_1"
    },
    {
      "id": "dlg_1",
      "type": "dialogue",
      "speaker": "npc_elder",
      "text": "Xin chào, dũng sĩ! Ngươi muốn gì?",
      "emotion": "happy",
      "portrait": null,
      "audio": null,
      "next": "choice_1"
    },
    {
      "id": "choice_1",
      "type": "choice",
      "prompt": "",
      "options": [
        {
          "text": "Tôi muốn nhận quest",
          "next": "dlg_2",
          "condition": null
        },
        {
          "text": "Tạm biệt",
          "next": "end_1",
          "condition": null
        },
        {
          "text": "Tôi đã có kiếm rồi",
          "next": "dlg_3",
          "condition": {
            "variable": "has_sword",
            "operator": "==",
            "value": true
          }
        }
      ]
    },
    {
      "id": "dlg_2",
      "type": "dialogue",
      "speaker": "npc_elder",
      "text": "Hãy đến hang động phía Bắc!",
      "emotion": "neutral",
      "next": "evt_1"
    },
    {
      "id": "evt_1",
      "type": "event",
      "actions": [
        {
          "action": "set_variable",
          "key": "quest_started",
          "value": true
        }
      ],
      "next": "end_2"
    },
    {
      "id": "cond_1",
      "type": "condition",
      "variable": "gold",
      "operator": ">=",
      "value": 100,
      "true_next": "dlg_rich",
      "false_next": "dlg_poor"
    },
    {
      "id": "end_1",
      "type": "end",
      "tag": "farewell"
    },
    {
      "id": "end_2",
      "type": "end",
      "tag": "quest_accepted"
    }
  ]
}
```

**Đặc điểm format:**
- Flat array of nodes (dễ parse, dễ lookup bằng id)
- `next` field thay cho connection (đã "bake" graph thành linked list)
- Không chứa vị trí node (không cần cho game engine)
- Condition inline trong choice option
- Dễ dàng mở rộng thêm field bằng game-specific metadata

**Localization fields (chỉ khi có extra locales):**
```json
{
  "version": "1.0",
  "name": "main_quest_dialogue",
  "default_locale": "en",
  "locales": ["en", "fr", "ja"],
  "strings": {
    "dlg_1": { "en": "Hello, traveler!", "fr": "Bonjour, voyageur!", "ja": "こんにちは！" },
    "choice_1_prompt": { "en": "What will you do?", "fr": "Que voulez-vous faire?", "ja": "" },
    "choice_1_opt_0": { "en": "Fight", "fr": "Combattre", "ja": "戦う" }
  },
  "variables": [...],
  "characters": [...],
  "nodes": [...]
}
```
- `default_locale`, `locales`, `strings` bị ẩn khi không có extra locales (skip_serializing_if)
- String keys dùng readable IDs giống nodes (dlg_1, choice_1, etc.)
- Game engine đọc `strings[node_id][current_locale]` để lấy text đã dịch

---

## 5.1 Runtime Plugins (Godot & Unity)

TaleNode cung cấp drop-in runtime plugin cho **Godot 4.x** (GDScript), **Unity** (C#), và **Unreal Engine** (C++).
Plugin xử lý: load JSON, duyệt node, evaluate condition, weighted random, event execution,
và expression engine đầy đủ (arithmetic, boolean, comparison, text interpolation).

### Export từ TaleNode

Menu **File > Export Godot Plugin...**, **Export Unity Plugin...**, hoặc **Export Unreal Plugin...**
chọn thư mục project → plugin tự tạo cấu trúc thư mục đúng.

### Godot 4.x — GDScript

**Cấu trúc:** `addons/talenode/` (3 files: `plugin.cfg`, `talenode_expr.gd`, `talenode_runner.gd`)

```gdscript
# Sử dụng TaleNodeRunner
var runner = TaleNodeRunner.new()
runner.dialogue_line.connect(_on_dialogue_line)
runner.choice_presented.connect(_on_choice)
runner.dialogue_ended.connect(_on_ended)
runner.load_dialogue("res://dialogues/intro.json")
runner.start()

# Khi nhận dialogue_line signal → hiển thị text, gọi runner.advance() để tiếp
# Khi nhận choice_presented signal → hiện UI chọn, gọi runner.choose(index)
```

**Signals:**
- `dialogue_line(speaker, text, emotion, portrait, audio, node_id)`
- `choice_presented(prompt, options: Array)`
- `dialogue_ended(tag)`
- `variable_changed(key, value)`
- `event_triggered(action, key, value)`

**Expression engine** (`TaleNodeExpr`):
- `interpolate_text(text, variables)` — parse `{variable}`, `{100 - gold}`, `{if cond}...{else}...{/if}`
- `parse(input)` → AST, `evaluate(ast, variables)` → Variant

### Unity — C#

**Namespace:** `TaleNode` (2 files: `TaleNodeExpression.cs`, `TaleNodeRunner.cs`)

```csharp
var runner = new TaleNodeRunner();
runner.OnDialogueLine += (s, e) => Debug.Log($"{e.Speaker}: {e.Text}");
runner.OnChoicePresented += (s, e) => ShowChoiceUI(e.Prompt, e.Options);
runner.OnDialogueEnded += (s, e) => Debug.Log($"Ended: {e.Tag}");
runner.LoadDialogue("Assets/Dialogues/intro.json");
runner.Start();

// Advance() sau dialogue line, Choose(index) sau choice
```

**Events:** `OnDialogueStarted`, `OnDialogueLine`, `OnChoicePresented`, `OnDialogueEnded`, `OnEventTriggered`, `OnVariableChanged`

**Expression engine** (`TaleNodeExpression`):
- `TaleNodeExpression.InterpolateText(text, variables)` — cùng syntax `{...}` như Godot
- `TaleNodeExpression.Evaluate(expr, variables)` → `TaleValue`
- `TaleNodeExpression.EvaluateBool(expr, variables)` → `bool`

### Unreal Engine — C++

**Cấu trúc:** `TaleNode/` (4 files: `TaleNodeRunner.h`, `TaleNodeRunner.cpp`, `TaleNodeProcess.cpp`, `TaleNodeValue.cpp`)

```cpp
// Sử dụng TaleNodeRunner
UTaleNodeRunner* Runner = NewObject<UTaleNodeRunner>();
Runner->LoadDialogue(FilePath);
Runner->Start();
// Advance() sau dialogue line, Choose(index) sau choice
```

### Expression Syntax (chung cho cả 3 plugin)

| Syntax | Ví dụ | Mô tả |
|---|---|---|
| `{variable}` | `{gold}` | Hiển thị giá trị biến |
| `{expr}` | `{100 - gold}` | Biểu thức toán học |
| `{if cond}A{else}B{/if}` | `{if has_key}Mở cửa{else}Khóa{/if}` | Conditional text |

**Operators (theo precedence thấp → cao):**
`||` → `&&` → `==`/`!=` → `>`/`<`/`>=`/`<=` → `+`/`-` → `*`/`/`/`%` → unary `!`/`-`

---

## 6. Lộ trình phát triển (Phases)

### Phase 1 — MVP Core ✅
- [x] Setup Rust project, eframe window
- [x] Data model: Node, Connection, Graph (12 tests)
- [x] Canvas: pan, zoom, grid background
- [x] Render node boxes (Start + Dialogue + End)
- [x] Kéo thả node trên canvas
- [x] Port system + kéo dây nối
- [x] Bezier curve rendering cho wires
- [x] Right-click context menu → thêm node

### Phase 2 — Full Node Types ✅
- [x] Choice node (dynamic outputs)
- [x] Condition node
- [x] Event node
- [x] Random node
- [x] SubGraph node (nested dialogue)
- [x] Inspector panel (edit properties khi click node)

### Phase 3 — Project Management ✅
- [x] Save / Load project file (.talenode)
- [x] Export JSON + XML
- [x] Export Godot/Unity/Unreal plugins
- [x] Variable panel
- [x] Character panel
- [x] Undo / Redo (snapshot-based)

### Phase 4 — Polish ✅
- [x] Minimap
- [x] Search & Replace
- [x] Validation panel (cảnh báo lỗi)
- [x] Preview / Playtest mode (with node glow)
- [x] Dark / Light theme
- [x] Node grouping
- [x] Keyboard shortcuts (13 shortcuts)
- [x] Bookmarks / Tags
- [x] Comments / Review system
- [x] Version history + diff
- [x] Template library
- [x] Analytics panel
- [x] Script editor (dual Yarn view)
- [x] Audio manager
- [x] Import from Yarn / Chat Mapper / articy
- [x] Voice script export (CSV)
- [x] Confirmation dialogs
- [x] Tooltips on all buttons
- [x] Port hover feedback
- [x] Node hover tooltip
- [x] Zoom-to-fit (F key)
- [x] Collapsible left panel sections
- [x] Color-coded status bar messages
- [x] Expression engine (scripting module)
- [x] Localization system (locale panel, inspector switcher, CSV export/import, JSON string table)

### Phase 5 — Ship
- [ ] Example project files
- [ ] Landing page / Screenshots
- [ ] Build cho Windows / macOS / Linux
- [ ] Đăng bán itch.io / gumroad

---

## 6.1 Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| Cmd/Ctrl+N | New Project (with confirmation) |
| Cmd/Ctrl+O | Open Project |
| Cmd/Ctrl+S | Save Project |
| Cmd/Ctrl+Z | Undo |
| Cmd/Ctrl+Shift+Z | Redo |
| Cmd/Ctrl+A | Select All Nodes |
| Cmd/Ctrl+D | Duplicate Selected |
| Cmd/Ctrl+F | Find |
| Cmd/Ctrl+Shift+H (Mac) / Cmd/Ctrl+H (Win) | Find & Replace |
| Delete / Backspace | Delete Selected Nodes |
| F | Zoom to Fit |
| Escape | Close Search / Exit SubGraph |
| Enter (in search) | Next Match |
| Middle Mouse Drag | Pan Canvas |
| Ctrl+Scroll | Zoom Canvas |
| Right Click | Context Menu |
| Double Click (SubGraph node) | Enter SubGraph |

---

## 7. Key Technical Decisions

| Quyết định | Lựa chọn | Lý do |
|---|---|---|
| UI Framework | egui / eframe | Nhẹ, cross-platform, Rust native |
| Node graph lib | Tự viết | egui_node_graph đã ngừng maintain |
| Serialization | serde_json | Chuẩn, dễ debug, game engine đều đọc được |
| ID system | UUID v4 | Unique, không conflict khi merge |
| Project format | JSON (.talenode) | Dễ version control (git-friendly) |
| Wire rendering | Cubic bezier | Trông đẹp, chuẩn industry |
| Undo system | Command pattern | Reliable, dễ mở rộng |

---

## 8. Competitive Edge (Điểm khác biệt)

So với các tool hiện có (Yarn Spinner, Dialogue System for Unity, Twine):

1. **Standalone** — Không cần Unity/Godot mới chạy được
2. **Lightweight** — Native Rust, mở lên tức thì, không cần Electron
3. **Engine-agnostic JSON** — Output chuẩn, dùng cho engine nào cũng được
4. **Built-in variable system** — Không cần plugin thêm
5. **Playtest trong tool** — Preview hội thoại mà không cần chạy game
6. **Cross-platform** — Windows, macOS, Linux từ 1 codebase
