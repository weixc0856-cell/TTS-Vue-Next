use practice_domain::{ScoreResult, WordScore, WordStatus};
use speech_engine::Audio;

use crate::{AssessmentError, PronunciationAssessor};

/// Word Error Rate based assessor
///
/// Computes WER between reference text and Whisper-transcribed text.
/// Provides word-level accuracy, completeness, and overall score.
/// No phoneme analysis — pure text matching suitable for MVP Phase 1.
pub struct WerAssessor {
    recognizer: Box<dyn speech_engine::SpeechRecognizer + Send + Sync>,
}

impl WerAssessor {
    pub fn new(recognizer: Box<dyn speech_engine::SpeechRecognizer + Send + Sync>) -> Self {
        WerAssessor { recognizer }
    }

    /// Normalize text for comparison (lowercase, trim punctuation)
    fn normalize(text: &str) -> String {
        text.to_lowercase()
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace())
            .collect::<String>()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Split text into words
    fn words(text: &str) -> Vec<String> {
        Self::normalize(text)
            .split_whitespace()
            .map(|s| s.to_string())
            .collect()
    }

    /// Compute Levenshtein distance on word sequences
    fn word_edit_distance(ref_words: &[String], hyp_words: &[String]) -> Vec<Vec<usize>> {
        let ref_len = ref_words.len();
        let hyp_len = hyp_words.len();
        let mut dp = vec![vec![0usize; hyp_len + 1]; ref_len + 1];

        for i in 0..=ref_len {
            dp[i][0] = i;
        }
        for j in 0..=hyp_len {
            dp[0][j] = j;
        }

        for i in 1..=ref_len {
            for j in 1..=hyp_len {
                let cost = if ref_words[i - 1] == hyp_words[j - 1] {
                    0
                } else {
                    1
                };
                dp[i][j] = (dp[i - 1][j] + 1)          // deletion
                    .min(dp[i][j - 1] + 1)              // insertion
                    .min(dp[i - 1][j - 1] + cost);      // substitution
            }
        }

        dp
    }

    /// Compute per-word status by tracing the edit distance back
    fn compute_word_scores(
        ref_words: &[String],
        hyp_words: &[String],
        dp: &[Vec<usize>],
    ) -> (Vec<WordScore>, f64) {
        let mut word_scores = Vec::new();
        let mut i = ref_words.len();
        let mut j = hyp_words.len();
        let mut correct = 0;

        // Trace back through DP matrix
        while i > 0 || j > 0 {
            if i > 0 && j > 0 && ref_words[i - 1] == hyp_words[j - 1] {
                word_scores.push(WordScore {
                    word: ref_words[i - 1].clone(),
                    status: WordStatus::Correct,
                    confidence: 1.0,
                });
                correct += 1;
                i -= 1;
                j -= 1;
            } else if j > 0 && (i == 0 || dp[i][j - 1] + 1 == dp[i][j]) {
                // Extra word in hypothesis
                word_scores.push(WordScore {
                    word: hyp_words[j - 1].clone(),
                    status: WordStatus::Extra,
                    confidence: 0.0,
                });
                j -= 1;
            } else if i > 0 && (j == 0 || dp[i - 1][j] + 1 == dp[i][j]) {
                // Omitted word in reference
                word_scores.push(WordScore {
                    word: ref_words[i - 1].clone(),
                    status: WordStatus::Omitted,
                    confidence: 0.0,
                });
                i -= 1;
            } else {
                // Substitution
                word_scores.push(WordScore {
                    word: ref_words[i - 1].clone(),
                    status: WordStatus::Wrong,
                    confidence: 0.0,
                });
                i -= 1;
                j -= 1;
            }
        }

        word_scores.reverse();
        let accuracy = if ref_words.is_empty() {
            1.0
        } else {
            correct as f64 / ref_words.len() as f64
        };

        (word_scores, accuracy)
    }
}

