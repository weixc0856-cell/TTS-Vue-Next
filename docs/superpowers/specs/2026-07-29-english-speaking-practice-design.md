# English Speaking Practice — Design Document

> Extending TTS Vue Next (Edge TTS + Vue 3 + Vuetify + Tauri) into a complete English speaking practice tool with listening-speaking feedback loop.

## 1. Overview

### 1.1 Goals

Transform the existing single-direction TTS application into a bidirectional **listening + speaking + feedback** platform for English oral practice. Key capabilities:

- **Shadowing (影子跟读)** — Listen to native audio → user repeats → pronunciation scored (Phase 1)
- **Role-play (场景对话)** — Simulated dialogues with TTS-powered interlocutor (Phase 1)
- **Document Reading (文档阅读跟读)** — Import TXT/MD for reading and shadowing practice (Phase 1)
- **Pronunciation Drill (发音纠正)** — Targeted phoneme-level correction (Phase 3)
- **Free Talk (自由对话)** — LLM-powered open conversation (Phase 3)

### 1.2 Architecture Principles

1. **Provider Trait Pattern** — All AI capabilities (TTS, STT, Assessment) defined as Rust traits, not concrete implementations. Business logic depends on interfaces, not models. Models become swappable providers.
2. **Domain-Driven Crate Structure** — Separate crates for domain model, audio engine, speech recognition, assessment, content, and storage.
3. **Text-Level MVP** — Phase 1 scoring uses WER (word error rate) only. No phoneme, no forced alignment. Phoneme assessment deferred to Phase 3.
4. **SQLite Persistence** — Practice history, attempts, and progress stored in SQLite via `rusqlite`, not localStorage.
5. **Focused Phase 0** — Infrastructure first: Tauri commands, provider traits, SQLite schema, session lifecycle — before any practice mode UI.
6. **Offline-First** — Core practice loop works fully offline. LLM features are additive, enabled only when user provides an API key.

### 1.3 Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Speech-to-Text | **Whisper local** (`whisper-rs`, base.en) | Offline, privacy, zero cost; base.en ~150MB good accuracy on CPU (i7-13700K) |
| Pronunciation Assessment (Phase 1) | **Text-level WER** | Fast, reliable, no phoneme dependency; phoneme scoring deferred to wav2vec2 in Phase 3 |
| Content Source | **Pre-built JSON scenarios** | Works offline OOTB; LLM integration deferred to Phase 3 |
| Storage | **SQLite** (`rusqlite`) | Structured history, learning curve analysis, session replay |
| Architecture | **Provider traits + DDD crates** | AI models swappable, business logic testable, future-proof |

## 2. Architecture

### 2.1 Module Overview

```
┌──────────────────────────────────────────────────┐
│               Vue 3 Frontend                      │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐       │
│  │Shadowing │  │ Roleplay │  │Document  │       │
│  │Session   │  │Session   │  │Import    │       │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘       │
│       └──────┬──────┴──────────────┘              │
│         ┌────┴────┐                               │
│         │Shared UI│ (RecordBtn, ScoreCard, Viz)   │
│         └────┬────┘                               │
│  ┌─────┬─────┼─────────┬─────────┐                │
│  │session.ts │recorder.ts│scoring.ts│content.ts   │
│  └─────┬─────┴─────────┴─────────┘                │
└────────┼──────────────────────────────────────────┘
         │ Tauri IPC
         ▼
┌──────────────────────────────────────────────────┐
│            Tauri App Shell (src/)                  │
│  commands/practice.rs · commands/storage.rs        │
│  lib.rs                                           │
└────────┬─────────────────────────────────────────┘
         │
    ┌────┴───────────────────────────┐
    │        Rust Crates              │
    │                                 │
    │ ┌──────────────┐                │
    │ │practice-domain│ (pure types)  │
    │ └──────┬───────┘                │
    │        │ depends on             │
    │ ┌──────▼───────┐ ┌────────────┐ │
    │ │ audio-engine  │ │ storage    │ │
    │ │ (cpal/hound)  │ │ (SQLite)   │ │
    │ └──────────────┘ └────────────┘ │
    │                                 │
    │ Provider Layer (traits)         │
    │ ┌──────────────┐ ┌────────────┐ │
    │ │ speech-engine │ │assessment- │ │
    │ │   trait +     │ │ engine     │ │
    │ │ Whisper impl  │ │ trait +    │ │
    │ └──────────────┘ │ WER impl   │ │
    │                  └────────────┘ │
    │ ┌──────────────┐                │
    │ │content-engine │ (JSON→SQLite) │
    │ └──────────────┘                │
    │                                 │
    │ ┌──────────────────────────┐    │
    │ │ EdgeTtsProvider (trait)  │    │
    │ │ wraps existing Communicate│   │
    │ └──────────────────────────┘    │
    └─────────────────────────────────┘
```

