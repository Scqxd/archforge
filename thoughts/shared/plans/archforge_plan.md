# Implementation Plan: ARCHFORGE
Generated: 2026-02-11

## Goal

ARCHFORGE is a pure Rust TUI application that automates PKGBUILD creation, dependency prediction, and AUR management for Arch Linux. It provides a magical user experience where users can describe packages in natural language ("собери firefox с vaapi + u2f") and receive fully-formed PKGBUILDs ready for the AUR.

---

## Research Summary

### Key Technology Choices

| Component | Selected Technology | Rationale |
|-----------|---------------------|-----------|
| **TUI Framework** | Ratatui + Crossterm | Actively maintained, widget-based, excellent docs, async-friendly |
| **Local LLM** | candle-rs + gguf | Pure Rust, Hugging Face ecosystem, supports quantized models |
| **P2P Networking** | libp2p-rust | Mature, production-tested, supports QUIC/WebTransport |
| **Database** | sled | Pure Rust, embedded, B-tree based, fast key-value storage |
| **AUR Integration** | aur + pacman crates | Direct AUR RPC API access, pacman wrapper |
| **PKGBUILD Parsing** | nom | Rust's premier parsing combinator library |
| **CLI Parsing** | clap v4 | Derive macros, async subcommands, mature ecosystem |

### AUR RPC API Format (Verified)

```
Base: https://aur.archlinux.org/rpc.php?type=json

Endpoints:
- search?arg=<query>     → Search packages
- info?arg[]=<pkgname>   → Package details
- multiinfo?arg[]=...    → Bulk package info
```

### Ratatui Architecture Patterns (2025 Best Practices)

- MVC pattern with separation of state, view, and event handling
- Component-based widget composition
- Centralized `App` state with `Reducer` pattern for updates
- Async I/O combined with Tokio runtime

---

## Existing Codebase Analysis

**Current State:** Empty project directory with only `.tldrignore` configured.

This is a greenfield project requiring full implementation from scratch.

---

## Project Structure

```
/home/scqxd/ArchForge/
├── Cargo.toml                          # Workspace root
├── Cargo.lock
├── rust-toolchain.toml                # Toolchain version (stable)
├── LICENSE (AGPL-3.0)
├── README.md
├── CHANGELOG.md
├── archforge/
│   ├── Cargo.toml                     # Main binary crate
│   └── src/
│       ├── main.rs                    # Entry point, CLI/TTY detection
│       ├── lib.rs                     # Public re-exports
│       ├── cli.rs                     # clap command definitions
│       ├── config.rs                  # Config loading (~/.config/archforge/)
│       ├── error.rs                   # Global error types (thiserror)
│       └── tui/
│           ├── mod.rs
│           ├── app.rs                 # Central App state
│           ├── event.rs               # Event loop, crossterm handling
│           ├── key.rs                 # Vim-like keybindings
│           ├── components/
│           │   ├── mod.rs
│           │   ├── input.rs           # Command/input widget
│           │   ├── preview.rs         # PKGBUILD preview panel
│           │   ├── list.rs            # Package list, search results
│           │   ├── table.rs           # Dependency table
│           │   ├── status.rs          # Build progress, status bar
│           │   └── layout.rs          # Layout definitions
│           └── widgets/
│               ├── mod.rs
│               ├── pkgbuild_syntax.rs # Syntax highlighting
│               └── logs.rs            # Build log viewer
│
├── core/                              # AI_CORE agent (LLM integration)
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── model/
│       │   ├── mod.rs
│       │   ├── loader.rs              # GGUF model loading
│       │   ├── engine.rs              # candle-rs inference
│       │   └── tokenizer.rs           # HF-compatible tokenizers
│       ├── prompt/
│       │   ├── mod.rs
│       │   ├── pkgbuild.rs            # PKGBUILD-specific prompts
│       │   └── system.rs              # System prompt engineering
│       └── generation/
│           ├── mod.rs
│           ├── generator.rs           # Main generation logic
│           └── validator.rs           # PKGBUILD v2.2 validation
│
├── build/                             # BUILD_ENGINE agent
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── pkgbuild/
│       │   ├── mod.rs
│       │   ├── parser.rs              # nom-based PKGBUILD parser
│       │   ├── generator.rs           # PKGBUILD from struct
│       │   └── validator.rs           # v2.2 compliance checks
│       ├── builder/
│       │   ├── mod.rs
│       │   ├── makepkg.rs             # makepkg wrapper
│       │   ├── aur.rs                 # yay/paru integration
│       │   └── pacman.rs              # pacman operations
│       └── conflict/
│           ├── mod.rs
│           ├── analyzer.rs            # Dependency conflict detection
│           └── predictor.rs           # ML-based conflict prediction
│
├── swarm/                             # SWARM_AGENT (P2P telemetry)
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── network/
│       │   ├── mod.rs
│       │   ├── behaviour.rs           # libp2p Behaviour definition
│       │   ├── discovery.rs           # mDNS/Libp2p discovery
│       │   └── protocol.rs            # Custom swarm protocols
│       ├── storage/
│       │   ├── mod.rs
│       │   ├── db.rs                  # sled database wrapper
│       │   ├── telemetry.rs           # Build telemetry schema
│       │   └── cache.rs               # Model/dependency cache
│       └── sync/
│           ├── mod.rs
│           ├── gossipsub.rs           # PubSub for telemetry
│           └── sync.rs                # Peer-to-peer sync
│
├── deploy/                            # DEPLOYER agent
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── aur/
│       │   ├── mod.rs
│       │   ├── uploader.rs            # AUR package submission
│       │   ├── maintainer.rs          # AUR package maintenance
│       │   └── api.rs                 # AUR RPC client
│       ├── container/
│       │   ├── mod.rs
│       │   ├── docker.rs              # Docker build support
│       │   ├── flatpak.rs             # Flatpak + bubblewrap
│       │   └── nix.rs                 # Nix flake generation
│       └── vcs/
│           ├── mod.rs
│           ├── git.rs                 # Git integration
│           └── hg.rs                  # Mercurial support
│
├── examples/
│   ├── basic_usage.rs                 # "cargo run --example basic"
│   ├── firefox_vaapi.rs               # "собери firefox с vaapi"
│   ├── discord.rs                     # Discord package build
│   ├── vscode.rs                      # VSCode (VSCodium) build
│   └── custom_kernel.rs               # Custom kernel package
│
├── tests/
│   ├── integration/
│   │   ├── test_pkgbuild_gen.rs       # End-to-end generation tests
│   │   ├── test_aur_flow.rs           # AUR integration tests
│   │   └── test_deploy.rs             # Deployment tests
│   └── unit/
│       ├── pkgbuild_parser_test.rs    # Parser unit tests
│       ├── conflict_test.rs           # Conflict detection tests
│       └── model_inference_test.rs    # LLM inference tests
│
├── scripts/
│   ├── build.sh                       # Build script with musl
│   ├── generate_pkgs.sh               # Generate example PKGBUILDs
│   ├── train_model.py                 # Model training script
│   └── benchmark.sh                   # Performance benchmarks
│
├── resources/
│   ├── models/
│   │   ├── pkgbuild-v1.gguf           # Fine-tuned model
│   │   └── tokenizer.json             # Custom tokenizer
│   ├── prompts/
│   │   ├── system.txt                 # System prompt
│   │   └── examples.txt               # Few-shot examples
│   └── schemas/
│       ├── pkgbuild-v2.2.json         # PKGBUILD schema
│       └── aur-package.json           # AUR package schema
│
├── .github/
│   ├── workflows/
│   │   ├── ci.yml                     # CI pipeline
│   │   ├── release.yml                # Release pipeline
│   │   └── aur.yml                    # AUR package publish
│   └── ISSUE_TEMPLATE.md
│
├── PKGBUILD                           # For AUR distribution
├── .cargo/
│   └── config.toml                    # Build target configuration
│
└── .claude/
    └── skills/
        └── ...                        # Claude-specific skills
```

---

## Architecture Diagram

