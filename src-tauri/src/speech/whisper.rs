use std::path::PathBuf;
use std::process::Stdio;

use speech_engine::{Audio, RecognitionError, Segment, SpeechRecognizer, Transcript};

/// Recognizer that uses the whisper.cpp CLI with explicit binary and model paths
pub struct ConfiguredRecognizer {
    pub binary_path: PathBuf,
    pub model_path: PathBuf,
}

impl ConfiguredRecognizer {
    pub fn new(binary_path: PathBuf, model_path: PathBuf) -> Self {
        ConfiguredRecognizer {
            binary_path,
            model_path,
        }
    }
}

/// Recognizer that uses the whisper.cpp CLI as an external sidecar process
/// Resolves binary and model from environment variables or bundled locations.
pub struct CliRecognizer;

impl CliRecognizer {
    fn find_binary() -> Option<PathBuf> {
        // 1. Check env var
        if let Ok(path) = std::env::var("TTS_VUE_NEXT_WHISPER_PATH") {
            let p = PathBuf::from(&path);
            if p.exists() {
                return Some(p);
            }
        }

        // 2. Check next to executable (bundled like FFmpeg)
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(parent) = exe_path.parent() {
                let candidates = [
                    parent.join("binaries").join("whisper-cli.exe"),
                    parent.join("binaries").join("whisper-cli"),
                    parent.join("whisper-cli.exe"),
                    parent.join("whisper-cli"),
                ];
                for c in &candidates {
                    if c.exists() {
                        return Some(c.clone());
                    }
                }
            }
        }

        None
    }

    fn find_model() -> Option<PathBuf> {
        // 1. Check env var
        if let Ok(path) = std::env::var("TTS_VUE_NEXT_WHISPER_MODEL") {
            let p = PathBuf::from(&path);
            if p.exists() {
                return Some(p);
            }
        }

        // 2. Check bundled
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(parent) = exe_path.parent() {
                let candidates = [
                    parent.join("binaries").join("ggml-base.en.bin"),
                    parent.join("ggml-base.en.bin"),
                ];
                for c in &candidates {
                    if c.exists() {
                        return Some(c.clone());
                    }
                }
            }
        }

        // 3. Check current directory
        let cwd = std::env::current_dir().ok()?;
        let p = cwd.join("ggml-base.en.bin");
        if p.exists() {
            return Some(p);
        }

        None
    }

    pub fn is_available() -> bool {
        Self::find_binary().is_some() && Self::find_model().is_some()
    }

    /// Run whisper-cli with explicit binary and model paths
    async fn run_with_paths(
        binary: &PathBuf,
        model: &PathBuf,
        audio_data: &[u8],
    ) -> Result<Transcript, RecognitionError> {
        let tmp_dir = std::env::temp_dir();
        let input_path = tmp_dir.join(format!("whisper-input-{}.wav", uuid::Uuid::new_v4()));

        tokio::fs::write(&input_path, audio_data).await
            .map_err(|e| RecognitionError::Internal(format!("Failed to write temp audio: {e}")))?;

        let binary_str = binary.to_string_lossy().to_string();
        let model_str = model.to_string_lossy().to_string();
        let input_str = input_path.to_string_lossy().to_string();
        let output_dir = tmp_dir.to_string_lossy().to_string();

        let result = tokio::task::spawn_blocking(move || {
            let output = std::process::Command::new(&binary_str)
                .args([
                    "--model", &model_str,
                    "--file", &input_str,
                    "--output-txt",
                    "--output-dir", &output_dir,
                    "--no-prints",
                ])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output();

            // Clean up input file
            let _ = std::fs::remove_file(&input_path);

            match output {
                Ok(out) => {
                    if !out.status.success() {
                        let stderr_str = String::from_utf8_lossy(&out.stderr);
                        Err(RecognitionError::TranscriptionFailed(
                            format!("whisper-cli exited with code {}: {stderr_str}", out.status)
                        ))
                    } else {
                        // Try reading the output txt file
                        let txt_path = input_path.with_extension("txt");
                        match std::fs::read_to_string(&txt_path) {
                            Ok(text) => {
                                let _ = std::fs::remove_file(&txt_path);
                                Ok(transcript_from_text(&text))
                            }
                            Err(_) => {
                                let stdout = String::from_utf8_lossy(&out.stdout);
                                Ok(transcript_from_text(&stdout))
                            }
                        }
                    }
                }
                Err(e) => Err(RecognitionError::Internal(format!(
                    "Failed to launch whisper-cli: {e}"
                ))),
            }
        }).await
        .map_err(|e| RecognitionError::Internal(format!("Task error: {e}")))?;

        result
    }

    /// Write audio to a temp WAV file, run whisper-cli, return transcript
    /// Uses auto-detected binary and model paths.
    async fn run_whisper_cli(
        audio_data: &[u8],
    ) -> Result<Transcript, RecognitionError> {
        let binary = Self::find_binary()
            .ok_or_else(|| RecognitionError::ModelNotLoaded(
                "whisper-cli not found. Set TTS_VUE_NEXT_WHISPER_PATH or bundle the binary.".to_string()
            ))?;
        let model = Self::find_model()
            .ok_or_else(|| RecognitionError::ModelNotLoaded(
                "ggml model not found. Set TTS_VUE_NEXT_WHISPER_MODEL or place ggml-base.en.bin in binaries/.".to_string()
            ))?;

        Self::run_with_paths(&binary, &model, audio_data).await
    }
}