### 2.2 Core Data Flow (Shadowing, Phase 1)

```
User selects sentence (from scenario or typed text)
       │
       ▼
TTS plays reference audio (EdgeTtsProvider wraps Communicate)
       │
       ▼
User records voice via microphone (audio-engine: cpal → WAV)
       │
       ▼
Whisper transcribes audio → Transcript with text + segments (speech-engine)
       │
       ▼
WerAssessor computes: reference text vs transcript → WER + per-word status (assessment-engine)
       │
       ▼
ScoreResult returned to frontend → render word-level feedback (ScoreCard + WordScoreList)
```

## 3. Workspace Structure

```
src-tauri/
├── Cargo.toml                    # workspace root
├── crates/
│   ├── practice-domain/          # Pure domain model, zero dependencies
│   ├── audio-engine/             # cpal + hound + rodio
│   ├── speech-engine/            # Provider trait + whisper-rs implementation
│   ├── assessment-engine/        # Provider trait + WER implementation
│   ├── content-engine/           # Scenario + sentence management
│   └── storage/                  # SQLite via rusqlite
├── src/                          # Tauri app shell, command registration
│   ├── lib.rs
│   └── commands/
│       ├── mod.rs
│       ├── practice.rs           # NEW: practice session commands
│       └── storage.rs            # NEW: storage-backed queries
├── assets/
│   └── scenarios/                # JSON scenario library files
└── binaries/                     # ggml-base.en.bin (downloaded separately)
```

## 4. Provider Traits

### 4.1 TtsProvider
```rust
/// Abstract text-to-speech synthesis
#[async_trait]
pub trait TtsProvider: Send + Sync {
    async fn synthesize(&self, text: &str, voice: &str, 
                         config: &TtsConfig) -> Result<Audio, TtsError>;
}

// Provider: EdgeTtsProvider wraps existing Communicate struct
pub struct EdgeTtsProvider { /* holds Communicate config */ }
```

### 4.2 SpeechRecognizer
```rust
/// Abstract speech recognition
#[async_trait]
pub trait SpeechRecognizer: Send + Sync {
    async fn transcribe(&self, audio: &Audio) -> Result<Transcript, RecognitionError>;
}

// Provider: WhisperRecognizer uses whisper-rs with ggml-base.en.bin
pub struct WhisperRecognizer { /* thread-safe model singleton */ }
```

### 4.3 PronunciationAssessor
```rust
/// Abstract pronunciation assessment
#[async_trait]
pub trait PronunciationAssessor: Send + Sync {
    async fn evaluate(&self, reference: &str, audio: &Audio) 
        -> Result<Assessment, AssessmentError>;
}

// Phase 1: WerAssessor (text-match only, uses SpeechRecognizer internally)
pub struct WerAssessor<R: SpeechRecognizer> { recognizer: R }

// Phase 3: PhonemeAssessor (wav2vec2 forced alignment)
// pub struct PhonemeAssessor { ... }
```

## 5. domain Model (practice-domain crate)

### practice-domain/src/lib.rs

```rust
pub struct PracticeSession {
    pub id: String,
    pub exercise_id: String,
    pub mode: PracticeMode,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub overall_score: Option<f64>,
}

pub enum PracticeMode { Shadowing, Roleplay }

pub struct Exercise {
    pub id: String,
    pub title: String,
    pub sentences: Vec<Sentence>,
    pub difficulty: Difficulty,
    pub category: ScenarioCategory,
}

pub struct Sentence {
    pub id: String,
    pub text: String,
    pub translation: Option<String>,
    pub order_index: i32,
}

pub struct Attempt {
    pub id: String,
    pub session_id: String,
    pub sentence_id: String,
    pub audio_path: Option<String>,
    pub transcript: Option<String>,
    pub score: Option<f64>,
    pub word_scores: Vec<WordScore>,
    pub completeness: Option<f64>,
    pub duration_ms: Option<i64>,
}

pub struct WordScore {
    pub word: String,
    pub status: WordStatus,  // Correct | Wrong | Omitted | Extra
    pub confidence: f64,
}

pub enum WordStatus { Correct, Wrong, Omitted, Extra }

pub struct ScoreResult {
    pub overall: f64,
    pub accuracy: f64,
    pub completeness: f64,
    pub word_scores: Vec<WordScore>,
}
```