```
                    ┌─────────────────────────────────────────────────┐
                    │              ARCHFORGE TUI Application          │
                    │              =========================           │
                    │                                                 │
┌───────────────────┼─────────────────────────────────────────────────┼───────────────┐
│                   │                                                 │               │
│    ┌──────────────┴──────────────┐    ┌────────────────────────────┴────────────┐ │
│    │       CLI Interface         │    │      TUI Interface (Ratatui)          │ │
│    │   (clap, --help, args)      │    │      Crossterm, vim bindings         │ │
│    └──────────────┬──────────────┘    └────────────────────────────┬────────────┘ │
│                   │                                                 │               │
│                   └─────────────────────┬───────────────────────────┘               │
│                                         │                                           │
│                                         ▼                                           │
│                    ┌─────────────────────────────────────────────────┐             │
│                    │              APP STATE (Reducer Pattern)         │             │
│                    │    ┌─────────────────────────────────────┐       │             │
│                    │    │  AppState:                         │       │             │
│                    │    │  - current_view: View              │       │             │
│                    │    │  - packages: Vec<Package>          │       │             │
│                    │    │  - build_logs: Logs                │       │             │
│                    │    │  - model_status: ModelStatus       │       │             │
│                    │    │  - config: Config                  │       │             │
│                    │    └─────────────────────────────────────┘       │             │
│                    └─────────────────────────────────────────────────┘             │
│                                         │                                           │
│                    ┌────────────────────┼────────────────────┐                      │
│                    ▼                    ▼                    ▼                      │
│         ┌──────────────────┐ ┌──────────────────┐ ┌──────────────────┐             │
│         │   ARCHITECT      │ │    AI_CORE       │ │  BUILD_ENGINE    │             │
│         │   (UI + State)   │ │   (Local LLM)    │ │   (makepkg)      │             │
│         ├──────────────────┤ ├──────────────────┤ ├──────────────────┤             │
│         │ - Ratatui TUI    │ │ - candle-rs      │ │ - nom parser     │             │
│         │ - Tokio async    │ │ - GGUF models    │ │ - makepkg wrapper│             │
│         │ - CLI structure  │ │ - Tokenizers     │ │ - aur helpers    │             │
│         │ - Event loop     │ │ - Prompt eng.    │ │ - Conflict pred. │             │
│         └────────┬─────────┘ └────────┬─────────┘ └────────┬─────────┘             │
│                  │                    │                    │                       │
│                  │                    │                    │                       │
│                  └────────────────────┼────────────────────┘                       │
│                                         │                                           │
│                                         ▼                                           │
│                    ┌─────────────────────────────────────────────────┐             │
│                    │              SWARM_AGENT (P2P Network)          │             │
│                    │  ┌─────────────────────────────────────────┐    │             │
│                    │  │ libp2p-rust:                            │    │             │
│                    │  │   - QUIC/WebTransport transport          │    │             │
│                    │  │   - mDNS discovery                       │    │             │
│                    │  │   - GossipSub for telemetry              │    │             │
│                    │  │   - Identify protocol                    │    │             │
│                    │  └─────────────────────────────────────────┘    │             │
│                    │                    │                            │             │
│                    │                    ▼                            │             │
│                    │         ┌──────────────────┐                   │             │
│                    │         │ sled DB          │                   │             │
│                    │         │ - Build logs     │                   │             │
│                    │         │ - Telemetry      │                   │             │
│                    │         │ - Model cache    │                   │             │
│                    │         │ - Dependency DB  │                   │             │
│                    │         └──────────────────┘                   │             │
│                    └─────────────────────────────────────────────────┘             │
│                                         │                                           │
│                                         ▼                                           │
│                    ┌─────────────────────────────────────────────────┐             │
│                    │              DEPLOYER Agent                      │             │
│                    │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌───────┐ │             │
│                    │  │ AUR     │ │ Docker  │ │ Flatpak │ │ Nix   │ │             │
│                    │  │ Upload  │ │ Build   │ │ Bundled │ │ Flake │ │             │
│                    │  └─────────┘ └─────────┘ └─────────┘ └───────┘ │             │
│                    └─────────────────────────────────────────────────┘             │
│                                                                                     │
│                    ┌─────────────────────────────────────────────────┐             │
│                    │              External Systems                    │             │
│                    │  ┌─────────────┐ ┌─────────────┐ ┌──────────┐  │             │
│                    │  │ AUR RPC     │ │ AUR Web     │ │pacman    │  │             │
│                    │  │ JSON API    │ │ Interface   │ │(yay/paru)│  │             │
│                    │  └─────────────┘ └─────────────┘ └──────────┘  │             │
│                    └─────────────────────────────────────────────────┘             │
└─────────────────────────────────────────────────────────────────────────────────────┘
```

### Data Flow Diagram

```
Natural Language Input
         │
         ▼
┌─────────────────┐
│ TUI Input       │──┐
│ "собери firefox │   │
│  с vaapi + u2f" │   │
└─────────────────┘   │
                      ▼
              ┌───────────────┐
              │ ARCHITECT     │
              │ (Parse input) │
              └───────┬───────┘
                      │
                      ▼
              ┌───────────────┐     ┌─────────────────┐
              │ AI_CORE       │────▶│ PKGBUILD v2.2   │
              │ (Generation)  │     │ Generator       │
              └───────┬───────┘     └─────────────────┘
                      │
                      ▼
              ┌───────────────┐
              │ BUILD_ENGINE  │
              │ - Parser      │──┐
              │ - Validator   │  │
              └───────┬───────┘  │
                      │          │
                      ▼          │
              ┌───────────────┐  │
              │ Conflict      │  │    ┌─────────────┐
              │ Analyzer      │──┼───▶│ SWARM_AGENT │
              └───────────────┘  │    │ (P2P cache) │
                      │          │    └─────────────┘
                      ▼          │
              ┌───────────────┐  │
              │ Build Test    │  │    ┌─────────────┐
              │ (makepkg)     │──┼───▶│ Telemetry   │
              └───────────────┘  │    │ (Peer sync) │
                      │          │    └─────────────┘
                      ▼          │
              ┌───────────────┐  │
              │ DEPLOYER      │◀─┘
              │ - AUR Upload  │
              │ - Docker      │
              │ - Flatpak     │
              │ - Nix Flake   │
              └───────────────┘
                      │
                      ▼
              ┌───────────────┐
              │ User Gets:    │
              │ - PKGBUILD    │
              │ - AUR link    │
              │ - Logs        │
              └───────────────┘
```

---

## Implementation Phases

### Phase 1: Core Infrastructure (Weeks 1-2)

**Goal:** Establish project foundation, error handling, configuration, and basic CLI.

**Files to create/modify:**

- `/home/scqxd/ArchForge/Cargo.toml` - Workspace definition
- `/home/scqxd/ArchForge/archforge/Cargo.toml` - Main binary crate
- `/home/scqxd/ArchForge/archforge/src/error.rs` - Error types
- `/home/scqxd/ArchForge/archforge/src/config.rs` - Config loading
- `/home/scqxd/ArchForge/archforge/src/cli.rs` - CLI parsing

**Steps:**

1. Create workspace `Cargo.toml` with all member crates
2. Implement `ArchforgeError` enum with `thiserror`:
   ```rust
   #[derive(Error, Debug)]
   pub enum ArchforgeError {
       #[error(transparent)]
       Config(#[from] ConfigError),
       #[error(transparent)]
       Model(#[from] ModelError),
       #[error(transparent)]
       Build(#[from] BuildError),
       #[error(transparent)]
       Network(#[from] NetworkError),
       #[error("io error: {0}")]
       Io(#[from] std::io::Error),
   }
   ```
3. Implement config loader for `~/.config/archforge/config.toml`:
   ```toml
   [model]
   path = "~/.cache/archforge/models/pkgbuild-v1.gguf"
   temperature = 0.1
   max_tokens = 4096

   [build]
   makepkg_path = "/usr/bin/makepkg"
   aur_helper = "paru"

   [ui]
   theme = "arch_dark"
   vim_mode = true
   ```
4. Create clap CLI with subcommands:
   - `archforge generate "description"` - Generate PKGBUILD
   - `archforge build <pkgname>` - Build package
   - `archforge search <query>` - Search AUR
   - `archforge deploy --aur <pkgname>` - Deploy to AUR
   - `archforge tui` - Launch interactive TUI
5. Set up `tracing` for structured logging

**Acceptance criteria:**
- [ ] `cargo run -- --help` shows all subcommands
- [ ] Config file loads without errors
- [ ] Errors are well-formatted and user-friendly
- [ ] Tests pass: `cargo test -p archforge --lib`

---

### Phase 2: PKGBUILD Parser & Generator (Weeks 2-3)

**Goal:** Implement PKGBUILD v2.2 parsing and generation using nom combinators.

**Files to create/modify:**

- `/home/scqxd/ArchForge/build/Cargo.toml`
- `/home/scqxd/ArchForge/build/src/pkgbuild/mod.rs`
- `/home/scqxd/ArchForge/build/src/pkgbuild/parser.rs`
- `/home/scqxd/ArchForge/build/src/pkgbuild/generator.rs`

**Steps:**

