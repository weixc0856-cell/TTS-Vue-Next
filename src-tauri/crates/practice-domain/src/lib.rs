use serde::{Deserialize, Serialize};

/// The type of practice exercise
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PracticeMode {
    Shadowing,
    Roleplay,
}

impl PracticeMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            PracticeMode::Shadowing => "shadowing",
            PracticeMode::Roleplay => "roleplay",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "shadowing" => Some(PracticeMode::Shadowing),
            "roleplay" => Some(PracticeMode::Roleplay),
            _ => None,
        }
    }
}

/// Difficulty level of an exercise
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Difficulty {
    Beginner,
    Intermediate,
    Advanced,
}

impl Difficulty {
    pub fn as_str(&self) -> &'static str {
        match self {
            Difficulty::Beginner => "beginner",
            Difficulty::Intermediate => "intermediate",
            Difficulty::Advanced => "advanced",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "beginner" => Some(Difficulty::Beginner),
            "intermediate" => Some(Difficulty::Intermediate),
            "advanced" => Some(Difficulty::Advanced),
            _ => None,
        }
    }
}

/// Category of an exercise scenario
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ScenarioCategory {
    Daily,
    Travel,
    Business,
    Academic,
    Exam,
}

impl ScenarioCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            ScenarioCategory::Daily => "daily",
            ScenarioCategory::Travel => "travel",
            ScenarioCategory::Business => "business",
            ScenarioCategory::Academic => "academic",
            ScenarioCategory::Exam => "exam",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "daily" => Some(ScenarioCategory::Daily),
            "travel" => Some(ScenarioCategory::Travel),
            "business" => Some(ScenarioCategory::Business),
            "academic" => Some(ScenarioCategory::Academic),
            "exam" => Some(ScenarioCategory::Exam),
            _ => None,
        }
    }
}

/// Source of an exercise (built-in, user-imported, or document-derived)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExerciseSource {
    Builtin,
    User,
    Document,
}

impl ExerciseSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            ExerciseSource::Builtin => "builtin",
            ExerciseSource::User => "user",
            ExerciseSource::Document => "document",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "builtin" => Some(ExerciseSource::Builtin),
            "user" => Some(ExerciseSource::User),
            "document" => Some(ExerciseSource::Document),
            _ => None,
        }
    }
}

/// A spoken sentence within an exercise
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sentence {
    pub id: String,
    pub text: String,
    pub translation: Option<String>,
    pub order_index: i32,
}

/// An exercise containing multiple sentences for practice
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Exercise {
    pub id: String,
    pub title: String,
    pub mode: PracticeMode,
    pub difficulty: Difficulty,
    pub category: ScenarioCategory,
    pub source: ExerciseSource,
    pub sentences: Vec<Sentence>,
    pub metadata: serde_json::Value,
}

/// Word-level score status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WordStatus {
    Correct,
    Wrong,
    Omitted,
    Extra,
}

/// Score for a single word
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordScore {
    pub word: String,
    pub status: WordStatus,
    pub confidence: f64,
}

/// A practice session (one run through an exercise)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PracticeSession {
    pub id: String,
    pub exercise_id: String,
    pub mode: PracticeMode,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub overall_score: Option<f64>,
}

/// A single attempt at one sentence
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// Result of scoring an attempt against reference text
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreResult {
    pub overall: f64,
    pub accuracy: f64,
    pub completeness: f64,
    pub word_scores: Vec<WordScore>,
}
