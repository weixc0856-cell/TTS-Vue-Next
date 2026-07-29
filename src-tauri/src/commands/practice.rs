use std::sync::Mutex;

use tauri::{command, AppHandle, Manager};
use uuid::Uuid;

use assessment_engine::wer::WerAssessor;
use assessment_engine::PronunciationAssessor;
use content_engine::ContentManager;
use storage::{PracticeStats, SessionSummary};
use practice_domain::{Attempt, Exercise, PracticeMode, PracticeSession, ScoreResult};
use speech_engine::{Audio, SpeechRecognizer};

/// Application state holding our practice engine components
pub struct AppState {
    pub content: Mutex<ContentManager>,
}

impl AppState {
    pub fn new(storage: storage::Storage) -> Self {
        let content = ContentManager::new(storage);
        AppState {
            content: Mutex::new(content),
        }
    }
}

/// Select the best available speech recognizer:
/// 1. ConfiguredRecognizer if explicit paths provided
/// 2. CliRecognizer if env vars or bundled files found
/// 3. DummyRecognizer fallback for development
fn create_recognizer(
    whisper_bin_path: Option<&str>,
    whisper_model_path: Option<&str>,
) -> Box<dyn SpeechRecognizer + Send + Sync> {
    // 1. Explicit paths from settings
    if let (Some(bin), Some(model)) = (whisper_bin_path, whisper_model_path) {
        let bin_path = std::path::PathBuf::from(bin);
        let model_path = std::path::PathBuf::from(model);
        if bin_path.exists() && model_path.exists() {
            log::info!("Using configured Whisper paths: {:?}, {:?}", bin_path, model_path);
            return Box::new(crate::speech::whisper::ConfiguredRecognizer::new(bin_path, model_path));
        }
    }

    // 2. Auto-detected (env vars / bundled)
    if crate::speech::whisper::CliRecognizer::is_available() {
        log::info!("Using auto-detected Whisper CLI recognizer");
        Box::new(crate::speech::whisper::CliRecognizer)
    } else {
        log::warn!(
            "Whisper not configured. Set paths in Settings or set TTS_VUE_NEXT_WHISPER_PATH \
             and TTS_VUE_NEXT_WHISPER_MODEL env vars. Using DummyRecognizer."
        );
        Box::new(crate::speech::whisper::DummyRecognizer::new())
    }
}

/// Transcribe audio and score it against reference text
fn transcribe_and_score_inner(
    audio_data: &[u8],
    reference_text: &str,
    duration_ms: i64,
    whisper_bin_path: Option<&str>,
    whisper_model_path: Option<&str>,
) -> Result<(Attempt, ScoreResult), String> {
    // Build Audio from raw WAV bytes
    let audio = Audio {
        data: audio_data.to_vec(),
        sample_rate: 16000,
        channels: 1,
        duration_ms: duration_ms as u64,
    };

    // Select recognizer and run assessment
    let recognizer = create_recognizer(whisper_bin_path, whisper_model_path);
    let assessor = WerAssessor::new(recognizer);

    let score_result = tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            assessor.evaluate(reference_text, &audio).await
        })
    }).map_err(|e| format!("Scoring failed: {e}"))?;

    let attempt = Attempt {
        id: Uuid::new_v4().to_string(),
        session_id: String::new(), // filled in by caller
        sentence_id: String::new(), // filled in by caller
        audio_path: None,
        transcript: None,
        score: Some(score_result.overall),
        word_scores: score_result.word_scores.clone(),
        completeness: Some(score_result.completeness),
        duration_ms: Some(duration_ms),
    };

    Ok((attempt, score_result))
}

