use rusqlite::{Connection, params};
use serde_json;
use thiserror::Error;

use practice_domain::*;

/// Errors from the storage layer
#[derive(Debug, Error)]
pub enum StorageError {
    #[error("Database error: {0}")]
    Db(#[from] rusqlite::Error),
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("Not found: {0}")]
    NotFound(String),
}

/// SQLite-backed storage for practice data
pub struct Storage {
    conn: Connection,
}

/// Session summary for list display
#[derive(Debug, Clone, serde::Serialize)]
pub struct SessionSummary {
    pub id: String,
    pub exercise_title: String,
    pub mode: String,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub overall_score: Option<f64>,
    pub sentence_count: i64,
    pub attempt_count: i64,
}

/// Aggregate practice statistics
#[derive(Debug, Clone, serde::Serialize)]
pub struct PracticeStats {
    pub total_sessions: i64,
    pub total_attempts: i64,
    pub average_score: Option<f64>,
    pub total_practice_minutes: f64,
    pub best_score: Option<f64>,
    pub recent_scores: Vec<f64>,
}

impl Storage {
    /// Open or create the database at `path` and run migrations
    pub fn open(path: &str) -> Result<Self, StorageError> {
        let conn = Connection::open(path)?;
        let mut store = Storage { conn };
        store.run_migrations()?;
        Ok(store)
    }

    /// Open an in-memory database (for testing)
    pub fn open_in_memory() -> Result<Self, StorageError> {
        let conn = Connection::open_in_memory()?;
        let mut store = Storage { conn };
        store.run_migrations()?;
        Ok(store)
    }