1. Define PKGBUILD v2.2 struct using `serde`:
   ```rust
   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct Pkgbuild {
       pub pkgname: String,
       pub pkgver: String,
       pub pkgrel: u32,
       pub epoch: Option<u32>,
       pub pkgdesc: Option<String>,
       pub url: Option<String>,
       pub arch: Vec<String>,
       pub license: Vec<String>,
       pub groups: Vec<String>,
       pub depends: Vec<String>,
       pub makedepends: Vec<String>,
       pub checkdepends: Vec<String>,
       pub optdepends: Vec<String>,
       pub provides: Vec<String>,
       pub conflicts: Vec<String>,
       pub replaces: Vec<String>,
       pub backup: Vec<String>,
       pub options: Vec<BuildOption>,
       pub install: Option<String>,
       pub changelog: Option<String>,
       pub source: Vec<Source>,
       pub sha256sums: Vec<String>,
       pub validpgpkeys: Vec<String>,
       pub prepare: Vec<Function>,
       pub build: Vec<Function>,
       pub check: Vec<Function>,
       pub package: Vec<Function>,
   }

   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct Source {
       pub url: String,
       pub arch: Option<String>,  // Per-architecture source
   }

   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct Function {
       pub name: Option<String>,  // Anonymous if None
       pub body: String,
   }
   ```

2. Implement nom parser for PKGBUILD syntax:
   ```rust
   impl Pkgbuild {
       pub fn parse(content: &str) -> Result<Self, PkgbuildParseError> {
           let mut vars = HashMap::new();
           let mut parser = Parser::new(content);
           parser.parse_variables(&mut vars)?;
           parser.parse_functions(&mut vars)?;
           Ok(parser.pkgbuild)
       }
   }

   // nom combinators for variable assignment
   fn variable_assignment(input: &str) -> IResult<&str, (String, Value)> {
       separated_pair(
           identifier,
           tag("="),
           value_with_envvar_interpolation,
       )(input)
   }
   ```

3. Implement generator (struct -> PKGBUILD string):
   ```rust
   impl Pkgbuild {
       pub fn to_string(&self) -> String {
           let mut output = String::new();
           output.push_str(&format!("pkgname={}\n", self.pkgname));
           output.push_str(&format!("pkgver={}\n", self.pkgver));
           // ... handle all fields with proper escaping
           output
       }
   }
   ```

4. Implement v2.2 compliance validator:
   - Check required fields (`pkgname`, `pkgver`)
   - Validate variable interpolation syntax
   - Verify MD5/SHA256 arrays match source count
   - Check for deprecated fields

**Acceptance criteria:**
- [ ] Parser handles all v2.2 fields
- [ ] Generator produces valid PKGBUILD syntax
- [ ] Round-trip: parse → generate → parse produces same result
- [ ] 100+ unit tests for parser edge cases

---

### Phase 3: TUI Implementation - ARCHITECT Agent (Weeks 3-5)

**Goal:** Build the interactive TUI with Ratatui, vim keybindings, and component architecture.

**Files to create/modify:**

- `/home/scqxd/ArchForge/archforge/Cargo.toml` (update deps)
- `/home/scqxd/ArchForge/archforge/src/tui/mod.rs`
- `/home/scqxd/ArchForge/archforge/src/tui/app.rs`
- `/home/scqxd/ArchForge/archforge/src/tui/event.rs`
- `/home/scqxd/ArchForge/archforge/src/tui/key.rs`
- `/home/scqxd/ArchForge/archforge/src/tui/components/`

**Steps:**

1. Set up Ratatui and crossterm event loop:
   ```rust
   pub struct Tui {
       terminal: Terminal<CrosstermBackend<Stdout>>,
       events: EventHandler,
   }

   impl Tui {
       pub fn new() -> Result<Self> {
           let backend = CrosstermBackend::new(stdout());
           let terminal = Terminal::new(backend)?;
           let events = EventHandler::new(250);  // 250ms tick
           Ok(Self { terminal, events })
       }

       pub fn run(&mut self, app: &mut App) -> Result<()> {
           loop {
               self.terminal.draw(|frame| app.render(frame))?;
               if let Event::Key(key) = self.events.next()? {
                   app.handle_key(key);
               }
           }
       }
   }
   ```

2. Implement vim-like keybindings:
   ```rust
   #[derive(Debug, Clone, PartialEq)]
   pub enum Key {
       Normal, Insert, Visual,
       // Normal mode
       J, K, H, L,           // Movement
       I, A,                 // Enter insert mode
       C,                    // Change
       D,                    // Delete
       Y,                    // Yank
       P,                    // Paste
       :Wq, :Q, :Q!, :W,    // Command mode
       /,                    // Search
       // Insert mode
       Esc,                  // Return to normal
       CtrlW,                // Delete word
       CtrlU,                // Delete line
   }

   impl From<KeyCode> for Key {
       fn from(code: KeyCode) -> Self {
           match code {
               KeyCode::Char('j') => Self::J,
               KeyCode::Char('k') => Self::K,
               KeyCode::Char('i') => Self::I,
               KeyCode::Esc => Self::Esc,
               // ...
           }
       }
   }
   ```

3. Create component architecture:
   ```rust
   pub trait Component {
       fn render(&self, area: Rect, buf: &mut Buffer);
       fn handle_events(&mut self, event: Event) -> Option<Action>;
       fn focus(&mut self);
       fn unfocus(&mut self);
   }

   // Main layout components
   pub struct InputComponent {
       input: TextArea,
       focused: bool,
   }

   pub struct PreviewComponent {
       content: String,
       syntax_highlighter: SyntaxHighlighter,
   }

   pub struct PackageListComponent {
       packages: Vec<PackageInfo>,
       selected: usize,
   }

   pub struct StatusComponent {
       progress: BuildProgress,
       messages: Vec<StatusMessage>,
   }
   ```

4. Implement main app state with reducer pattern:
   ```rust
   pub struct App {
       state: AppState,
       actions: Vec<Action>,
       previous_states: Vec<AppState>,
   }

   #[derive(Debug, Clone, PartialEq)]
   pub enum AppState {
       Idle,
       Generating { prompt: String },
       Building { package: String, progress: f32 },
       Previewing { pkgbuild: Pkgbuild },
       Deploying { target: DeployTarget },
       Error(ArchforgeError),
   }

   impl Reducer for App {
       fn reduce(&mut self, action: Action) {
           match action {
               Action::Generate(prompt) => {
                   self.state = AppState::Generating { prompt };
                   // Queue AI_CORE task
               }
               Action::Build(package) => {
                   self.state = AppState::Building { package, progress: 0.0 };
                   // Queue BUILD_ENGINE task
               }
               // ...
           }
       }
   }
   ```

5. Create syntax highlighting for PKGBUILD:
   ```rust
   pub struct PkgbuildSyntax;

   impl Highlighter for PkgbuildSyntax {
       fn highlight(&self, line: &str) -> StyledText {
           let mut spans = Vec::new();
           // Highlight variables: $pkgname, ${pkgver}
           // Highlight functions: prepare(), build(), package()
           // Highlight comments: # This is a comment
           // Highlight strings: "https://..."
           StyledText::with_spans(spans)
       }
   }
   ```

6. Build split-pane layout:
   ```
   ┌─────────────────────────────────────────┐
   │ COMMAND BAR: [ archforge "собери..." ]  │
   ├──────────┬──────────────────────────────┤
   │          │                              │
   │ PACKAGE  │     PKGBUILD PREVIEW         │
   │  LIST    │                              │
   │          │  pkgname=firefox             │
   │  firefox │  pkgver=120.0                │
   │  vscodium│  depends=('libgtk-3')        │
   │  ...     │  ...                         │
   │          │                              │
   ├──────────┴──────────────────────────────┤
   │ STATUS: [✓] Ready | Model: Loaded |     │
   └─────────────────────────────────────────┘
   ```

**Acceptance criteria:**
- [ ] TUI renders without flicker
- [ ] Vim keybindings work (hjkl, i, :, /)
- [ ] Input field accepts natural language
- [ ] Preview panel shows generated PKGBUILD with syntax highlighting
- [ ] Build progress updates in real-time

---

### Phase 4: Local LLM Integration - AI_CORE Agent (Weeks 5-7)

**Goal:** Implement local LLM inference using candle-rs for PKGBUILD generation.

**Files to create/modify:**

- `/home/scqxd/ArchForge/core/Cargo.toml`
- `/home/scqxd/ArchForge/core/src/model/loader.rs`
- `/home/scqxd/ArchForge/core/src/model/engine.rs`
- `/home/scqxd/ArchForge/core/src/generation/generator.rs`

**Steps:**

1. Set up candle-rs with GGUF support:
   ```rust
   use candle_transformers::models::llama::{Llama, Config as LlamaConfig};
   use candle_core::{Device, Tensor, Result as CandleResult};
   use candle_gguf::GGUFReader;

   pub struct LocalModel {
       model: Llama,
       tokenizer: Tokenizer,
       device: Device,
       config: LlamaConfig,
   }

   impl LocalModel {
       pub fn load_gguf(path: impl AsRef<Path>) -> CandleResult<Self> {
           let reader = GGUFReader::new(path)?;
           let device = Device::Cpu;
           let config = LlamaConfig::from_gguf(&reader)?;
           let model = Llama::from_gguf(&reader, &device)?;
           let tokenizer = Tokenizer::from_gguf(&reader)?;
           Ok(Self { model, tokenizer, device, config })
       }

       pub fn generate(
           &mut self,
           prompt: &str,
           sampling: SamplingConfig,
       ) -> CandleResult<String> {
           let tokens = self.tokenizer.encode(prompt)?;
           let mut generated = Vec::new();
           let mut logits = self.model.forward(&tokens, 0)?;

           loop {
               let token = sample(&logits, &sampling)?;
               generated.push(token);
               logits = self.model.forward(&generated, generated.len() - 1)?;

               if token == self.tokenizer.eos_token_id().unwrap() {
                   break;
               }
           }
           Ok(self.tokenizer.decode(&generated)?)
       }
   }
   ```