## 6. Data Model (SQLite Schema)

### exercises
```sql
CREATE TABLE exercises (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    mode TEXT NOT NULL CHECK(mode IN ('shadowing','roleplay')),
    difficulty TEXT NOT NULL CHECK(difficulty IN ('beginner','intermediate','advanced')),
    category TEXT NOT NULL CHECK(category IN ('daily','travel','business','academic','exam')),
    source TEXT NOT NULL DEFAULT 'builtin' CHECK(source IN ('builtin','user','document')),
    metadata TEXT DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
```

### sentences
```sql
CREATE TABLE sentences (
    id TEXT PRIMARY KEY,
    exercise_id TEXT NOT NULL REFERENCES exercises(id) ON DELETE CASCADE,
    text TEXT NOT NULL,
    translation TEXT,
    order_index INTEGER NOT NULL,
    expected_duration_ms INTEGER
);
CREATE INDEX idx_sentences_exercise ON sentences(exercise_id);
```

### practice_sessions
```sql
CREATE TABLE practice_sessions (
    id TEXT PRIMARY KEY,
    exercise_id TEXT NOT NULL REFERENCES exercises(id),
    mode TEXT NOT NULL,
    started_at TEXT NOT NULL DEFAULT (datetime('now')),
    completed_at TEXT,
    overall_score REAL
);
CREATE INDEX idx_sessions_exercise ON practice_sessions(exercise_id);
```

### attempts
```sql
CREATE TABLE attempts (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL REFERENCES practice_sessions(id) ON DELETE CASCADE,
    sentence_id TEXT NOT NULL REFERENCES sentences(id),
    audio_path TEXT,
    transcript TEXT,
    score REAL,
    word_scores TEXT,             -- JSON array of WordScore
    fluency REAL,
    completeness REAL,
    duration_ms INTEGER,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX idx_attempts_session ON attempts(session_id);
```

## 7. Phase Plan

### Phase 0 — Infrastructure (This sprint)

**Goal:** All backend crates in place, compiles, provider traits defined, SQLite seeded.

**Backend:**
1. Set up workspace `Cargo.toml` with crate paths
2. Create `practice-domain` crate with domain types
3. Create `audio-engine` crate:
   - List audio devices
   - `start_recording()` / `stop_recording()` → WAV via cpal + hound
   - `get_audio_level()` → peak level 0-1
4. Create `storage` crate:
   - SQLite init with schema migrations
   - CRUD: exercises, sentences, sessions, attempts
   - Connection pool via `r2d2`
5. Create `content-engine` crate:
   - JSON scenario file parser
   - Seed built-in exercises into SQLite on first run
   - `list_exercises(filters)` / `get_exercise_detail(id)`
6. Create `speech-engine` crate:
   - `SpeechRecognizer` trait
   - (No impl yet — Whisper model download deferred to Phase 1)
7. Create `assessment-engine` crate:
   - `PronunciationAssessor` trait
   - (No impl yet — WER assessor deferred to Phase 1)
8. Create `practice.rs` command module with skeleton commands:
   - `start_session(exercise_id) → session_id`
   - `end_session(session_id)`
   - `get_session(session_id)`
9. Implement `EdgeTtsProvider` — wraps existing `Communicate` struct behind `TtsProvider` trait
10. Register all Phase 0 commands in `lib.rs`
11. Add Tauri plugins needed: `tauri-plugin-store` (for config)

**Frontend:**
12. Create `src/types/practice.ts` — all TypeScript types matching domain model
13. Create `src/stores/practice/session.ts` — session lifecycle state
14. Create `src/stores/practice/recorder.ts` — recording state (mirrors audio-engine)
15. Create `src/stores/practice/scoring.ts` — score result state
16. Create `src/stores/practice/content.ts` — exercise list/detail
17. Create `src/components/practice/shared/`:
    - `RecordButton.vue` — button with recording state machine (idle→recording→done)
    - `AudioVisualizer.vue` — canvas-based waveform display

### Phase 1 — MVP Practice Loop (Next sprint)

**Backend:**
1. Download & bundle `ggml-base.en.bin` (~150MB) into `src-tauri/binaries/`
2. Implement `WhisperRecognizer`:
   - Lazy singleton load via `OnceLock<Mutex<WhisperModel>>`
   - CPU-only inference, thread-safe
   - Model path configurable via env var, fallback to bundled path
   - Return `Transcript { text, segments: [{start, end, text}] }`