#[async_trait::async_trait]
impl SpeechRecognizer for ConfiguredRecognizer {
    async fn transcribe(&self, audio: &Audio) -> Result<Transcript, RecognitionError> {
        CliRecognizer::run_with_paths(&self.binary_path, &self.model_path, &audio.data).await
    }
}

fn transcript_from_text(text: &str) -> Transcript {
    let segments = vec![Segment {
        start: 0.0,
        end: text.len() as f64 * 0.1, // rough estimate
        text: text.trim().to_string(),
    }];

    Transcript {
        text: text.trim().to_string(),
        segments,
    }
}

#[async_trait::async_trait]
impl SpeechRecognizer for CliRecognizer {
    async fn transcribe(&self, audio: &Audio) -> Result<Transcript, RecognitionError> {
        Self::run_whisper_cli(&audio.data).await
    }
}

/// Dummy recognizer for development/testing
///
/// Returns the reference text as a "perfect" transcript, useful when
/// testing the scoring pipeline without Whisper installed.
pub struct DummyRecognizer {
    pub delay_ms: u64,
}

impl DummyRecognizer {
    pub fn new() -> Self {
        DummyRecognizer { delay_ms: 0 }
    }

    pub fn with_delay(delay_ms: u64) -> Self {
        DummyRecognizer { delay_ms }
    }
}

impl Default for DummyRecognizer {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl SpeechRecognizer for DummyRecognizer {
    async fn transcribe(&self, _audio: &Audio) -> Result<Transcript, RecognitionError> {
        if self.delay_ms > 0 {
            tokio::time::sleep(std::time::Duration::from_millis(self.delay_ms)).await;
        }
        Ok(Transcript {
            text: "dummy transcription result".to_string(),
            segments: vec![Segment {
                start: 0.0,
                end: 1.0,
                text: "dummy transcription result".to_string(),
            }],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transcript_from_text() {
        let t = transcript_from_text("hello world");
        assert_eq!(t.text, "hello world");
        assert_eq!(t.segments.len(), 1);
    }

    #[test]
    fn test_dummy_recognizer() {
        let rec = DummyRecognizer::new();
        let audio = Audio {
            data: vec![],
            sample_rate: 16000,
            channels: 1,
            duration_ms: 1000,
        };

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(rec.transcribe(&audio)).unwrap();
        assert!(!result.text.is_empty());
    }

    #[test]
    fn test_cli_recognizer_availability() {
        // Just check it doesn't panic
        let _available = CliRecognizer::is_available();
    }
}
