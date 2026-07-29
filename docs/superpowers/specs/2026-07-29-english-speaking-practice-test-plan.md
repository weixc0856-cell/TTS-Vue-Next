# English Speaking Practice — Test Plan

## 1. Overview

This document describes the test strategy for the English Speaking Practice extension. It covers unit tests, integration tests, and manual verification for all components.

### Test Layers

| Layer | Tool | Location |
|-------|------|----------|
| Rust unit tests | `cargo test` | `src-tauri/crates/*/src/` and `src-tauri/src/` |
| Rust integration tests | `cargo test` | Across crate boundaries at the app level |
| Frontend unit tests | `vitest` | `src/stores/*.test.ts` and `src/components/*.test.ts` |
| Manual end-to-end | `pnpm tauri dev` | Full app |

---

## 2. Rust Backend Tests

### 2.1 practice-domain (crates/practice-domain)

Pure domain types with no dependencies. Tests cover serialization, enum conversions, and type invariants.

| Test | File | What it verifies |
|------|------|------------------|
| `PracticeMode::as_str/from_str` round-trip | N/A (manual) | All 3 modes convert correctly |
| `Difficulty::as_str/from_str` round-trip | N/A (manual) | All 3 levels convert correctly |
| `WordStatus` variants | N/A (manual) | 4 statuses serialize correctly |

### 2.2 audio-engine (crates/audio-engine)

Recording engine tests.

| Test | File | What it verifies |
|------|------|------------------|
| `test_recorder_new_state` | `audio-engine/src/lib.rs` | New recorder is idle with level 0 |
| `test_stop_without_start_returns_error` | `audio-engine/src/lib.rs` | Stopping before starting returns error |
| `test_list_devices_does_not_panic` | `audio-engine/src/lib.rs` | `list_devices()` doesn't crash in CI/headless |

### 2.3 speech-engine (crates/speech-engine)

Trait definitions only — no concrete implementations to test at this level.

### 2.4 assessment-engine (crates/assessment-engine)

WER-based pronunciation assessment.

| Test | File | What it verifies |
|------|------|------------------|
| `test_perfect_match` | `assessment-engine/src/wer.rs` | Identical reference/transcript → 100% score |
| `test_partial_match` | `assessment-engine/src/wer.rs` | Partial match → lower score, correct word statuses |
| `test_empty_reference_error` | `assessment-engine/src/wer.rs` | Empty reference text → error |
| `test_word_edit_distance_identical` | `assessment-engine/src/wer.rs` | Levenshtein distance = 0 for identical word lists |
| `test_word_edit_distance_different` | `assessment-engine/src/wer.rs` | Levenshtein distance = 1 for one substitution |
| `test_normalize_text` | `assessment-engine/src/wer.rs` | Punctuation and case normalization |

### 2.5 storage (crates/storage)

SQLite-backed persistence.

| Test | File | What it verifies |
|------|------|------------------|
| `test_migrations_run_successfully` | `storage/src/lib.rs` | Schema creation + exercise CRUD |
| `test_insert_and_retrieve_sentences` | `storage/src/lib.rs` | Sentence insert + query by exercise_id |
| `test_session_lifecycle` | `storage/src/lib.rs` | Create session → complete → verify score |
| `test_session_history_returns_sessions` | `storage/src/lib.rs` | Paginated history query joins correctly |
| `test_practice_stats_aggregates_correctly` | `storage/src/lib.rs` | Stats aggregation across sessions and attempts |

### 2.6 content-engine (crates/content-engine)

Scenario parsing and custom exercise creation.

| Test | File | What it verifies |
|------|------|------------------|
| `test_scenario_file_parsing` | `content-engine/src/lib.rs` | JSON scenario file → struct conversion |
| `test_dialogue_file_parsing` | `content-engine/src/lib.rs` | JSON dialogue file → struct conversion |
| `test_create_custom_exercise` | `content-engine/src/lib.rs` | Custom exercise creation + persistence |

### 2.7 Running Backend Tests

```bash
# All workspace crates
cd src-tauri && cargo test -p practice-domain -p audio-engine -p speech-engine \
  -p assessment-engine -p content-engine -p storage

# Full app including existing tests
cd src-tauri && cargo test -p auravioce

# All tests
cd src-tauri && cargo test
```

---

## 3. Frontend Tests

### 3.1 Existing Frontend Tests

The project already has Vitest tests for:
- `src/stores/tts.test.ts` — TTS conversion store
- `src/stores/batch.test.ts` — Batch conversion store
- `src/stores/settings.test.ts` — Settings store
- `src/stores/voices.test.ts` — Voice list store
- `src/utils/themeMode.test.ts` — Theme mode utility
- `src/views/BatchConvert.test.ts` — Batch convert page
- `src/views/Settings.test.ts` — Settings page
- `src/components/*.test.ts` — Various component tests