2. Implement PKGBUILD-specific prompts:
   ```rust
   pub struct PkgbuildPrompt;

   impl PkgbuildPrompt {
       pub fn generate_system_prompt() -> String {
           r#"You are an expert Arch Linux package maintainer.
Your task is to create a valid PKGBUILD from natural language descriptions.

## Rules:
1. Always use PKGBUILD v2.2 format
2. Use makepkg-compatible syntax
3. Include proper dependency resolution
4. Add helpful comments for non-standard options
5. Use variable interpolation for重复 values

## Examples:

Input: "собери firefox с vaapi и u2f"
Output:
```bash
pkgname=firefox-vaapi
pkgver=120.0
pkgrel=1
pkgdesc="Firefox with VA-API and U2F support enabled"
arch=('x86_64')
url="https://www.mozilla.org/firefox/"
license=('MPL-2.0')
depends=('firefox' 'libva' 'libu2f-host')
makedepends=('rust' 'llvm' 'clang')
optdepends=('firefox-i18n-ru: Russian localization')
provides=('firefox')
conflicts=('firefox')
prepare() {
    # Enable VA-API hardware acceleration
    sed -i 's|--disable-eme| --enable-eme|' configure
    sed -i 's|#define MOZ_WAYLAND 0|#define MOZ_WAYLAND 1|'mozilla/config/gecko_dev_content.patch
}
```

## Response Format:
Provide ONLY the PKGBUILD content. No markdown, no explanations.
"#.to_string()
       }

       pub fn format_prompt(description: &str) -> String {
           format!(
               "{}\n\nInput: \"{}\"\n\nOutput:",
               Self::generate_system_prompt(),
               description
           )
       }
   }
   ```

