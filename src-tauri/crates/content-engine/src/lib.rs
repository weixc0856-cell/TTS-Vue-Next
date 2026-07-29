use std::path::Path;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use practice_domain::*;
use storage::Storage;

/// Errors from content operations
#[derive(Debug, Error)]
pub enum ContentError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Storage error: {0}")]
    Storage(#[from] storage::StorageError),
    #[error("Scenario not found: {0}")]
    NotFound(String),
    #[error("Invalid scenario format: {0}")]
    InvalidFormat(String),
}

/// JSON format for a scenario file
#[derive(Debug, Serialize, Deserialize)]
pub struct ScenarioFile {
    pub id: String,
    pub title: String,
    pub mode: String,
    pub difficulty: String,
    pub category: String,
    pub sentences: Vec<ScenarioSentence>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScenarioSentence {
    pub text: String,
    pub translation: Option<String>,
}

/// JSON format for a dialogue file
#[derive(Debug, Serialize, Deserialize)]
pub struct DialogueFile {
    pub id: String,
    pub title: String,
    pub difficulty: String,
    pub category: String,
    pub exchanges: Vec<ScenarioExchange>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScenarioExchange {
    pub role: String,
    pub text: String,
    pub translation: Option<String>,
}

/// Content manager: loads scenarios and dialogues from JSON files
pub struct ContentManager {
    storage: Storage,
}

impl ContentManager {
    pub fn new(storage: Storage) -> Self {
        ContentManager { storage }
    }

    /// Get reference to underlying storage
    pub fn get_storage(&self) -> &Storage {
        &self.storage
    }

    /// Get mutable reference to underlying storage
    pub fn get_storage_mut(&mut self) -> &mut Storage {
        &mut self.storage
    }

    /// Seed built-in scenarios from a directory of JSON files
    /// Skips exercises that already exist in the database
    pub fn seed_from_directory(&self, dir: &Path) -> Result<Vec<String>, ContentError> {
        let mut seeded = Vec::new();

        if !dir.exists() {
            return Ok(seeded);
        }

        let mut entries: Vec<_> = std::fs::read_dir(dir)?
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path().extension().and_then(|ext| ext.to_str()) == Some("json")
            })
            .collect();
        entries.sort_by_key(|e| e.path());

        for entry in entries {
            let content = std::fs::read_to_string(entry.path())?;
            let path_str = entry.path().to_string_lossy().to_string();

            // Try parsing as exercise scenario first
            if let Ok(scenario) = serde_json::from_str::<ScenarioFile>(&content) {
                if let Err(_) = self.storage.get_exercise(&scenario.id) {
                    let exercise = self.scenario_to_exercise(&scenario);
                    self.storage.insert_exercise(&exercise)?;
                    self.storage.insert_sentences(&exercise.id, &exercise.sentences)?;
                    seeded.push(scenario.id);
                }
            }
            // Try parsing as dialogue (roleplay) scenario
            else if let Ok(dialogue) = serde_json::from_str::<DialogueFile>(&content) {
                let id = format!("dialogue-{}", dialogue.id);
                if let Err(_) = self.storage.get_exercise(&id) {
                    let exercise = self.dialogue_to_exercise(&dialogue, &id);
                    self.storage.insert_exercise(&exercise)?;
                    self.storage.insert_sentences(&exercise.id, &exercise.sentences)?;
                    seeded.push(id);
                }
            } else {
                log::warn!("Skipping unrecognized scenario file: {path_str}");
            }
        }

        Ok(seeded)
    }

    fn scenario_to_exercise(&self, file: &ScenarioFile) -> Exercise {
        let mode = PracticeMode::from_str(&file.mode).unwrap_or(PracticeMode::Shadowing);
        let difficulty = Difficulty::from_str(&file.difficulty).unwrap_or(Difficulty::Beginner);
        let category =
            ScenarioCategory::from_str(&file.category).unwrap_or(ScenarioCategory::Daily);

        let sentences: Vec<Sentence> = file
            .sentences
            .iter()
            .enumerate()
            .map(|(i, s)| Sentence {
                id: format!("{}-s{}", file.id, i),
                text: s.text.clone(),
                translation: s.translation.clone(),
                order_index: i as i32,
            })
            .collect();

        Exercise {
            id: file.id.clone(),
            title: file.title.clone(),
            mode,
            difficulty,
            category,
            source: ExerciseSource::Builtin,
            sentences,
            metadata: serde_json::json!({}),
        }
    }

