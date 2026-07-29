use thiserror::Error;

use practice_domain::ScoreResult;
use speech_engine::Audio;

pub mod phoneme;
pub mod wer;

pub use phoneme::lookup_phonemes;

/// Errors from pronunciation assessment
#[derive(Debug, Error)]
pub enum AssessmentError {
    #[error("Recognition failed: {0}")]
    RecognitionFailed(String),
    #[error("Empty reference text")]
    EmptyReference,
    #[error("Empty audio")]
    EmptyAudio,
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Abstract pronunciation assessment provider
#[async_trait::async_trait]
pub trait PronunciationAssessor: Send + Sync {
    /// Evaluate user's pronunciation against reference text
    async fn evaluate(
        &self,
        reference: &str,
        audio: &Audio,
    ) -> Result<ScoreResult, AssessmentError>;
}
