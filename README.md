# Shaft

Universal zero-click, offline, hardware-accelerated drag-and-drop file converter, contextual media tool, and quick action wheel engineered in pure safe Rust and Tauri 2.0.

---

## 1. Architectural Overview

Shaft is a standalone, commercial-grade desktop utility that eliminates conversion friction by transforming file interactions into instantaneous, zero-click radial gestures. Built from first principles in safe Rust (`#![deny(unsafe_code)]`), Shaft operates 100% offline with zero cloud telemetry, intermediate disk leaks, or external API dependencies.

```
+-------------------------------------------------------------------------------+
|                            OS Drag Event Pipeline                             |
|  - System DragEnter / DragOver / DragLeave / Drop                             |
+-------------------------------------------------------------------------------+
                                       │
                                       ▼
+-------------------------------------------------------------------------------+
|                       React 19 Drag & Drop Coordinator                        |
|                                                                               |
|   +------------------------------------+   +------------------------------+   |
|   |         DragGesturePill            |   |         RadialWheel          |   |
|   |  - Floating Obsidian Capsule       |   |  - Polar Angle Annular Slices|   |
|   |  - Cursor Spring Tracking          |   |  - Shift: Formats (275 DAG)  |   |
|   |  - Format Badge & Item Counter     |   |  - Alt: 26 Media Tools       |   |
|   |  - Modifier Triggers Indicator     |   |  - Ctrl: 6 File Actions      |   |
|   +------------------------------------+   +------------------------------+   |
+-------------------------------------------------------------------------------+
                                       │
                                       ▼
+-------------------------------------------------------------------------------+
|                     275-Format Conversion DAG Router                          |
|  - Sub-millisecond Dijkstra shortest-path route solver                        |
+-------------------------------------------------------------------------------+
       │                  │                  │                  │
       ▼                  ▼                  ▼                  ▼
[shaft-image]       [shaft-video]      [shaft-audio]      [shaft-document]
(Resvg/TinySkia)     (Palettegen GIF)  (Adaptive Codecs)  (XLSX/PDF/Markdown)
       │                  │                  │                  │
       └──────────────────┴────────┬─────────┴──────────────────┘
                                   ▼
+-------------------------------------------------------------------------------+
|               Multi-Threaded Rayon Task Queue Manager                         |
|  - CancellationToken, dynamic progress streaming, IPC event bus               |
+-------------------------------------------------------------------------------+
```

---

## 2. Core Capabilities

### 275-Format Conversion DAG
- **Graph-Theoretic Route Solver**: Maintains a directed weighted adjacency matrix of 275 format pairs with Dijkstra shortest-path route solving.
- **Header Metadata Prober**: Extracts binary dimension headers (PNG IHDR, JPEG SOF, GIF, BMP) and audio parameters (WAV fmt) in sub-millisecond latency.
- **Concurrent Rayon Queue**: Non-blocking worker pool with cancellation tokens, remaining time estimation, and real-time progress streaming.

### Format Domains
- **Images (`shaft-image`)**: Bidirectional conversion across PNG, JPG, WebP, BMP, TIFF, ICO. High-fidelity SVG vector rasterization via `resvg` and `tiny-skia`. Deterministic alpha flattening over custom matte backgrounds.
- **Video (`shaft-video`)**: Container transcoding across MP4, MOV, MKV, WebM, AVI, WMV. Two-pass dynamic palettegen 256-color smooth GIF synthesis. Lossless sub-50ms keyframe stream trimming (`-c copy`). Direct audio extraction to MP3/AAC.
- **Audio (`shaft-audio`)**: Audio transcoding across MP3, M4A, WAV, FLAC, OGG, Opus, AIFF, WMA with sample rate and channel layout adaptation.
- **Documents & Spreadsheets (`shaft-document`)**: Full bidirectional spreadsheet engine for XLSX, XLS, ODS, CSV, and TSV using pure Rust `calamine` and `rust_xlsxwriter`. Safe PDF text extraction via `lopdf`. CommonMark to HTML and plaintext compilation with `pulldown-cmark`. Subtitle SRT to VTT converter.
- **Archives (`shaft-archive`)**: Streaming compression and decompression for ZIP, TAR, and TAR.GZ. Cryptographic-grade Zip Slip path traversal sanitizer. Zero-unpack in-memory archive table-of-contents (TOC) inspection. Direct format-to-format archive transcoding.