/// Split text into sentences for document-based practice
#[command]
pub async fn split_sentences(text: String) -> Result<Vec<String>, String> {
    // Simple sentence splitting on sentence-ending punctuation
    let normalized = text.replace('\n', " ");
    let mut sentences = Vec::new();
    let mut current = String::new();

    for c in normalized.chars() {
        current.push(c);
        if matches!(c, '.' | '!' | '?' | '。' | '！' | '？') {
            let trimmed = current.trim().to_string();
            if !trimmed.is_empty() && trimmed.len() > 5 {
                sentences.push(trimmed);
            }
            current = String::new();
        }
    }

    let trimmed = current.trim().to_string();
    if !trimmed.is_empty() && trimmed.len() > 5 {
        sentences.push(trimmed);
    }

    // If no sentences found (no punctuation), split by length
    if sentences.is_empty() && !text.trim().is_empty() {
        // Split into roughly 100-char chunks
        for chunk in text.as_bytes().chunks(100) {
            let s = String::from_utf8_lossy(chunk).trim().to_string();
            if !s.is_empty() {
                sentences.push(s);
            }
        }
    }

    Ok(sentences)
}

/// Initialize the content engine and seed built-in scenarios
#[command]
pub async fn seed_content(app: AppHandle) -> Result<Vec<String>, String> {
    let state = app.state::<AppState>();
    let resource_dir = app
        .path()
        .resource_dir()
        .map_err(|e| format!("Failed to get resource dir: {e}"))?;

    let scenarios_dir = resource_dir.join("assets").join("scenarios");

    // If resource dir doesn't have scenarios, try development path
    let scenarios_dir = if scenarios_dir.exists() {
        scenarios_dir
    } else {
        // Fallback: check relative to manifest dir or cwd
        let dev_path: std::path::PathBuf = [
            env!("CARGO_MANIFEST_DIR"),
            "assets",
            "scenarios",
        ]
        .iter()
        .collect();
        if dev_path.exists() {
            dev_path
        } else {
            return Err("Scenarios directory not found".to_string());
        }
    };

    let content = state.content.lock().map_err(|e| e.to_string())?;
    let seeded = content
        .seed_from_directory(&scenarios_dir)
        .map_err(|e| e.to_string())?;

    Ok(seeded)
}

/// List all available exercises with optional filters
#[command]
pub async fn list_exercises(
    app: AppHandle,
    mode: Option<String>,
    category: Option<String>,
    difficulty: Option<String>,
) -> Result<Vec<Exercise>, String> {
    let state = app.state::<AppState>();
    let content = state.content.lock().map_err(|e| e.to_string())?;
    let exercises = content
        .list_exercises(mode.as_deref(), category.as_deref(), difficulty.as_deref())
        .map_err(|e| e.to_string())?;
    Ok(exercises)
}

/// Get full detail of an exercise including all sentences
#[command]
pub async fn get_exercise_detail(app: AppHandle, id: String) -> Result<Exercise, String> {
    let state = app.state::<AppState>();
    let content = state.content.lock().map_err(|e| e.to_string())?;
    let exercise = content.get_exercise(&id).map_err(|e| e.to_string())?;
    Ok(exercise)
}

/// Start a new practice session for an exercise
#[command]
pub async fn start_session(
    app: AppHandle,
    exercise_id: String,
    mode: String,
) -> Result<String, String> {
    let state = app.state::<AppState>();
    let content = state.content.lock().map_err(|e| e.to_string())?;
    let storage = content.get_storage();

    let practice_mode = PracticeMode::from_str(&mode)
        .ok_or_else(|| format!("Invalid practice mode: {mode}"))?;

    let session = PracticeSession {
        id: Uuid::new_v4().to_string(),
        exercise_id,
        mode: practice_mode,
        started_at: chrono::Utc::now().to_rfc3339(),
        completed_at: None,
        overall_score: None,
    };

    storage.create_session(&session).map_err(|e| e.to_string())?;
    Ok(session.id)
}