### 3.2 New Practice Store Tests

Add tests for the 4 new practice stores under `src/stores/practice/`.

### 3.3 Running Frontend Tests

```bash
cd src-tauri/..  # project root
pnpm test        # runs vitest
pnpm test:coverage  # with coverage report
```

### 3.4 Manual Verification Checklist

#### Practice Hub
- [ ] Click "Speaking Practice" in sidebar → navigates to /practice
- [ ] Hub shows 4 mode cards (Shadowing active, Role-play active, 2 disabled)
- [ ] Whisper not-configured warning shows when paths are empty
- [ ] Click "Shadowing" → navigates to /practice/shadowing
- [ ] Click "Role-play" → navigates to /practice/roleplay
- [ ] "History" button navigates to /practice/history

#### Shadowing Session
- [ ] Exercise loads sentences from backend
- [ ] Current sentence displays with translation
- [ ] "Play Reference" button triggers TTS playback
- [ ] RecordButton shows recording state machine (idle→recording→processing→done)
- [ ] AudioVisualizer animates during recording
- [ ] ScoreCard shows after recording
- [ ] WordScoreList shows per-word color coding
- [ ] SessionProgress updates on next sentence
- [ ] "Complete Practice" ends session and returns to hub

#### Role-play Session
- [ ] Partner lines play automatically via TTS
- [ ] Dialogue bubbles display with correct alignment
- [ ] User can record their turn
- [ ] Score shows after recording
- [ ] Auto-advances to next exchange
- [ ] Completes properly

#### Settings
- [ ] Whisper config section displays at bottom
- [ ] Binary path selector works
- [ ] Model path selector works
- [ ] Model size dropdown updates
- [ ] Download model link opens correct URL
- [ ] Clear paths button resets configuration
- [ ] Status chip updates correctly (✅/⚠️)

#### Practice History
- [ ] Stats overview shows total sessions, attempts, avg score, minutes
- [ ] Score trend bars display
- [ ] Session list shows with title, mode, date, score
- [ ] Click session → navigates to detail view
- [ ] Detail view shows all attempts with scores and word breakdown

---

## 4. Integration Test: Whisper Scoring Pipeline

### 4.1 With DummyRecognizer (no Whisper binary)
```
Start → Record → DummyRecognizer returns placeholder text
→ WerAssessor compares against reference
→ ScoreResult returned with word scores
```

### 4.2 With ConfiguredRecognizer (Whisper binary + model)
```
Start → Record WAV → whisper-cli transcribes
→ WerAssessor compares transcript vs reference
→ Accurate word-level scores
```

### 4.3 Verification Commands
```bash
# Full Rust test suite
cd src-tauri && cargo test

# Full frontend test suite
cd .. && pnpm test

# Build check
cd src-tauri && cargo build -p auravioce
cd .. && npx vue-tsc --noEmit && npx vite build
```

---

## 5. Test Data

### 5.1 Built-in Test Scenarios

JSON scenario files in `src-tauri/assets/scenarios/`:

| File | Mode | Sentences | Difficulty |
|------|------|-----------|------------|
| `shadowing-coffee-shop.json` | Shadowing | 5 | Beginner |
| `shadowing-airport.json` | Shadowing | 6 | Intermediate |
| `shadowing-job-interview.json` | Shadowing | 7 | Advanced |
| `dialogue-hotel.json` | Role-play | 7 exchanges | Beginner |
| `dialogue-restaurant.json` | Role-play | 8 exchanges | Beginner |
| `dialogue-doctor.json` | Role-play | 8 exchanges | Intermediate |

### 5.2 WER Test Pairs

| Reference | Transcript | Expected Score |
|-----------|------------|----------------|
| "hello world" | "hello world" | 100% |
| "hello world" | "hello" | ~50% (omission) |
| "hello world" | "goodbye world" | ~50% (substitution) |
| "" | anything | Error: EmptyReference |

---

## 6. Edge Cases

| Scenario | Expected Behavior |
|----------|------------------|
| Empty exercise list | "No exercises found" message |
| No sentences in exercise | "No sentences available" message |
| Stop recording without starting | Error shown, not crash |
| All attempts with no score | Score = 0, not NaN |
| Session with no attempts | Completes with score 0 |
| Whisper binary path invalid | Falls back to DummyRecognizer |
| Very long sentence (>4096 bytes) | Split by existing chunking logic |

## 7. Test Infrastructure

- **Mocking**: Rust tests use in-memory SQLite (`Storage::open_in_memory()`) for storage tests
- **Mock recognizer**: `DummyRecognizer` in `speech/whisper.rs` returns placeholder text
- **Vitest**: Frontend tests use `happy-dom` with mocked `invoke()` calls
- **CI**: GitHub Actions runs `cargo test` and `pnpm test` on push
