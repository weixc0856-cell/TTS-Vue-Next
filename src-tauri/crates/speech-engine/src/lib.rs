use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Audio data container — typically 16kHz mono WAV bytes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Audio {
    pub data: Vec<u8>,
    pub sample_rate: u32,
    pub channels: u16,
    pub duration_ms: u64,
}

/// A segment of transcribed audio with timing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Segment {
    pub start: f64,
    pub end: f64,
    pub text: String,
}

/// Result of speech recognition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transcript {
    pub text: String,
    pub segments: Vec<Segment>,
}

/// Errors from speech recognition
#[derive(Debug, Error)]
pub enum RecognitionError {
    #[error("Model not loaded: {0}")]
    ModelNotLoaded(String),
    #[error("Transcription failed: {0}")]
    TranscriptionFailed(String),
    #[error("Audio format not supported: {0}")]
    InvalidAudio(String),
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Abstract speech recognition provider
#[async_trait::async_trait]
pub trait SpeechRecognizer: Send + Sync {
    /// Transcribe audio to text with timing segments
    async fn transcribe(&self, audio: &Audio) -> Result<Transcript, RecognitionError>;
}

/// Configuration for TTS synthesis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TtsConfig {
    pub voice: String,
    pub rate: String,
    pub pitch: String,
    pub volume: String,
    pub output_format: String,
}

impl Default for TtsConfig {
    fn default() -> Self {
        Self {
            voice: "en-US-EmmaMultilingualNeural".to_string(),
            rate: "+0%".to_string(),
            pitch: "+0Hz".to_string(),
            volume: "+0%".to_string(),
            output_format: "audio-24khz-48kbitrate-mono-mp3".to_string(),
        }
    }
}

/// Errors from TTS synthesis
#[derive(Debug, Error)]
pub enum TtsError {
    #[error("Synthesis failed: {0}")]
    SynthesisFailed(String),
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Abstract text-to-speech provider
#[async_trait::async_trait]
pub trait TtsProvider: Send + Sync {
    /// Synthesize text to audio
    async fn synthesize(
        &self,
        text: &str,
        config: &TtsConfig,
    ) -> Result<Audio, TtsError>;
}