3. Implement `WerAssessor`:
   - Internal whisper call → WER (edit distance)
   - Per-word status: correct/wrong/omitted/extra
   - Completeness ratio
   - Pure Rust implementation, no additional model
4. Implement practice workflow commands:
   - `record_attempt(session_id, sentence_id) → Attempt` (record → transcribe → score → save)
   - `get_session_results(session_id) → Vec<Attempt>`
5. Implement `import_document(path)`:
   - Read .txt / .md, smart sentence splitting (regex `[.!?]` + quote handling)
   - Create exercise from extracted sentences
6. Add `split_sentences(text)` utility command
7. TTS page integration: "Send to Practice" button

**Frontend:**
8. Add nav item to `AppLayout.vue`: `Speaking Practice → /practice`
9. Build `PracticeHub.vue` — mode selection with cards (shadowing + roleplay active, others disabled)
10. Build `ScenarioSelector.vue` — browse exercises by difficulty/category
11. Build `ShadowingSession.vue`:
    - SentencePanel: show text + translation, "Play" TTS button
    - RecordButton: record → auto-detect silence → stop → process
    - ScoreCard: overall + accuracy + completeness
    - WordScoreList: color-coded words (green/amber/red/gray)
    - SessionProgress: N/M sentences
12. Build `RoleplaySession.vue`:
    - DialogueBubble: chat-style exchange display
    - TTS auto-plays partner lines
    - User records their turn → scored
13. Wire stores to backend commands
14. i18n additions for new pages
15. Route registration

### Phase 2 — Progress & History (Future)

1. Practice history queries: session list, detail view, attempt replay
2. Statistics: streaks, score trends, total practice time
3. `.pdf` document import via `pdf-extract`
4. History UI page
5. Statistics dashboard
6. 10+ additional built-in scenarios

### Phase 3 — Deep Assessment & LLM (Future)

1. wav2vec2 forced alignment for phoneme-level assessment
2. Pronunciation Drill mode with PhonemeFeedback
3. LLM Gateway trait + providers (OpenAI, Ollama)
4. Free Talk mode
5. LLM-powered grammar/content evaluation
6. API key management in Settings

## 8. Document Import Scope

| Format | Phase | Library | Notes |
|--------|-------|---------|-------|
| `.txt` | 1 | — | Simple read, already exists in codebase |
| `.md` | 1 | `pulldown-cmark` | Already supported; strip formatting for practice |

## 9. Frontend Store Structure

```
src/stores/practice/
├── session.ts      — PracticeSession state: current sentence, mode, progress
├── recorder.ts     — Recording state: isRecording, audioLevel, duration
├── scoring.ts      — ScoreResult: word scores, attempts, session summary
└── content.ts      — Exercise catalog: list, filters, detail
```

Split from the original monolithic `practice.ts` design to match domain boundaries.

## 10. i18n Additions

### `src/locales/en.ts` + `zh.ts`

```typescript
nav: {
  // existing...
  practice: "Speaking Practice" / "口语练习",
},

practice: {
  hub: {
    title: "Practice Center" / "练习中心",
    subtitle: "Choose a practice mode" / "选择练习模式",
    shadowing: { title: "Shadowing", desc: "Listen and repeat" },
    roleplay: { title: "Role-play", desc: "Simulated dialogues" },
    pronunciation: { title: "Pronunciation", desc: "Coming soon" },
    freetalk: { title: "Free Talk", desc: "Coming soon" },
  },
  shadowing: {
    listen: "Listen",
    record: "Record",
    stop: "Stop",
    next: "Next Sentence",
    result: "Your Score",
  },
  scoring: {
    overall: "Overall",
    accuracy: "Accuracy",
    completeness: "Completeness",
    fluency: "Fluency",
  },
  // ...
}
```

## 11. Verification

**Backend tests:**
- `cargo test` — unit tests for each crate (domain, storage, assessment)
- Mock provider traits to test business logic without real AI models
- SQLite in-memory tests for storage CRUD
- WER assessor: known reference/transcript pairs verify correct scores

**Frontend tests:**
- `pnpm test` — Vitest tests following existing patterns
- Store tests with mocked `invoke` calls

**Manual verification:**
- `pnpm tauri dev` — full app launch
- Phase 0: verify commands can be invoked from frontend
- Phase 1: record audio → transcribe → score → see results in UI