    fn dialogue_to_exercise(&self, file: &DialogueFile, id: &str) -> Exercise {
        let difficulty = Difficulty::from_str(&file.difficulty).unwrap_or(Difficulty::Beginner);
        let category =
            ScenarioCategory::from_str(&file.category).unwrap_or(ScenarioCategory::Daily);

        let sentences: Vec<Sentence> = file
            .exchanges
            .iter()
            .enumerate()
            .map(|(i, e)| Sentence {
                id: format!("{}-e{}", id, i),
                text: format!("{}: {}", e.role, e.text),
                translation: e.translation.clone(),
                order_index: i as i32,
            })
            .collect();

        Exercise {
            id: id.to_string(),
            title: file.title.clone(),
            mode: PracticeMode::Roleplay,
            difficulty,
            category,
            source: ExerciseSource::Builtin,
            sentences,
            metadata: serde_json::json!({}),
        }
    }

    /// List all available exercises with optional filters
    pub fn list_exercises(
        &self,
        mode: Option<&str>,
        category: Option<&str>,
        difficulty: Option<&str>,
    ) -> Result<Vec<Exercise>, ContentError> {
        let mut exercises = self.storage.list_exercises(mode, category, difficulty)?;

        // Load sentences for each exercise
        for ex in &mut exercises {
            let sentences = self.storage.get_sentences(&ex.id)?;
            ex.sentences = sentences;
        }

        Ok(exercises)
    }

    /// Get full exercise detail including all sentences
    pub fn get_exercise(&self, id: &str) -> Result<Exercise, ContentError> {
        let mut exercise = self.storage.get_exercise(id)?;
        exercise.sentences = self.storage.get_sentences(id)?;
        Ok(exercise)
    }

    /// Create a custom exercise from user-provided text
    pub fn create_custom_exercise(
        &self,
        title: &str,
        sentences: &[String],
        translations: &[Option<String>],
    ) -> Result<Exercise, ContentError> {
        let id = format!("custom-{}", uuid::Uuid::new_v4());

        let sentence_objs: Vec<Sentence> = sentences
            .iter()
            .enumerate()
            .map(|(i, text)| Sentence {
                id: format!("{}-s{}", id, i),
                text: text.clone(),
                translation: translations.get(i).cloned().unwrap_or(None),
                order_index: i as i32,
            })
            .collect();

        let exercise = Exercise {
            id: id.clone(),
            title: title.to_string(),
            mode: PracticeMode::Shadowing,
            difficulty: Difficulty::Beginner,
            category: ScenarioCategory::Daily,
            source: ExerciseSource::User,
            sentences: sentence_objs.clone(),
            metadata: serde_json::json!({}),
        };

        self.storage.insert_exercise(&exercise)?;
        self.storage.insert_sentences(&id, &sentence_objs)?;

        Ok(exercise)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use storage::Storage;

    #[test]
    fn test_scenario_file_parsing() {
        let json = r#"{
            "id": "test-shadow-1",
            "title": "Test Scenario",
            "mode": "shadowing",
            "difficulty": "beginner",
            "category": "daily",
            "sentences": [
                {"text": "Hello, how are you?", "translation": "你好"},
                {"text": "I am fine, thank you.", "translation": "我很好"}
            ]
        }"#;

        let scenario: ScenarioFile = serde_json::from_str(json).unwrap();
        assert_eq!(scenario.id, "test-shadow-1");
        assert_eq!(scenario.sentences.len(), 2);
        assert_eq!(scenario.sentences[0].text, "Hello, how are you?");
    }

    #[test]
    fn test_dialogue_file_parsing() {
        let json = r#"{
            "id": "test-dialogue-1",
            "title": "Test Dialogue",
            "difficulty": "beginner",
            "category": "travel",
            "exchanges": [
                {"role": "A", "text": "Welcome!", "translation": "欢迎"},
                {"role": "B", "text": "Thank you!", "translation": "谢谢"}
            ]
        }"#;

        let dialogue: DialogueFile = serde_json::from_str(json).unwrap();
        assert_eq!(dialogue.id, "test-dialogue-1");
        assert_eq!(dialogue.exchanges.len(), 2);
    }

    #[test]
    fn test_create_custom_exercise() {
        let storage = Storage::open_in_memory().unwrap();
        let manager = ContentManager::new(storage);

        let texts = vec![
            "Hello world".to_string(),
            "How are you?".to_string(),
        ];
        let translations = vec![
            Some("你好世界".to_string()),
            None,
        ];

        let exercise = manager
            .create_custom_exercise("My Custom", &texts, &translations)
            .unwrap();

        assert_eq!(exercise.title, "My Custom");
        assert_eq!(exercise.sentences.len(), 2);
        assert_eq!(exercise.source, ExerciseSource::User);

        // Verify it was persisted
        let retrieved = manager.get_exercise(&exercise.id).unwrap();
        assert_eq!(retrieved.sentences.len(), 2);
    }
}