### Contextual Media Tools & Quick Actions (`shaft-tools`)
- **Audio Tools**: Silence trimmer, EBU R128 loudness normalizer, waveform visualizer video generator, bitrate optimizer.
- **Video Tools**: Pitch-compensated speed ramping (0.25x to 4x), resolution downscaling, aspect ratio cropper (1:1, 9:16, 16:9), frame extraction.
- **Image & PDF Tools**: Metadata scrubber & lossless compressor, multi-resolution favicon ICO bundle generator, PDF page splitter.
- **File Quick Actions**: Cross-platform terminal reveal, file payload copying, absolute path copying, and file moving.

---

## 3. Keyboard Modifier Wheel System

Hovering or dragging files activates contextual radial wheels dynamically:

| Modifier | Mode | Description |
|---|---|---|
| `Shift` | **Formats** | 275-route format conversion DAG selector |
| `Alt` | **Tools** | 26 contextual media manipulation tools |
| `Ctrl` / `Cmd` | **Actions** | 6 instant file and system quick actions |

---

## 4. Multi-OS Vibrancy & Design System

- **Obsidian Dark Canvas**: `#08090a` background with `#0f1012` card surfaces and `rgba(255, 255, 255, 0.08)` hairline borders.
- **macOS**: Native `NSVisualEffectView` frosted glass vibrancy.
- **Windows**: Native Acrylic and Mica blur material integration.
- **Linux**: Transparent compositor blur matching Wayland and X11 modern environments.
- **Radial Wheel Math**: Annular polar sector geometry ($r, \theta$) with angular gaps ($\delta = 0.035$) and spring-damped hover expansion.

---

## 5. Repository Structure

```
modules/shaft/
├── Cargo.toml                    # Cargo workspace root
├── crates/
│   ├── shaft-core/               # 275-format DAG, probe, queue manager, registry
│   ├── shaft-image/              # Resvg vector rasterizer, image transcoders
│   ├── shaft-audio/              # Audio format transcoding engine
│   ├── shaft-video/              # Video containers, palettegen GIF, stream copy
│   ├── shaft-document/           # Calamine, rust_xlsxwriter, lopdf, subtitles
│   ├── shaft-archive/            # Zip/Tar streaming engine, Zip Slip sanitizer
│   └── shaft-tools/              # 26 media tools, 6 file actions, dispatcher
├── src-tauri/                    # Tauri 2.0 shell, window vibrancy, IPC handlers
│   ├── Cargo.toml
│   ├── tauri.conf.json           # Packaging configs (deb, appimage, dmg, msi)
│   └── src/
│       ├── main.rs
│       └── lib.rs
├── src/                          # React 19 + TypeScript + Tailwind CSS UI
│   ├── components/
│   │   ├── dropzone/             # DropZone and FilePreviewCard
│   │   ├── gesture/              # DragGesturePill floating cursor capsule
│   │   ├── layout/               # Titlebar and Footer
│   │   ├── queue/                # TaskQueue batch conversion progress
│   │   └── wheel/                # RadialWheel SVG polar trigonometry
│   ├── hooks/                    # useModifiers keyboard shortcut state
│   ├── stores/                   # useQueueStore Zustand batch state
│   └── types/                    # TypeScript interfaces and signatures
└── package.json
```

---

## 6. Verification & Build Commands

### Workspace Tests
Execute the full unit test suite across all 7 Rust crates:
```bash
cargo test --workspace
```

### Frontend Production Build
Validate TypeScript typings and compile the production bundle:
```bash
npm run build
```

### Desktop Application Packaging
Compile native distribution bundles:
```bash
# Linux (.deb, .AppImage)
cargo tauri build --bundles deb,appimage

# macOS (.dmg)
cargo tauri build --bundles dmg

# Windows (.msi)
cargo tauri build --bundles msi
```

---

## 7. Engineering Standards

- **Safe Rust Guarantee**: `#![deny(unsafe_code)]` enforced across all crates. Zero undefined behavior.
- **Zero Scope Drift**: Focused strictly on high-performance local format transcoding and media manipulation.
- **Zero Emojis**: Rigorous commercial engineering aesthetic across all code, tests, documentation, and commits.
- **Conventional Commits**: Clean commit history following conventional formatting (`feat:`, `fix:`, `chore:`).
