import { defineStore } from 'pinia';
import { invoke } from '@tauri-apps/api/core';
import type { RecordingStatus } from '../../types/practice';

interface RecorderState {
  status: RecordingStatus;
  audioLevel: number;
  audioData: Uint8Array | null;
  durationMs: number;
  error: string | null;
}

export const useRecorderStore = defineStore('practiceRecorder', {
  state: (): RecorderState => ({
    status: 'idle',
    audioLevel: 0,
    audioData: null,
    durationMs: 0,
    error: null,
  }),

  actions: {
    async startRecording(): Promise<void> {
      if (this.status === 'recording') return;

      this.status = 'recording';
      this.audioData = null;
      this.durationMs = 0;
      this.error = null;

      try {
        await invoke('start_recording');
      } catch (e) {
        this.status = 'error';
        this.error = String(e);
      }
    },

    async stopRecording(): Promise<Uint8Array | null> {
      if (this.status !== 'recording') return null;

      this.status = 'processing';
      try {
        const result = await invoke<number[]>('stop_recording');
        this.audioData = new Uint8Array(result);
        this.status = 'done';
        return this.audioData;
      } catch (e) {
        this.status = 'error';
        this.error = String(e);
        return null;
      }
    },

    pollAudioLevel(): void {
      // Called periodically from UI to update level display
      if (this.status === 'recording') {
        invoke<number>('get_audio_level')
          .then((level) => {
            this.audioLevel = level;
          })
          .catch(() => {
            // Silently fail — will retry on next poll
          });
      }
    },

    reset(): void {
      this.status = 'idle';
      this.audioLevel = 0;
      this.audioData = null;
      this.durationMs = 0;
      this.error = null;
    },
  },
});