    fn run_migrations(&mut self) -> Result<(), StorageError> {
        let tx = self.conn.transaction()?;

        tx.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS exercises (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                mode TEXT NOT NULL CHECK(mode IN ('shadowing','roleplay')),
                difficulty TEXT NOT NULL CHECK(difficulty IN ('beginner','intermediate','advanced')),
                category TEXT NOT NULL CHECK(category IN ('daily','travel','business','academic','exam')),
                source TEXT NOT NULL DEFAULT 'builtin' CHECK(source IN ('builtin','user','document')),
                metadata TEXT DEFAULT '{}',
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS sentences (
                id TEXT PRIMARY KEY,
                exercise_id TEXT NOT NULL REFERENCES exercises(id) ON DELETE CASCADE,
                text TEXT NOT NULL,
                translation TEXT,
                order_index INTEGER NOT NULL,
                expected_duration_ms INTEGER
            );
            CREATE INDEX IF NOT EXISTS idx_sentences_exercise ON sentences(exercise_id);

            CREATE TABLE IF NOT EXISTS practice_sessions (
                id TEXT PRIMARY KEY,
                exercise_id TEXT NOT NULL REFERENCES exercises(id),
                mode TEXT NOT NULL,
                started_at TEXT NOT NULL DEFAULT (datetime('now')),
                completed_at TEXT,
                overall_score REAL
            );
            CREATE INDEX IF NOT EXISTS idx_sessions_exercise ON practice_sessions(exercise_id);

            CREATE TABLE IF NOT EXISTS attempts (
                id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL REFERENCES practice_sessions(id) ON DELETE CASCADE,
                sentence_id TEXT NOT NULL REFERENCES sentences(id),
                audio_path TEXT,
                transcript TEXT,
                score REAL,
                word_scores TEXT,
                fluency REAL,
                completeness REAL,
                duration_ms INTEGER,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE INDEX IF NOT EXISTS idx_attempts_session ON attempts(session_id);
            ",
        )?;

        tx.commit()?;
        Ok(())
    }

    // ── Exercises ──

    pub fn insert_exercise(&self, exercise: &Exercise) -> Result<(), StorageError> {
        let mut stmt = self.conn.prepare_cached(
            "INSERT INTO exercises (id, title, mode, difficulty, category, source, metadata)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        )?;
        stmt.execute(params![
            exercise.id,
            exercise.title,
            exercise.mode.as_str(),
            exercise.difficulty.as_str(),
            exercise.category.as_str(),
            exercise.source.as_str(),
            exercise.metadata.to_string(),
        ])?;
        Ok(())
    }

    pub fn list_exercises(
        &self,
        mode: Option<&str>,
        category: Option<&str>,
        difficulty: Option<&str>,
    ) -> Result<Vec<Exercise>, StorageError> {
        let mut sql = String::from(
            "SELECT id, title, mode, difficulty, category, source, metadata, created_at FROM exercises WHERE 1=1",
        );
        let mut param_values: Vec<String> = Vec::new();

        if let Some(m) = mode {
            param_values.push(m.to_string());
            sql.push_str(&format!(" AND mode = ?{}", param_values.len()));
        }
        if let Some(c) = category {
            param_values.push(c.to_string());
            sql.push_str(&format!(" AND category = ?{}", param_values.len()));
        }
        if let Some(d) = difficulty {
            param_values.push(d.to_string());
            sql.push_str(&format!(" AND difficulty = ?{}", param_values.len()));
        }
        sql.push_str(" ORDER BY created_at DESC");

        let mut stmt = self.conn.prepare_cached(&sql)?;
        let params_refs: Vec<&dyn rusqlite::types::ToSql> =
            param_values.iter().map(|v| v as &dyn rusqlite::types::ToSql).collect();

        let rows = stmt.query_map(params_refs.as_slice(), |row| {
            let mode_str: String = row.get(2)?;
            let diff_str: String = row.get(3)?;
            let cat_str: String = row.get(4)?;
            let source_str: String = row.get(5)?;
            let meta_str: String = row.get(6)?;

            Ok(Exercise {
                id: row.get(0)?,
                title: row.get(1)?,
                mode: PracticeMode::from_str(&mode_str).unwrap_or(PracticeMode::Shadowing),
                difficulty: Difficulty::from_str(&diff_str).unwrap_or(Difficulty::Beginner),
                category: ScenarioCategory::from_str(&cat_str).unwrap_or(ScenarioCategory::Daily),
                source: ExerciseSource::from_str(&source_str).unwrap_or(ExerciseSource::Builtin),
                metadata: serde_json::from_str(&meta_str).unwrap_or_default(),
                sentences: Vec::new(), // sentences loaded separately
            })
        })?;

        let mut exercises = Vec::new();
        for row in rows {
            exercises.push(row?);
        }
        Ok(exercises)
    }

    pub fn get_exercise(&self, id: &str) -> Result<Exercise, StorageError> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT id, title, mode, difficulty, category, source, metadata, created_at
             FROM exercises WHERE id = ?1",
        )?;

        let exercise = stmt
            .query_row(params![id], |row| {
                let mode_str: String = row.get(2)?;
                let diff_str: String = row.get(3)?;
                let cat_str: String = row.get(4)?;
                let source_str: String = row.get(5)?;
                let meta_str: String = row.get(6)?;

                Ok(Exercise {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    mode: PracticeMode::from_str(&mode_str).unwrap_or(PracticeMode::Shadowing),
                    difficulty: Difficulty::from_str(&diff_str).unwrap_or(Difficulty::Beginner),
                    category: ScenarioCategory::from_str(&cat_str)
                        .unwrap_or(ScenarioCategory::Daily),
                    source: ExerciseSource::from_str(&source_str).unwrap_or(ExerciseSource::Builtin),
                    metadata: serde_json::from_str(&meta_str).unwrap_or_default(),
                    sentences: Vec::new(),
                })
            })
            .map_err(|_| StorageError::NotFound(format!("Exercise {id}")))?;

        Ok(exercise)
    }

    // ── Sentences ──

