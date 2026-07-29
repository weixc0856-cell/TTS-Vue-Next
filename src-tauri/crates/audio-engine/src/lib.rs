use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use thiserror::Error;

/// Errors from audio operations
#[derive(Debug, Error)]
pub enum AudioError {
    #[error("No audio device available: {0}")]
    NoDevice(String),
    #[error("Stream error: {0}")]
    Stream(String),
    #[error("WAV encoding error: {0}")]
    WavEncode(String),
    #[error("Playback error: {0}")]
    Playback(String),
    #[error("Already recording")]
    AlreadyRecording,
    #[error("Not recording")]
    NotRecording,
}

/// A recorded audio clip (16kHz mono WAV)
#[derive(Debug, Clone)]
pub struct RecordedAudio {
    pub data: Vec<u8>,
    pub sample_rate: u32,
    pub duration_ms: u64,
}

/// Audio recording engine
pub struct Recorder {
    recording: Arc<AtomicBool>,
    audio_buffer: Arc<Mutex<Vec<f32>>>,
    sample_rate: u32,
    /// Keep the stream alive while recording (on Windows cpal::Stream is Send)
    #[allow(dead_code)]
    stream: Option<cpal::Stream>,
}

impl Recorder {
    /// Create a new recorder
    pub fn new() -> Self {
        Recorder {
            recording: Arc::new(AtomicBool::new(false)),
            audio_buffer: Arc::new(Mutex::new(Vec::new())),
            sample_rate: 16000,
            stream: None,
        }
    }

    /// List available audio input devices
    pub fn list_devices() -> Result<Vec<String>, AudioError> {
        let host = cpal::default_host();
        let devices = host
            .input_devices()
            .map_err(|e| AudioError::NoDevice(e.to_string()))?;

        let names: Vec<String> = devices
            .filter_map(|d| d.name().ok())
            .collect();

        Ok(names)
    }

    /// Start recording from the default input device
    pub fn start_recording(&mut self) -> Result<(), AudioError> {
        if self.recording.load(Ordering::SeqCst) {
            return Err(AudioError::AlreadyRecording);
        }

        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or_else(|| AudioError::NoDevice("No default input device".to_string()))?;

        let config = device
            .default_input_config()
            .map_err(|e| AudioError::Stream(e.to_string()))?;

        self.sample_rate = config.sample_rate().0;

        let err_fn = |err| log::error!("Audio stream error: {err}");

        let recording = Arc::clone(&self.recording);
        recording.store(true, Ordering::SeqCst);

        let audio_buffer = Arc::clone(&self.audio_buffer);
        {
            let mut buffer = audio_buffer.lock().unwrap();
            buffer.clear();
        }

        let stream = device
            .build_input_stream(
                &config.into(),
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    if recording.load(Ordering::SeqCst) {
                        if let Ok(mut buffer) = audio_buffer.lock() {
                            buffer.extend_from_slice(data);
                        }
                    }
                },
                err_fn,
                None,
            )
            .map_err(|e| AudioError::Stream(e.to_string()))?;

        stream
            .play()
            .map_err(|e| AudioError::Stream(e.to_string()))?;

        self.stream = Some(stream);
        Ok(())
    }

    /// Stop recording and return the audio as WAV bytes
    pub fn stop_recording(&mut self) -> Result<RecordedAudio, AudioError> {
        if !self.recording.load(Ordering::SeqCst) {
            return Err(AudioError::NotRecording);
        }

        self.recording.store(false, Ordering::SeqCst);
        self.stream = None; // Drop the stream

        // Small delay for callback to flush remaining data
        std::thread::sleep(std::time::Duration::from_millis(50));

        let samples = {
            let buffer = self.audio_buffer.lock().unwrap();
            buffer.clone()
        };

        if samples.is_empty() {
            return Ok(RecordedAudio {
                data: Vec::new(),
                sample_rate: self.sample_rate,
                duration_ms: 0,
            });
        }

        // Convert f32 samples to i16 for WAV
        let samples_i16: Vec<i16> = samples
            .iter()
            .map(|&s| {
                if s >= 1.0 {
                    i16::MAX
                } else if s <= -1.0 {
                    i16::MIN
                } else {
                    (s * i16::MAX as f32) as i16
                }
            })
            .collect();

        let duration_ms = (samples.len() as u64 * 1000) / self.sample_rate as u64;

        // Encode as WAV (16-bit mono)
        let mut wav_data = Vec::new();
        {
            let spec = hound::WavSpec {
                channels: 1,
                sample_rate: self.sample_rate,
                bits_per_sample: 16,
                sample_format: hound::SampleFormat::Int,
            };
            let mut writer = hound::WavWriter::new(
                std::io::Cursor::new(&mut wav_data),
                spec,
            )
            .map_err(|e| AudioError::WavEncode(e.to_string()))?;

            for sample in samples_i16 {
                writer
                    .write_sample(sample)
                    .map_err(|e| AudioError::WavEncode(e.to_string()))?;
            }

            writer
                .finalize()
                .map_err(|e| AudioError::WavEncode(e.to_string()))?;
        }

        Ok(RecordedAudio {
            data: wav_data,
            sample_rate: self.sample_rate,
            duration_ms,
        })
    }

    /// Get current audio level (peak RMS 0.0 - 1.0)
    pub fn audio_level(&self) -> f64 {
        let buffer = self.audio_buffer.lock().unwrap();
        if buffer.is_empty() {
            return 0.0;
        }

        let sum_sq: f32 = buffer.iter().map(|&s| s * s).sum();
        let rms = (sum_sq / buffer.len() as f32).sqrt();
        (rms as f64).min(1.0)
    }
}

impl Default for Recorder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_devices_does_not_panic() {
        let result = Recorder::list_devices();
        match result {
            Ok(devices) => {
                println!("Found {} audio device(s)", devices.len());
            }
            Err(e) => {
                println!("No audio devices: {e}");
            }
        }
    }

    #[test]
    fn test_recorder_new_state() {
        let recorder = Recorder::new();
        assert!(!recorder.recording.load(Ordering::SeqCst));
        assert_eq!(recorder.audio_level(), 0.0);
    }

    #[test]
    fn test_stop_without_start_returns_error() {
        let mut recorder = Recorder::new();
        let result = recorder.stop_recording();
        assert!(result.is_err());
        match result {
            Err(AudioError::NotRecording) => {}
            _ => panic!("Expected NotRecording error"),
        }
    }
}