#[async_trait::async_trait]
impl PronunciationAssessor for WerAssessor {
    async fn evaluate(
        &self,
        reference: &str,
        audio: &Audio,
    ) -> Result<ScoreResult, AssessmentError> {
        if reference.trim().is_empty() {
            return Err(AssessmentError::EmptyReference);
        }

        // Transcribe the audio
        let transcript = self
            .recognizer
            .transcribe(audio)
            .await
            .map_err(|e| AssessmentError::RecognitionFailed(e.to_string()))?;

        let ref_words = Self::words(reference);
        let hyp_words = Self::words(&transcript.text);

        if ref_words.is_empty() {
            return Ok(ScoreResult {
                overall: 100.0,
                accuracy: 100.0,
                completeness: 100.0,
                word_scores: vec![],
            });
        }

        // Compute edit distance
        let dp = Self::word_edit_distance(&ref_words, &hyp_words);

        // Compute WER
        let wer = dp[ref_words.len()][hyp_words.len()] as f64 / ref_words.len() as f64;
        let accuracy = (1.0 - wer).max(0.0);

        // Compute per-word scores
        let (word_scores, _) = Self::compute_word_scores(&ref_words, &hyp_words, &dp);

        // Count omissions
        let omitted = word_scores
            .iter()
            .filter(|ws| matches!(ws.status, WordStatus::Omitted))
            .count();
        let completeness = 1.0 - (omitted as f64 / ref_words.len() as f64);

        // Overall: weighted combination
        let overall = (accuracy * 0.6 + completeness * 0.4) * 100.0;

        Ok(ScoreResult {
            overall: overall.round(),
            accuracy: (accuracy * 100.0).round(),
            completeness: (completeness * 100.0).round(),
            word_scores,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockRecognizer;
    #[async_trait::async_trait]
    impl speech_engine::SpeechRecognizer for MockRecognizer {
        async fn transcribe(
            &self,
            _audio: &speech_engine::Audio,
        ) -> Result<speech_engine::Transcript, speech_engine::RecognitionError> {
            // Returns a perfect transcript for testing
            Ok(speech_engine::Transcript {
                text: "hello world".to_string(),
                segments: vec![],
            })
        }
    }

    #[tokio::test]
    async fn test_perfect_match() {
        let assessor = WerAssessor::new(Box::new(MockRecognizer));
        let audio = Audio {
            data: vec![],
            sample_rate: 16000,
            channels: 1,
            duration_ms: 1000,
        };

        let result = assessor.evaluate("hello world", &audio).await.unwrap();
        assert_eq!(result.overall, 100.0);
        assert_eq!(result.accuracy, 100.0);
        assert_eq!(result.completeness, 100.0);
        assert_eq!(result.word_scores.len(), 2);
        assert!(result.word_scores.iter().all(|w| matches!(w.status, WordStatus::Correct)));
    }

    #[tokio::test]
    async fn test_partial_match() {
        struct PartialRecognizer;
        #[async_trait::async_trait]
        impl speech_engine::SpeechRecognizer for PartialRecognizer {
            async fn transcribe(
                &self,
                _audio: &speech_engine::Audio,
            ) -> Result<speech_engine::Transcript, speech_engine::RecognitionError> {
                Ok(speech_engine::Transcript {
                    text: "hello".to_string(),
                    segments: vec![],
                })
            }
        }

        let assessor = WerAssessor::new(Box::new(PartialRecognizer));
        let audio = Audio {
            data: vec![],
            sample_rate: 16000,
            channels: 1,
            duration_ms: 1000,
        };

        let result = assessor.evaluate("hello world", &audio).await.unwrap();
        assert!(result.overall < 100.0);
        assert_eq!(result.word_scores.len(), 2);

        // "hello" should be correct, "world" should be omitted
        assert_eq!(result.word_scores[0].status, WordStatus::Correct);
    }

    #[tokio::test]
    async fn test_empty_reference_error() {
        let assessor = WerAssessor::new(Box::new(MockRecognizer));
        let audio = Audio {
            data: vec![],
            sample_rate: 16000,
            channels: 1,
            duration_ms: 1000,
        };

        let result = assessor.evaluate("", &audio).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_word_edit_distance_identical() {
        let ref_words = vec!["hello".to_string(), "world".to_string()];
        let hyp_words = vec!["hello".to_string(), "world".to_string()];

        let dp = WerAssessor::word_edit_distance(&ref_words, &hyp_words);
        assert_eq!(dp[ref_words.len()][hyp_words.len()], 0);
    }

    #[test]
    fn test_word_edit_distance_different() {
        let ref_words = vec!["hello".to_string(), "world".to_string()];
        let hyp_words = vec!["goodbye".to_string(), "world".to_string()];

        let dp = WerAssessor::word_edit_distance(&ref_words, &hyp_words);
        assert_eq!(dp[ref_words.len()][hyp_words.len()], 1);
    }

    #[test]
    fn test_normalize_text() {
        assert_eq!(
            WerAssessor::normalize("Hello, World!"),
            "hello world"
        );
        assert_eq!(
            WerAssessor::normalize("I'm fine, thank you."),
            "im fine thank you"
        );
    }
}