    pub fn insert_sentences(
        &self,
        exercise_id: &str,
        sentences: &[Sentence],
    ) -> Result<(), StorageError> {
        let mut stmt = self.conn.prepare_cached(
            "INSERT INTO sentences (id, exercise_id, text, translation, order_index)
             VALUES (?1, ?2, ?3, ?4, ?5)",
        )?;
        for s in sentences {
            stmt.execute(params![s.id, exercise_id, s.text, s.translation, s.order_index])?;
        }
        Ok(())
    }

    pub fn get_sentences(&self, exercise_id: &str) -> Result<Vec<Sentence>, StorageError> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT id, exercise_id, text, translation, order_index, expected_duration_ms
             FROM sentences WHERE exercise_id = ?1 ORDER BY order_index",
        )?;

        let rows = stmt.query_map(params![exercise_id], |row| {
            Ok(Sentence {
                id: row.get(0)?,
                text: row.get(2)?,
                translation: row.get(3)?,
                order_index: row.get(4)?,
            })
        })?;

        let mut sentences = Vec::new();
        for row in rows {
            sentences.push(row?);
        }
        Ok(sentences)
    }

    // ── Practice Sessions ──

    pub fn create_session(&self, session: &PracticeSession) -> Result<(), StorageError> {
        let mut stmt = self.conn.prepare_cached(
            "INSERT INTO practice_sessions (id, exercise_id, mode, started_at)
             VALUES (?1, ?2, ?3, ?4)",
        )?;
        stmt.execute(params![
            session.id,
            session.exercise_id,
            session.mode.as_str(),
            session.started_at,
        ])?;
        Ok(())
    }

    pub fn complete_session(&self, id: &str, score: f64) -> Result<(), StorageError> {
        let mut stmt = self.conn.prepare_cached(
            "UPDATE practice_sessions SET completed_at = datetime('now'), overall_score = ?1 WHERE id = ?2",
        )?;
        stmt.execute(params![score, id])?;
        Ok(())
    }

    pub fn get_session(&self, id: &str) -> Result<PracticeSession, StorageError> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT id, exercise_id, mode, started_at, completed_at, overall_score
             FROM practice_sessions WHERE id = ?1",
        )?;

        stmt.query_row(params![id], |row| {
            let mode_str: String = row.get(2)?;
            Ok(PracticeSession {
                id: row.get(0)?,
                exercise_id: row.get(1)?,
                mode: PracticeMode::from_str(&mode_str).unwrap_or(PracticeMode::Shadowing),
                started_at: row.get(3)?,
                completed_at: row.get(4)?,
                overall_score: row.get(5)?,
            })
        })
        .map_err(|_| StorageError::NotFound(format!("Session {id}")))
    }

    // ── Attempts ──

    pub fn insert_attempt(&self, attempt: &Attempt) -> Result<(), StorageError> {
        let mut stmt = self.conn.prepare_cached(
            "INSERT INTO attempts (id, session_id, sentence_id, audio_path, transcript, score, word_scores, completeness, duration_ms)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        )?;
        stmt.execute(params![
            attempt.id,
            attempt.session_id,
            attempt.sentence_id,
            attempt.audio_path,
            attempt.transcript,
            attempt.score,
            serde_json::to_string(&attempt.word_scores)?,
            attempt.completeness,
            attempt.duration_ms,
        ])?;
        Ok(())
    }

    pub fn get_attempts(&self, session_id: &str) -> Result<Vec<Attempt>, StorageError> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT id, session_id, sentence_id, audio_path, transcript, score, word_scores, completeness, duration_ms, created_at
             FROM attempts WHERE session_id = ?1 ORDER BY created_at",
        )?;

        let rows = stmt.query_map(params![session_id], |row| {
            let word_scores_str: String = row.get(6)?;
            let word_scores: Vec<WordScore> =
                serde_json::from_str(&word_scores_str).unwrap_or_default();

            Ok(Attempt {
                id: row.get(0)?,
                session_id: row.get(1)?,
                sentence_id: row.get(2)?,
                audio_path: row.get(3)?,
                transcript: row.get(4)?,
                score: row.get(5)?,
                word_scores,
                completeness: row.get(7)?,
                duration_ms: row.get(8)?,
            })
        })?;

        let mut attempts = Vec::new();
        for row in rows {
            attempts.push(row?);
        }
        Ok(attempts)
    }

    pub fn get_session_history(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<SessionSummary>, StorageError> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT
                ps.id,
                COALESCE(e.title, 'Unknown') as exercise_title,
                ps.mode,
                ps.started_at,
                ps.completed_at,
                ps.overall_score,
                (SELECT COUNT(*) FROM sentences s WHERE s.exercise_id = ps.exercise_id) as sentence_count,
                (SELECT COUNT(*) FROM attempts a WHERE a.session_id = ps.id) as attempt_count
             FROM practice_sessions ps
             LEFT JOIN exercises e ON e.id = ps.exercise_id
             ORDER BY ps.started_at DESC
             LIMIT ?1 OFFSET ?2",
        )?;

        let rows = stmt.query_map(params![limit, offset], |row| {
            Ok(SessionSummary {
                id: row.get(0)?,
                exercise_title: row.get(1)?,
                mode: row.get(2)?,
                started_at: row.get(3)?,
                completed_at: row.get(4)?,
                overall_score: row.get(5)?,
                sentence_count: row.get(6)?,
                attempt_count: row.get(7)?,
            })
        })?;

        let mut sessions = Vec::new();
        for row in rows {
            sessions.push(row?);
        }
        Ok(sessions)
    }

    pub fn get_practice_stats(&self) -> Result<PracticeStats, StorageError> {
        let total_sessions: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM practice_sessions",
            [],
            |row| row.get(0),
        )?;

        let total_attempts: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM attempts",
            [],
            |row| row.get(0),
        )?;

        let average_score: Option<f64> = self.conn.query_row(
            "SELECT AVG(overall_score) FROM practice_sessions WHERE overall_score IS NOT NULL",
            [],
            |row| row.get(0),
        )?;

        let total_duration_ms: Option<i64> = self.conn.query_row(
            "SELECT SUM(duration_ms) FROM attempts",
            [],
            |row| row.get(0),
        )?;

        let best_score: Option<f64> = self.conn.query_row(
            "SELECT MAX(overall_score) FROM practice_sessions WHERE overall_score IS NOT NULL",
            [],
            |row| row.get(0),
        )?;

        let total_practice_minutes = total_duration_ms.unwrap_or(0) as f64 / 60_000.0;

        // Last 10 scores for trend
        let mut stmt = self.conn.prepare_cached(
            "SELECT overall_score FROM practice_sessions
             WHERE overall_score IS NOT NULL
             ORDER BY started_at DESC LIMIT 10",
        )?;
        let scores: Vec<f64> = stmt
            .query_map([], |row| row.get::<_, f64>(0))?
            .filter_map(|r| r.ok())
            .collect();

        Ok(PracticeStats {
            total_sessions,
            total_attempts,
            average_score,
            total_practice_minutes,
            best_score,
            recent_scores: scores,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn create_test_exercise() -> Exercise {
        Exercise {
            id: "test-1".to_string(),
            title: "Test Exercise".to_string(),
            mode: PracticeMode::Shadowing,
            difficulty: Difficulty::Beginner,
            category: ScenarioCategory::Daily,
            source: ExerciseSource::Builtin,
            sentences: vec![Sentence {
                id: "s1".to_string(),
                text: "Hello, how are you?".to_string(),
                translation: Some("你好，你好吗?".to_string()),
                order_index: 0,
            }],
            metadata: serde_json::json!({}),
        }
    }

    #[test]
    fn test_migrations_run_successfully() {
        let store = Storage::open_in_memory().unwrap();
        let exercise = create_test_exercise();
        store.insert_exercise(&exercise).unwrap();
        let exercises = store.list_exercises(None, None, None).unwrap();
        assert_eq!(exercises.len(), 1);
        assert_eq!(exercises[0].title, "Test Exercise");
    }

    fn insert_test_sentences(store: &Storage, exercise_id: &str) {
        let sentences = vec![
            Sentence {
                id: "s1".to_string(),
                text: "Hello world".to_string(),
                translation: Some("你好世界".to_string()),
                order_index: 0,
            },
            Sentence {
                id: "s2".to_string(),
                text: "How are you?".to_string(),
                translation: None,
                order_index: 1,
            },
        ];
        store.insert_sentences(exercise_id, &sentences).unwrap();
    }

    #[test]
    fn test_insert_and_retrieve_sentences() {
        let store = Storage::open_in_memory().unwrap();
        let exercise = create_test_exercise();
        store.insert_exercise(&exercise).unwrap();
        insert_test_sentences(&store, "test-1");

        let retrieved = store.get_sentences("test-1").unwrap();
        assert_eq!(retrieved.len(), 2);
        assert_eq!(retrieved[0].text, "Hello world");
        assert_eq!(retrieved[1].text, "How are you?");
    }

    #[test]
    fn test_session_lifecycle() {
        let store = Storage::open_in_memory().unwrap();
        let exercise = create_test_exercise();
        store.insert_exercise(&exercise).unwrap();

        let session = PracticeSession {
            id: "session-1".to_string(),
            exercise_id: "test-1".to_string(),
            mode: PracticeMode::Shadowing,
            started_at: "2026-01-01T00:00:00Z".to_string(),
            completed_at: None,
            overall_score: None,
        };

        store.create_session(&session).unwrap();
        let retrieved = store.get_session("session-1").unwrap();
        assert_eq!(retrieved.id, "session-1");
        assert_eq!(retrieved.mode, PracticeMode::Shadowing);

        store.complete_session("session-1", 85.0).unwrap();
        let completed = store.get_session("session-1").unwrap();
        assert!(completed.completed_at.is_some());
        assert_eq!(completed.overall_score, Some(85.0));
    }

    #[test]
    fn test_session_history_returns_sessions() {
        let store = Storage::open_in_memory().unwrap();
        let exercise = create_test_exercise();
        store.insert_exercise(&exercise).unwrap();

        let session = PracticeSession {
            id: "hist-1".to_string(),
            exercise_id: "test-1".to_string(),
            mode: PracticeMode::Shadowing,
            started_at: "2026-06-01T10:00:00Z".to_string(),
            completed_at: None,
            overall_score: None,
        };
        store.create_session(&session).unwrap();
        store.complete_session("hist-1", 85.0).unwrap();

        let history = store.get_session_history(10, 0).unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].overall_score, Some(85.0));
        assert_eq!(history[0].exercise_title, "Test Exercise");
    }

    #[test]
    fn test_practice_stats_aggregates_correctly() {
        let store = Storage::open_in_memory().unwrap();
        let exercise = create_test_exercise();
        store.insert_exercise(&exercise).unwrap();

        // Create a session with attempts
        let session = PracticeSession {
            id: "stats-1".to_string(),
            exercise_id: "test-1".to_string(),
            mode: PracticeMode::Shadowing,
            started_at: "2026-06-01T10:00:00Z".to_string(),
            completed_at: None,
            overall_score: None,
        };
        insert_test_sentences(&store, "test-1");
        store.create_session(&session).unwrap();
        store.complete_session("stats-1", 85.0).unwrap();

        let attempt = Attempt {
            id: "a1".to_string(),
            session_id: "stats-1".to_string(),
            sentence_id: "s1".to_string(),
            audio_path: None,
            transcript: Some("hello world".to_string()),
            score: Some(85.0),
            word_scores: vec![],
            completeness: Some(100.0),
            duration_ms: Some(5000),
        };
        store.insert_attempt(&attempt).unwrap();

        let stats = store.get_practice_stats().unwrap();
        assert_eq!(stats.total_sessions, 1);
        assert_eq!(stats.total_attempts, 1);
        assert!(stats.total_practice_minutes > 0.0);
    }
}