3. Implement generation pipeline:
   ```rust
   pub struct PkgbuildGenerator {
       model: LocalModel,
       cache: GenerationCache,
   }

   impl PkgbuildGenerator {
       pub async fn generate(
           &mut self,
           description: &str,
       ) -> Result<Pkgbuild, GenerationError> {
           let prompt = PkgbuildPrompt::format_prompt(description);

           let raw_output = self.model.generate(
               &prompt,
               SamplingConfig {
                   temperature: 0.1,
                   top_p: 0.9,
                   top_k: 50,
                   max_tokens: Some(4096),
                   stop_tokens: vec![50256, 29, 198, 628],  // EOS variants
               },
           )?;

           let cleaned = self.postprocess_output(&raw_output);
           let pkgbuild = Pkgbuild::parse(&cleaned)
               .map_err(GenerationError::ParseError)?;

           self.validate(&pkgbuild)?;
           Ok(pkgbuild)
       }

       fn postprocess_output(&self, output: &str) -> String {
           // Remove markdown code blocks
           // Extract PKGBUILD content
           // Fix common LLM mistakes
           output
               .lines()
               .skip_while(|l| !l.starts_with("pkgname="))
               .take_while(|l| !l.trim_start().starts_with("```"))
               .collect::<Vec<_>>()
               .join("\n")
       }
   }
   ```

4. Implement model caching and hot-reload:
   ```rust
   pub struct ModelManager {
       cache_dir: PathBuf,
       current_model: Option<LocalModel>,
   }

   impl ModelManager {
       pub async fn load(&mut self, model_path: &str) -> Result<()> {
           let path = self.cache_dir.join(model_path);
           self.current_model = Some(LocalModel::load_gguf(&path)?);
           Ok(())
       }

       pub fn model(&self) -> Option<&LocalModel> {
           self.current_model.as_ref()
       }
   }
   ```

5. Create fallback for offline mode (rule-based generation):
   ```rust
   pub struct RuleBasedGenerator {
       patterns: Vec<DependencyPattern>,
   }

   impl RuleBasedGenerator {
       pub fn generate(&self, description: &str) -> Pkgbuild {
           let mut pkgbuild = Pkgbuild::default();
           pkgbuild.pkgname = self.extract_name(description);
           pkgbuild.depends = self.extract_deps(description);
           // ...
           pkgbuild
       }
   }
   ```

**Acceptance criteria:**
- [ ] GGUF models load without errors
- [ ] Generation completes in <30 seconds on CPU
- [ ] Output is valid PKGBUILD syntax
- [ ] Model cache persists across restarts
- [ ] Fallback generates when model unavailable

---

### Phase 5: Build Engine - BUILD_ENGINE Agent (Weeks 7-9)

**Goal:** Wrapper for makepkg/yay/paru, conflict detection, and dependency prediction.

**Files to create/modify:**

- `/home/scqxd/ArchForge/build/Cargo.toml`
- `/home/scqxd/ArchForge/build/src/builder/makepkg.rs`
- `/home/scqxd/ArchForge/build/src/builder/aur.rs`
- `/home/scqxd/ArchForge/build/src/conflict/predictor.rs`

**Steps:**

1. Implement makepkg wrapper:
   ```rust
   pub struct MakepkgRunner {
       makepkg_path: PathBuf,
       work_dir: PathBuf,
   }

   impl MakepkgRunner {
       pub async fn run(
           &self,
           config: BuildConfig,
       ) -> Result<BuildResult, BuildError> {
           let output = Command::new(&self.makepkg_path)
               .args(&[
                   "--noconfirm",
                   "--skippgpcheck",
                   "--nodeps",
                   "--skipchecksums",
               ])
               .current_dir(&self.work_dir)
               .output()
               .await?;

           if !output.status.success() {
               return Err(BuildError::MakepkgFailed(
                   String::from_utf8_lossy(&output.stderr).to_string(),
               ));
           }

           let artifacts = self.collect_artifacts()?;
           Ok(BuildResult { artifacts, logs: output.stdout })
       }

       pub async fn dry_run(&self) -> Result<(), BuildError> {
           let output = Command::new(&self.makepkg_path)
               .arg("--nobuild")
               .current_dir(&self.work_dir)
               .output()
               .await?;
           Ok(())
       }
   }
   ```

2. Implement AUR helper integration:
   ```rust
   pub struct AURHelper {
       helper: AURHelperType,
   }

   pub enum AURHelperType {
       Yay,
       Paru,
       Custom(PathBuf),
   }

   impl AURHelper {
       pub async fn query(&self, package: &str) -> Result<Option<PackageInfo>> {
           let output = match self.helper {
               AURHelperType::Yay => {
                   Command::new("yay")
                       .args(&["-Qua", "--json"])
                       .output()
                       .await?
               }
               AURHelperType::Paru => {
                   Command::new("paru")
                       .args(&["-Qua", "--json"])
                       .output()
                       .await?
               }
               // ...
           };
           parse_aur_output(&output)
       }

       pub async fn install(&self, package: &str) -> Result<InstallResult> {
           let mut child = match self.helper {
               AURHelperType::Paru => {
                   Command::new("paru")
                       .args(&["-S", "--noconfirm", "--needed", package])
                       .spawn()?
               }
               // ...
           };
           child.wait().await?;
           Ok(InstallResult::default())
       }
   }
   ```

3. Implement PKGBUILD parser with nom (detailed):
   ```rust
   pub struct PkgbuildParser<'a> {
       input: &'a str,
       position: usize,
   }

   impl<'a> PkgbuildParser<'a> {
       pub fn parse(content: &'a str) -> Result<Pkgbuild, ParseError> {
           let mut parser = Self {
               input: content,
               position: 0,
           };
           Ok(parser.parse_variables_and_functions()?)
       }

       fn parse_variables_and_functions(&mut self) -> Result<Pkgbuild> {
           let mut pkgbuild = Pkgbuild::default();

           while let Ok((_, (name, value))) = self.variable() {
               match name.as_str() {
                   "pkgname" => pkgbuild.pkgname = value.unwrap_string(),
                   "pkgver" => pkgbuild.pkgver = value.unwrap_string(),
                   "depends" => pkgbuild.depends = value.unwrap_array()?,
                   "makedepends" => pkgbuild.makedepends = value.unwrap_array()?,
                   _ => {}  // Handle other fields
               }
           }

           while let Ok((_, func)) = self.function() {
               match func.name {
                   Some(name) if name == "prepare" => pkgbuild.prepare.push(func),
                   Some(name) if name == "build" => pkgbuild.build.push(func),
                   Some(name) if name == "package" => pkgbuild.package.push(func),
                   _ => {}  // Anonymous functions
               }
           }

           Ok(pkgbuild)
       }

       fn variable(&mut self) -> IResult<&str, (String, Value)> {
           let (rest, (name, _, value)) = tuple((
               identifier,
               opt(whitespace),
               char('='),
               value_with_variables,
           ))(self.input)?;
           self.position = self.input.len() - rest.len();
           Ok((rest, (name, value)))
       }
   }
   ```

4. Implement ML-based conflict prediction:
   ```rust
   pub struct ConflictPredictor {
       model: PredictorModel,
       dependency_graph: DependencyGraph,
   }

   impl ConflictPredictor {
       pub async fn predict_conflicts(
           &self,
           pkgbuild: &Pkgbuild,
       ) -> Vec<Conflict> {
           let deps = self.resolve_all_dependencies(pkgbuild);

           let features = ConflictFeatures {
               direct_deps: deps.len(),
               circular_deps: self.detect_cycles(&deps),
               version_conflicts: self.check_versions(&deps),
               optional_conflicts: self.analyze_optional(&pkgbuild),
           };

           let predictions = self.model.predict(features).await?;

           predictions
               .iter()
               .filter(|p| p.probability > 0.3)
               .map(|p| Conflict {
                   packages: p.conflicting_packages.clone(),
                   probability: p.probability,
                   reason: p.reason.clone(),
                   suggestion: p.suggestion.clone(),
               })
               .collect()
       }
   }

   #[derive(Debug, Clone)]
   pub struct Conflict {
       pub packages: Vec<String>,
       pub probability: f32,
       pub reason: String,
       pub suggestion: String,
   }
   ```

5. Implement real-time build log streaming:
   ```rust
   pub struct BuildLogStreamer {
       tx: mpsc::Sender<LogLine>,
       rx: mpsc::Receiver<LogLine>,
   }

   impl BuildLogStreamer {
       pub fn stream_logs(&self, path: impl AsRef<Path>) {
           let mut file = File::open(path).unwrap();
           let mut buffer = String::new();

           loop {
               buffer.clear();
               if let Ok(bytes) = file.read_line(&mut buffer) {
                   if bytes == 0 { break; }
                   self.tx.blocking_send(LogLine {
                       content: buffer.clone(),
                       timestamp: Utc::now(),
                       level: parse_log_level(&buffer),
                   }).unwrap();
               }
           }
       }
   }
   ```

**Acceptance criteria:**
- [ ] makepkg builds packages successfully
- [ ] AUR helper integration works for search/install
- [ ] Parser handles edge cases (arrays, multiline functions)
- [ ] Conflict prediction reduces build failures
- [ ] Logs stream in real-time to TUI

---

### Phase 6: P2P Network - SWARM_AGENT (Weeks 9-10)

**Goal:** libp2p-based telemetry sharing and local caching.

**Files to create/modify:**

- `/home/scqxd/ArchForge/swarm/Cargo.toml`
- `/home/scqxd/ArchForge/swarm/src/network/behaviour.rs`
- `/home/scqxd/ArchForge/swarm/src/network/discovery.rs`
- `/home/scqxd/ArchForge/swarm/src/storage/db.rs`
- `/home/scqxd/ArchForge/swarm/src/sync/gossipsub.rs`

**Steps:**

1. Set up libp2p network stack:
   ```rust
   use libp2p::{Swarm,identify, kademlia, mdns, noise, tcp, yamux, quic, Multiaddr, PeerId};
   use libp2p::gossipsub::{Gossipsub, Message, TopicHash};

   pub struct SwarmNetwork {
       swarm: Swarm<ArchForgeBehaviour>,
       local_peer_id: PeerId,
   }

   impl SwarmNetwork {
       pub fn new() -> Result<Self> {
           let local_key = identity::Keypair::generate_ed25519();
           let local_peer_id = PeerId::from(local_key.public());

           let transport = tcp::tokio::Transport::new(tcp::Config::default())
               .upgrade(upgrade::select(quic::Version::V1))
               .authenticate(noise::Config::new(&local_key)?)
               .multiplex(yamux::Config::default())
               .boxed();

           let behaviour = ArchForgeBehaviour {
               identify: identify::Behaviour::new(identify::Config::new(
                   "archforge/1.0".to_string(),
                   local_key.public(),
               )),
               kademlia: kademlia::Behaviour::new(local_peer_id, memory_store::MemoryStore::new(local_peer_id)),
               mdns: mdns::tokio::Behaviour::new(mdns::Config::default(), local_peer_id)?,
               gossipsub: Gossipsub::new(
                   MessageAuthenticity::Signed(local_key.clone()),
                   gossipsub::Config::default(),
               )?,
           };

           let mut swarm = Swarm::new(transport, behaviour, local_peer_id);
           swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

           Ok(Self { swarm, local_peer_id })
       }
   }
   ```

2. Implement custom behaviour for build telemetry:
   ```rust
   #[derive(NetworkBehaviour)]
   #[behaviour(out_event = "NetworkEvent")]
   pub struct ArchForgeBehaviour {
       pub identify: identify::Behaviour,
       pub kademlia: kademlia::Behaviour<MemoryStore>,
       pub mdns: mdns::tokio::Behaviour,
       pub gossipsub: Gossipsub,
   }

   #[derive(Debug, Clone)]
   pub enum NetworkEvent {
       Identify(identify::Event),
       Kademlia(kademlia::Event),
       Mdns(mdns::Event),
       Gossipsub(gossipsub::Event),
   }
   ```

3. Implement peer discovery:
   ```rust
   pub struct PeerDiscovery {
       swarm: Arc<Swarm<ArchForgeBehaviour>>,
       known_peers: HashMap<PeerId, PeerInfo>,
   }

   impl PeerDiscovery {
       pub async fn discover(&mut self) -> Result<Vec<PeerId>> {
           let mut new_peers = Vec::new();

           for event in self.swarm.behaviour_mut().mdns.discover() {
               match event {
                   mdns::Event::Discovered(peers) => {
                       for (peer, addr) in peers {
                           self.swarm.dial(addr)?;
                           new_peers.push(peer);
                       }
                   }
                   _ => {}
               }
           }
           Ok(new_peers)
       }

       pub fn bootstrap_kademlia(&mut self) {
           self.swarm.behaviour_mut().kademlia.bootstrap();
       }
   }
   ```

4. Implement sled database for telemetry:
   ```rust
   use sled::Db;

   pub struct TelemetryDb {
       db: Db,
   }

   impl TelemetryDb {
       pub fn new(path: impl AsRef<Path>) -> Result<Self> {
           let config = Config::default()
               .path(path)
               .cache_capacity(1024 * 1024 * 100);  // 100MB cache
           let db = sled::Db::start(config)?;
           Ok(Self { db })
       }

       pub async fn record_build(
           &self,
           pkg_name: &str,
           build: BuildTelemetry,
       ) -> Result<()> {
           let key = format!("build:{}:{}", pkg_name, build.timestamp);
           self.db.insert(key.as_bytes(), bincode::serialize(&build)?)?;
           Ok(())
       }

       pub async fn get_build_history(
           &self,
           pkg_name: &str,
       ) -> Result<Vec<BuildTelemetry>> {
           let prefix = format!("build:{}:", pkg_name);
           let builds: Vec<BuildTelemetry> = self.db
               .scan_prefix(prefix.as_bytes())
               .values()
               .map(|v| bincode::deserialize(&v))
               .collect::<Result<Vec<_>, _>>()?;
           Ok(builds)
       }

       pub async fn record_conflict(
           &self,
           conflict: ConflictRecord,
       ) -> Result<()> {
           let key = format!("conflict:{}", conflict.id);
           self.db.insert(key.as_bytes(), bincode::serialize(&conflict)?)?;
           Ok(())
       }
   }

   #[derive(Serialize, Deserialize, Debug)]
   pub struct BuildTelemetry {
       pub pkg_name: String,
       pub pkg_version: String,
       pub arch: String,
       pub makepkg_version: String,
       pub build_time_ms: u64,
       pub success: bool,
       pub error_type: Option<String>,
       pub system_info: SystemInfo,
       pub timestamp: chrono::DateTime<Utc>,
   }

   #[derive(Serialize, Deserialize, Debug)]
   pub struct SystemInfo {
       pub cpu: String,
       pub ram_gb: f32,
       pub kernel: String,
       #[serde(default)]
       pub gpu: Vec<String>,
   }
   ```

5. Implement GossipSub for telemetry broadcasting:
   ```rust
   pub struct TelemetryGossip {
       topic: Topic,
       swarm: Arc<Swarm<ArchForgeBehaviour>>,
   }

   impl TelemetryGossip {
       pub fn new(swarm: Arc<Swarm<ArchForgeBehaviour>>) -> Result<Self> {
           let topic = gossipsub::IdentTopic::new("archforge-build-telemetry");
           swarm.behaviour_mut().gossipsub.subscribe(&topic)?;

           Ok(Self { topic, swarm })
       }

       pub fn publish(&mut self, telemetry: BuildTelemetry) {
           let message = bincode::serialize(&telemetry).unwrap();
           if let Err(e) = self.swarm
               .behaviour_mut()
               .gossipsub
               .publish(self.topic.clone(), message)
           {
               tracing::error!("Failed to publish telemetry: {:?}", e);
           }
       }

       pub fn subscribe<F>(&mut self, callback: F)
       where
           F: Fn(BuildTelemetry) + Send + 'static,
       {
           let mut stream = self.swarm.behaviour_mut().gossipsub
               .subscribe(&self.topic)
               .unwrap()
               .map(|msg| bincode::deserialize::<BuildTelemetry>(&msg.data).unwrap());

           tokio::spawn(async move {
               while let Some(telemetry) = stream.next().await {
                   callback(telemetry);
               }
           });
       }
   }
   ```

6. Implement peer sync protocol:
   ```rust
   pub struct PeerSync {
       db: TelemetryDb,
       network: SwarmNetwork,
   }

   impl PeerSync {
       pub async fn sync_with_peer(&self, peer: &PeerId) -> Result<SyncResult> {
           let known_hashes = self.db.get_known_hashes().await?;

           let request = SyncRequest {
               peer_id: self.network.local_peer_id(),
               known_hashes,
               request_type: SyncRequestType::MissingAndNewer,
           };

           let response: SyncResponse = self.network
               .send_request(peer, "archforge/sync/v1", request)
               .await?;

           self.apply_sync(response).await
       }

       async fn apply_sync(&self, response: SyncResponse) -> Result<SyncResult> {
           let mut applied = 0;
           for telemetry in response.new_telemetry {
               self.db.record_build(&telemetry.pkg_name, &telemetry)?;
               applied += 1;
           }
           Ok(SyncResult { applied })
       }
   }
   ```

**Acceptance criteria:**
- [ ] Peers discover each other on local network
- [ ] GossipSub broadcasts telemetry
- [ ] sled DB stores and retrieves telemetry
- [ ] Sync protocol works for missing data
- [ ] Network is encrypted and authenticated

---

### Phase 7: Deployment - DEPLOYER Agent (Weeks 10-12)

**Goal:** AUR upload, Docker/Flatpak/Nix deployment, VCS integration.

**Files to create/modify:**

- `/home/scqxd/ArchForge/deploy/Cargo.toml`
- `/home/scqxd/ArchForge/deploy/src/aur/uploader.rs`
- `/home/scqxd/ArchForge/deploy/src/container/docker.rs`
- `/home/scqxd/ArchForge/deploy/src/container/flatpak.rs`
- `/home/scqxd/ArchForge/deploy/src/vcs/git.rs`

**Steps:**

1. Implement AUR uploader:
   ```rust
   pub struct AURUploader {
       username: String,
       session: AURSession,
   }

   impl AURUploader {
       pub async fn login(&mut self, password: &str) -> Result<()> {
           // Get AUR session cookie
           let response = self.session
               .post("https://aur.archlinux.org/account")
               .form(&[("user", &self.username), ("passwd", password)])
               .send()
               .await?;

           if !response.status().is_success() {
               return Err(AURError::LoginFailed);
           }
           Ok(())
       }

       pub async fn upload_package(
           &self,
           pkgbuild: &Pkgbuild,
           files: Vec<UploadedFile>,
       ) -> Result<AURPackage> {
           // Create tarball
           let tarball = self.create_tarball(pkgbuild, &files)?;

           // Upload to AUR
           let response = self.session
               .post("https://aur.archlinux.org/pkgupload")
               .multipart(form::Multipart::form())
               .file("pkgbuild", &tarball)?
               .send()
               .await?;

           // Parse response and return package info
           Ok(self.parse_upload_response(response).await?)
       }

       fn create_tarball(
           &self,
           pkgbuild: &Pkgbuild,
           files: &[UploadedFile],
       ) -> Result<NamedTempFile> {
           let mut archive = tar::Builder::new(Vec::new());

           // Add PKGBUILD
           let pkgbuild_content = pkgbuild.to_string();
           let header = tar::Header::new_gnu();
           header.set_size(pkgbuild_content.len() as u64);
           header.set_cksum();
           archive.append_data(&header, "PKGBUILD", pkgbuild_content.as_bytes())?;

           // Add source files
           for file in files {
               let mut file_content = std::fs::File::open(&file.path)?;
               archive.append_file(&file.name, &mut file_content)?;
           }

           // Finish and write
           let content = archive.into_inner()?;
           let mut temp_file = NamedTempFile::new()?;
           temp_file.write_all(&content)?;
           Ok(temp_file)
       }
   }
   ```

2. Implement Docker build support:
   ```rust
   pub struct DockerBuilder {
       docker_path: PathBuf,
   }

   impl DockerBuilder {
       pub async fn build(
           &self,
           pkgbuild: &Pkgbuild,
           context: &Path,
       ) -> Result<DockerBuildResult> {
           // Generate Dockerfile from PKGBUILD
           let dockerfile = self.generate_dockerfile(pkgbuild)?;
           let dockerfile_path = context.join("Dockerfile");
           std::fs::write(&dockerfile_path, dockerfile)?;

           // Build image
           let output = Command::new(&self.docker_path)
               .args(&[
                   "build",
                   "--tag", &format!("archforge/{}", pkgbuild.pkgname),
                   "--file", &dockerfile_path.to_string_lossy(),
                   context.to_string_lossy().as_ref(),
               ])
               .output()
               .await?;

           if !output.status.success() {
               return Err(DockerError::BuildFailed(
                   String::from_utf8_lossy(&output.stderr).to_string(),
               ));
           }

           Ok(DockerBuildResult {
               image_id: self.parse_image_id(&output.stdout),
           })
       }

       fn generate_dockerfile(&self, pkgbuild: &Pkgbuild) -> Result<String> {
           Ok(format!(r#"FROM archlinux:latest

# Install build dependencies
RUN pacman -Syu --noconfirm \
    base-devel \
    {}

# Copy PKGBUILD
COPY PKGBUILD /tmp/PKGBUILD
WORKDIR /tmp

# Build package
RUN makepkg -s --noconfirm

# Install the package
RUN pacman -U --noconfirm *.pkg.tar.zst

# Clean up
RUN rm -rf /tmp/*

CMD ["{}"]
"#, pkgbuild.makedepends.join(" \\\n    "), pkgbuild.pkgname))
       }
   }
   ```

3. Implement Flatpak bundle with bubblewrap:
   ```rust
   pub struct FlatpakBuilder {
       bwrap_path: PathBuf,
       flatpak_path: PathBuf,
   }

   impl FlatpakBuilder {
       pub async fn create_bundle(
           &self,
           pkgbuild: &Pkgbuild,
           sandbox_config: SandboxConfig,
       ) -> Result<FlatpakBundle> {
           let bundle_dir = std::env::temp_dir()
               .join(format!("archforge-{}", pkgbuild.pkgname));

           // Build in sandboxed environment
           let result = self.run_in_sandbox(pkgbuild, &bundle_dir, sandbox_config)
               .await?;

           // Export to flatpak bundle
           let bundle_path = bundle_dir.with_extension("flatpak");
           Command::new(&self.flatpak_path)
               .args(&[
                   "build-export",
                   "--repo", &bundle_dir.join("repo"),
                   &bundle_dir,
                   "stable",
               ])
               .output()
               .await?;

           Ok(FlatpakBundle { path: bundle_path })
       }

       async fn run_in_sandbox(
           &self,
           pkgbuild: &Pkgbuild,
           work_dir: &Path,
           config: SandboxConfig,
       ) -> Result<SandboxResult> {
           let mut cmd = Command::new(&self.bwrap_path);

           // Setup bubblewrap sandbox
           cmd.args(&[
               "--unshare-user",
               "--unshare-pid",
               "--unshare-net",
               "--ro-bind", "/usr", "/usr",
               "--bind", work_dir, "/build",
               "--tmpfs", "/tmp",
           ]);

           // Add capabilities
           if config.allow_gpu {
               cmd.args(&["--bind", "/dev/dri", "/dev/dri"]);
           }

           // Add environment
           cmd.env("HOME", "/build");

           // Run makepkg
           cmd.args(&["/bin/bash", "-c", "cd /build && makepkg"]);

           let output = cmd.output().await?;
           Ok(SandboxResult { output })
       }
   }
   ```

4. Implement Nix flake generation:
   ```rust
   pub struct NixFlakeGenerator;

   impl NixFlakeGenerator {
       pub fn generate_flake(&self, pkgbuild: &Pkgbuild) -> String {
           format!(r#"{{
  description = "{} - Generated by ArchForge";
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  inputs.flake-utils.url = "github:numtide/flake-utils";

  outputs = {{ self, nixpkgs, flake-utils }}:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {{ inherit system; }};
      in {{
        packages.{name} = pkgs.callPackage ({pkgs}:
          pkgs.stdenv.mkDerivation {{
            pname = "{name}";
            version = "{ver}";
            src = fetchFromGitHub {{
              owner = "{owner}";
              repo = "{repo}";
              rev = "{rev}";
              sha256 = "{sha256}";
            }};
            nativeBuildInputs = with pkgs; [{makedepends}];
            buildInputs = with pkgs; [{depends}];
          }} {{}};
      }};
    }};
}}
"#.to_string())
       }
   }
   ```

5. Implement VCS integration:
   ```rust
   pub struct VCSManager;

   impl VCSManager {
       pub async fn init_git_repo(
           &self,
           pkgbuild: &Pkgbuild,
           path: &Path,
       ) -> Result<GitRepo> {
           // Initialize repo
           let repo = git2::Repository::init(path)?;

           // Add PKGBUILD
           let mut index = repo.index()?;
           let _ = index.add_path(Path::new("PKGBUILD"))?;
           index.write()?;

           // Initial commit
           let signature = repo.signature()?;
           let tree_id = index.write_tree()?;
           let tree = repo.find_tree(tree_id)?;

           repo.commit(
               Some("HEAD"),
               &signature,
               &signature,
               &format!("Initial PKGBUILD for {}", pkgbuild.pkgname),
               &tree,
               &[],
           )?;

           Ok(GitRepo { repo })
       }

       pub async fn create_aur_remote(
           &self,
           repo: &GitRepo,
           aur_url: &str,
       ) -> Result<()> {
           let mut remote = repo.remote("aur", aur_url)?;
           remote.connect(git2::Direction::Push)?;
           remote.push(&["refs/heads/master:refs/heads/master"], None)?;
           Ok(())
       }
   }
   ```

**Acceptance criteria:**
- [ ] AUR upload works with session authentication
- [ ] Docker builds produce working images
- [ ] Flatpak sandbox runs builds safely
- [ ] Nix flake generates valid configuration
- [ ] Git remote connects to AUR

---

### Phase 8: Polish & Distribution (Weeks 12-13)

**Goal:** CI/CD, AUR packaging, examples, documentation.

**Files to create/modify:**

- `/home/scqxd/ArchForge/.github/workflows/ci.yml`
- `/home/scqxd/ArchForge/PKGBUILD`
- `/home/scqxd/ArchForge/examples/*.rs`
- `/home/scqxd/ArchForge/README.md`

**Steps:**

1. Set up CI pipeline:
   ```yaml
   # .github/workflows/ci.yml
   name: CI
   on: [push, pull_request]

   jobs:
     test:
       runs-on: ubuntu-latest
       steps:
         - uses: actions/checkout@v4
         - uses: dtolnay/rust-toolchain@stable
         - run: cargo test --all --all-features
         - run: cargo clippy --all --all-features -- -D warnings

     binary-build:
       runs-on: archlinux:latest
       steps:
         - uses: actions/checkout@v4
         - run: cargo build --release --target x86_64-unknown-linux-musl
         - uses: actions/upload-artifact@v4
           with:
             name: archforge-x86_64
             path: target/release/archforge

     integration-test:
       needs: binary-build
       runs-on: archlinux:latest
       steps:
         - uses: actions/checkout@v4
         - uses: actions/download-artifact@v4
           with:
             name: archforge-x86_64
         - run: chmod +x archforge
         - run: ./archforge --version
   ```

2. Create AUR PKGBUILD:
   ```bash
   # PKGBUILD
   pkgname=archforge
   pkgver=0.1.0
   pkgrel=1
   pkgdesc="AI-powered TUI for PKGBUILD generation and AUR management"
   arch=('x86_64')
   url="https://github.com/username/archforge"
   license=('AGPL-3.0')
   depends=('rust' 'crossterm' 'tokio')
   makedepends=('cargo' 'pkgconf')
   source=("$url/archive/v$pkgver.tar.gz")
   sha256sums=('...')

   build() {{
       cd "$pkgname-$pkgver"
       cargo build --release --target x86_64-unknown-linux-musl
   }}

   package() {{
       cd "$pkgname-$pkgver"
       install -Dm755 target/release/archforge "$pkgdir/usr/bin/archforge"
       install -Dm644 "archforge.desktop" "$pkgdir/usr/share/applications/"
   }}
   ```

3. Create examples with magical demos:
   ```rust
   // examples/firefox_vaapi.rs
   // Run: cargo run --example firefox_vaapi

   #[tokio::main]
   async fn main() -> Result<()> {
       let config = Config::load()?;
       let mut generator = PkgbuildGenerator::new(config)?;

       // This is the magical part
       let prompt = "собери firefox с vaapi и u2f";
       println!("Generating PKGBUILD for: {}", prompt);

       let pkgbuild = generator.generate(prompt).await?;
       println!("{}", pkgbuild.to_string());

       // Preview
       println!("\n=== Build Preview ===");
       println!("Package: {}", pkgbuild.pkgname);
       println!("Dependencies: {:?}", pkgbuild.depends);
       println!("Optional deps: {:?}", pkgbuild.optdepends);

       Ok(())
   }
   ```

4. Write comprehensive README with demo GIFs

**Acceptance criteria:**
- [ ] CI passes all tests
- [ ] AUR package builds successfully
- [ ] Examples run without errors
- [ ] Documentation is complete
- [ ] Demo feels "magical"

---

## Dependency Management

### Root Cargo.toml

```toml
[workspace]
members = [
    "archforge",
    "core",
    "build",
    "swarm",
    "deploy",
]

[workspace.package]
version = "0.1.0"
edition = "2021"
rust-version = "1.75"

[workspace.dependencies]
tokio = { version = "1.35", features = ["full"] }
thiserror = "1.0"
anyhow = "1.0"
tracing = "0.1"
tracing-subscriber = "0.3"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "0.8"
clap = { version = "4.4", features = ["derive", "cargo"] }
```

### archforge/Cargo.toml (TUI + CLI)

```toml
[package]
name = "archforge"
version.workspace = true
edition.workspace = true

[dependencies]
ratatui = "0.25"
crossterm = "0.27"
cursive = "0.20"  # Optional alternative UI
anyhow.workspace = true
clap.workspace = true
tokio = { workspace = true, features = ["sync"] }
tracing.workspace = true

[dev-dependencies]
assert_cmd = "2.0"
predicates = "3.0"
```

### core/Cargo.toml (AI/LLM)

```toml
[package]
name = "archforge-core"
version.workspace = true
edition.workspace = true

[dependencies]
candle-core = "0.4"
candle-nn = "0.4"
candle-transformers = "0.4"
candle-gguf = "0.2"
tokenizers = "0.15"
reqwest = { version = "0.11", features = ["json"] }
tokio = { workspace = true, features = ["full"] }
serde.workspace = true
serde_json.workspace = true
anyhow.workspace = true
tracing.workspace = true

[target.'cfg(not(target_arch = "wasm"))'.dependencies]
candle-core = { version = "0.4", features = ["cuda"] }
```

### build/Cargo.toml (makepkg wrapper)

```toml
[package]
name = "archforge-build"
version.workspace = true
edition.workspace = true

[dependencies]
nom = "7"
anyhow.workspace = true
tokio = { workspace = true, features = ["process", "sync", "fs"] }
serde.workspace = true
serde_json.workspace = true
tracing.workspace = true
dialoguer = "0.11"

[dependencies]
# ML for conflict prediction (optional)
candle-core = { version = "0.4", optional = true }
tch = { version = "0.12", optional = true }

[features]
ml = ["candle-core", "tch"]
```

### swarm/Cargo.toml (P2P)

```toml
[package]
name = "archforge-swarm"
version.workspace = true
edition.workspace = true

[dependencies]
libp2p = { version = "0.53", features = [
    "quic",
    "tokio",
    "dns",
    "tcp",
    "noise",
    "yamux",
    "gossipsub",
    "mdns",
    "kademlia",
    "identify",
] }
sled = "1.0"
bincode = "1.3"
anyhow.workspace = true
tokio = { workspace = true, features = ["sync", "net"] }
serde.workspace = true
tracing.workspace = true

[dev-dependencies]
libp2p = { version = "0.53", features = ["test-utils"] }
```

### deploy/Cargo.toml (Deployment)

```toml
[package]
name = "archforge-deploy"
version.workspace = true
edition.workspace = true

[dependencies]
anyhow.workspace = true
tokio = { workspace = true, features = ["process"] }
serde.workspace = true
reqwest = { version = "0.11", features = ["multipart"] }
git2 = "0.18"
tar = "0.4"
xz2 = "0.7"
ureq = "2.9"

[target.'cfg(unix)'.dependencies]
nix = "0.28"
libc = "0.2"
```

---

## Risk Analysis

| Risk | Severity | Probability | Impact | Mitigation |
|------|----------|-------------|--------|------------|
| Local LLM too slow for TUI | High | Medium | User frustration | Use small quantized models (4-bit), progressive streaming, fallback to rule-based |
| PKGBUILD generation quality | High | Medium | Broken builds | Prompt engineering, validation layer, human review UI |
| AUR authentication complexity | Medium | Low | Upload failures | Session caching, clear error messages, 2FA support |
| libp2p networking complexity | Medium | Medium | P2P features fail | mDNS fallback, local-only by default, manual peer add |
| sled DB performance at scale | Low | Low | Slow startup | Compression, TTL for old records, async writes |
| candle-rs compatibility issues | Medium | Low | Model loading fails | Test multiple model formats, fallback to ONNX |
| AUR API rate limits | Low | High | Search timeouts | Caching, rate limit handling, exponential backoff |
| Bubblewrap availability | Low | Medium | Flatpak deploy fails | Check for bwrap binary, error with alternatives |

### Contingency Plans

1. **If LLM integration proves too slow**: Implement streaming token-by-token output, use 3B parameter models instead of 7B, add "quick mode" that skips generation
2. **If libp2p is too complex**: Simplify to HTTP-based peer sync, use existing AUR infrastructure
3. **If candle-rs has issues**: Fall back to ONNX Runtime via `ort` crate

---

## ML/AI Strategy

### Model Selection

| Model | Size | Quantization | Expected Use |
|-------|------|--------------|--------------|
| TinyLlama 1.1B | 700MB | 4-bit GGUF | Fast, local-first |
| Phi-3-mini 3.8B | 2.5GB | 4-bit GGUF | Better quality |
| OpenHermes 7B | 4GB | 4-bit GGUF | Best quality (if hardware allows) |

**Recommendation:** Start with TinyLlama for portability, support Phi-3 for better results.

### Training Approach

1. **Data Collection**:
   - Scrape AUR for PKGBUILD examples (10,000+ packages)
   - Filter for well-maintained packages
   - Include Russian/English descriptions paired with PKGBUILDs

2. **Fine-tuning**:
   - Use QLoRA on quantized base model
   - Train on (description, PKGBUILD) pairs
   - 3 epochs, learning rate 2e-4

3. **Validation**:
   - Test on held-out PKGBUILDs
   - Check v2.2 compliance
   - Verify build success

4. **Distribution**:
   - Pre-convert models to GGUF
   - Provide download via AUR or GitHub releases
   - Cache in `~/.cache/archforge/models/`

### Rule-Based Fallback

When model is unavailable:
```rust
pub fn rule_based_generate(description: &str) -> Pkgbuild {
    let mut pkg = Pkgbuild::default();
    pkg.pkgname = extract_package_name(description);
    pkg.depends = common_dependencies(description);
    pkg.optdepends = optional_dependencies(description);
    pkg.makedepends = standard_makedeps();
    pkg.build = standard_build_script();
    pkg
}
```

---

## Testing Strategy

### Unit Tests

```rust
// tests/unit/pkgbuild_parser_test.rs

#[test]
fn test_parse_simple_pkgbuild() {
    let input = r#"
pkgname="test-package"
pkgver="1.0.0"
pkgrel="1"
depends=('glibc' 'gcc-libs')
"#;
    let pkgbuild = Pkgbuild::parse(input).unwrap();
    assert_eq!(pkgbuild.pkgname, "test-package");
    assert_eq!(pkgbuild.pkgver, "1.0.0");
}

#[test]
fn test_parse_array_with_variables() {
    let input = r#"
depends=("${pkgname}-core" 'glibc')
"#;
    let pkgbuild = Pkgbuild::parse(input).unwrap();
    assert!(pkgbuild.depends[0].contains("test-package-core"));
}
```

### Integration Tests

```rust
// tests/integration/test_aur_flow.rs

#[tokio::test]
async fn test_aur_package_flow() {
    let config = Config::test();
    let generator = PkgbuildGenerator::new(config).await.unwrap();

    let pkgbuild = generator
        .generate("собери hello с русской локалью")
        .await
        .unwrap();

    assert!(!pkgbuild.pkgname.is_empty());
    assert!(!pkgbuild.pkgdesc.is_empty());

    // Validate PKGBUILD syntax
    validator::validate(&pkgbuild).unwrap();
}
```

### Testing Pyramid

```
        /\
       /  \      E2E Tests (10%)
      /    \     - Full user flows
     /------\    - AUR upload
    /        \
   /  Unit    \  Integration Tests (30%)
  /   Tests    \ - Component interactions
 /    (60%)    \- API calls, DB operations
/--------------\
```

### CI/CD Pipeline

```
┌─────────────────────────────────────────────────────┐
│ GitHub Actions CI Pipeline                          │
├─────────────────────────────────────────────────────┤
│ 1. Lint: cargo clippy + cargo fmt                  │
│ 2. Test: cargo test --all --all-features           │
│ 3. Build: cargo build --release --target x86_64... │
│ 4. Integration: Run full AUR flow in Docker        │
│ 5. Benchmark: cargo bench                          │
│ 6. Security: cargo audit                           │
├─────────────────────────────────────────────────────┤
│ Success → Artifacts → Release                       │
│ Failure → Notify → Halt                            │
└─────────────────────────────────────────────────────┘
```

### Test Coverage Targets

- Unit tests: 80% line coverage
- Integration tests: All critical paths
- Fuzz testing: PKGBUILD parser

---

## Demo Workflows

### Demo 1: Firefox with VAAPI + U2F
```
$ archforge "собери firefox с vaapi и u2f"

✓ Parsing request...
✓ Consulting local model (TinyLlama)...
✓ Generated PKGBUILD in 12.3s

┌─────────────────────────────────────────────┐
│ pkgname=firefox-vaapi                       │
│ pkgver=120.0                                │
│ pkgdesc="Firefox with VA-API and U2F..."    │
│ depends=('firefox' 'libva' 'libu2f-host')   │
│ provides=('firefox')                        │
│ conflicts=('firefox')                       │
│ prepare() {                                 │
│   sed -i 's|#define MOZ_WAYLAND 0|...       │
│ }                                           │
└─────────────────────────────────────────────┘

Actions: [b]uild, [e]dit, [u]pload to AUR, [d]ockerize
```

### Demo 2: Discord (Electron app)
```
$ archforge "дискорд для игры с русским языком"

✓ Generating...
✓ Resolved dependencies: electron, libatomic

[PKGBUILD Preview]
Actions: [b]uild, [e]dit, [p]ush to AUR
```

### Demo 3: VSCode (VSCodium)
```
$ archforge "vscodium с русским интерфейсом"

✓ Generated: vscodium-ru
✓ Installed: AUR package ready
```

### Demo 4: Custom Kernel with BFS
```
$ archforge "linux-bfs игровой ядро с bfs планировщиком"

✓ Generated PKGBUILD for linux-bfs
✓ Added bfs-sched patch
✓ Configured with gaming optimizations
```

### Demo 5: Full TUI Session (Interactive)
```
$ archforge tui

[ Welcome to ArchForge ]
┌─────────────────────────────────────────┐
│ > _                                      │
├──────────┬──────────────────────────────┤
│ Search   │ [Preview Panel]              │
│ firefox  │                              │
│ vscodium │ pkgname=firefox...            │
│ discord  │                              │
│ ...      │                              │
├──────────┴──────────────────────────────┤
│ :gen "собери firefox с vaapi"           │
│ [Status: Ready | Model: Loaded]         │
└─────────────────────────────────────────┘

(typing ":gen " + prompt...")
```

---

## Estimated Complexity

| Phase | Duration | Complexity | Key Challenges |
|-------|----------|------------|----------------|
| Phase 1: Infrastructure | 2 weeks | Low | Config loading, error handling |
| Phase 2: PKGBUILD Parser | 2 weeks | Medium | nom combinators, edge cases |
| Phase 3: TUI | 3 weeks | High | State management, async events |
| Phase 4: AI/LLM | 3 weeks | High | Model loading, prompt engineering |
| Phase 5: Build Engine | 3 weeks | Medium | External process handling, ML |
| Phase 6: P2P Network | 2 weeks | Very High | libp2p complexity |
| Phase 7: Deployer | 3 weeks | Medium | AUR API, containerization |
| Phase 8: Polish | 2 weeks | Low | CI/CD, docs, examples |

**Total estimated: 20 weeks (5 months)**

---

## Next Steps

1. **Week 1**: Set up workspace, implement Phase 1 (infrastructure)
2. **Week 2-3**: Phase 2 (PKGBUILD parser)
3. **Week 4-6**: Phase 3 (TUI)
4. **Week 7-9**: Phase 4 (AI/LLM integration)
5. **Week 10-12**: Phase 5-6 (Build Engine + P2P)
6. **Week 13-14**: Phase 7 (Deployer)
7. **Week 15**: Phase 8 (Polish)

**Start command:** `cargo new --name archforge archforge && cd archforge`