/// Record an attempt with automatic transcription and scoring
#[command]
pub async fn record_attempt(
    app: AppHandle,
    session_id: String,
    sentence_id: String,
    reference_text: String,
    audio_data: Vec<u8>,
    duration_ms: i64,
    whisper_bin_path: Option<String>,
    whisper_model_path: Option<String>,
) -> Result<Attempt, String> {
    let state = app.state::<AppState>();
    let content = state.content.lock().map_err(|e| e.to_string())?;
    let storage = content.get_storage();

    // Transcribe and score
    let (mut attempt, _score_result) =
        transcribe_and_score_inner(
            &audio_data, &reference_text, duration_ms,
            whisper_bin_path.as_deref(), whisper_model_path.as_deref(),
        )?;

    attempt.session_id = session_id;
    attempt.sentence_id = sentence_id;

    storage
        .insert_attempt(&attempt)
        .map_err(|e| e.to_string())?;
    Ok(attempt)
}

/// Transcribe audio and score against reference text without saving
/// Useful for previewing scores before recording an attempt
#[command]
pub async fn transcribe_and_score(
    reference_text: String,
    audio_data: Vec<u8>,
    duration_ms: i64,
    whisper_bin_path: Option<String>,
    whisper_model_path: Option<String>,
) -> Result<ScoreResult, String> {
    let (_attempt, score) =
        transcribe_and_score_inner(
            &audio_data, &reference_text, duration_ms,
            whisper_bin_path.as_deref(), whisper_model_path.as_deref(),
        )?;
    Ok(score)
}

/// End a practice session and calculate overall score
#[command]
pub async fn end_session(app: AppHandle, session_id: String) -> Result<f64, String> {
    let state = app.state::<AppState>();
    let content = state.content.lock().map_err(|e| e.to_string())?;
    let storage = content.get_storage();

    // Calculate average score from all attempts
    let attempts = storage
        .get_attempts(&session_id)
        .map_err(|e| e.to_string())?;

    let avg_score: f64 = if attempts.is_empty() {
        0.0
    } else {
        let scores: Vec<f64> = attempts.iter().filter_map(|a| a.score).collect();
        if scores.is_empty() {
            0.0
        } else {
            scores.iter().sum::<f64>() / scores.len() as f64
        }
    };

    storage
        .complete_session(&session_id, avg_score)
        .map_err(|e| e.to_string())?;

    Ok(avg_score)
}

/// Get session details with all attempts
#[command]
pub async fn get_session(app: AppHandle, id: String) -> Result<SessionDetail, String> {
    let state = app.state::<AppState>();
    let content = state.content.lock().map_err(|e| e.to_string())?;
    let storage = content.get_storage();

    let session = storage.get_session(&id).map_err(|e| e.to_string())?;
    let attempts = storage.get_attempts(&id).map_err(|e| e.to_string())?;

    Ok(SessionDetail {
        session,
        attempts,
    })
}

/// Response type for session detail
#[derive(serde::Serialize)]
pub struct SessionDetail {
    pub session: PracticeSession,
    pub attempts: Vec<Attempt>,
}

/// Create a custom exercise from user-provided text
#[command]
pub async fn create_custom_exercise(
    app: AppHandle,
    title: String,
    sentences: Vec<String>,
    translations: Vec<Option<String>>,
) -> Result<Exercise, String> {
    let state = app.state::<AppState>();
    let content = state.content.lock().map_err(|e| e.to_string())?;
    let exercise = content
        .create_custom_exercise(&title, &sentences, &translations)
        .map_err(|e| e.to_string())?;
    Ok(exercise)
}

/// Get practice session history
#[command]
pub async fn get_session_history(
    app: AppHandle,
    limit: i64,
    offset: i64,
) -> Result<Vec<SessionSummary>, String> {
    let state = app.state::<AppState>();
    let content = state.content.lock().map_err(|e| e.to_string())?;
    let storage = content.get_storage();
    storage
        .get_session_history(limit, offset)
        .map_err(|e| e.to_string())
}

/// Get aggregate practice statistics
#[command]
pub async fn get_practice_stats(
    app: AppHandle,
) -> Result<PracticeStats, String> {
    let state = app.state::<AppState>();
    let content = state.content.lock().map_err(|e| e.to_string())?;
    let storage = content.get_storage();
    storage
        .get_practice_stats()
        .map_err(|e| e.to_string())
}
