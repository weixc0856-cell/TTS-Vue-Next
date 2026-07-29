export type PracticeMode = 'shadowing' | 'roleplay' | 'pronunciation' | 'freetalk';
export type Difficulty = 'beginner' | 'intermediate' | 'advanced';
export type ScenarioCategory = 'daily' | 'travel' | 'business' | 'academic' | 'exam';
export type ExerciseSource = 'builtin' | 'user' | 'document';
export type WordStatus = 'correct' | 'wrong' | 'omitted' | 'extra';

export interface Sentence {
  id: string;
  text: string;
  translation?: string;
  orderIndex: number;
}

export interface Exercise {
  id: string;
  title: string;
  mode: PracticeMode;
  difficulty: Difficulty;
  category: ScenarioCategory;
  source: ExerciseSource;
  sentences: Sentence[];
  metadata: Record<string, unknown>;
}

export interface PracticeSession {
  id: string;
  exerciseId: string;
  mode: PracticeMode;
  startedAt: string;
  completedAt?: string;
  overallScore?: number;
}

export interface WordScore {
  word: string;
  status: WordStatus;
  confidence: number;
}

export interface Attempt {
  id: string;
  sessionId: string;
  sentenceId: string;
  audioPath?: string;
  transcript?: string;
  score?: number;
  wordScores: WordScore[];
  completeness?: number;
  durationMs?: number;
}

export interface ScoreResult {
  overall: number;
  accuracy: number;
  completeness: number;
  wordScores: WordScore[];
}

export interface SessionDetail {
  session: PracticeSession;
  attempts: Attempt[];
}

export type RecordingStatus = 'idle' | 'recording' | 'processing' | 'done' | 'error';
