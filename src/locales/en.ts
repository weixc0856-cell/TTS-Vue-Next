export default {
  common: {
    appName: "TTS Vue Next",
    formats: {
      mp3: "MP3",
      wav: "WAV",
      ogg: "OGG",
      flac: "FLAC",
    },
  },
  nav: {
    textToSpeech: "TTS",
    batchConvert: "Batch Convert",
    practice: "Speaking Practice",
    settings: "Settings",
    versionCurrent: "Version {version}",
    versionUpdateAvailable: "Version {current} → {latest}",
  },
  titleBar: {
    openRepository: "Open GitHub repository",
    openRepositoryFailed: "Failed to open repository. Please try again.",
    toggleTheme: "Toggle theme",
    minimize: "Minimize window",
    toggleMaximize: "Toggle maximize window",
    close: "Close window",
    windowActionFailed: "Window action failed. Please try again.",
  },
  tts: {
    textInput: {
      title: "Text Input",
      placeholder: "Enter text to convert to speech...",
    },
    options: {
      title: "Voice Control Dock",
      language: "Language",
      voice: "Voice",
      rate: "Rate",
      pitch: "Pitch",
      volume: "Volume",
      outputFormat: "Output Format",
      generate: "Generate Speech",
      generating: "Generating...",
      stop: "Stop",
    },
    audioPlayer: {
      title: "Playback Console",
      togglePlayback: "Toggle playback",
      saveGeneratedAudio: "Save generated audio",
      saveFilterName: "Audio",
      defaultFileName: "tts-output.{ext}",
    },
  },
  batch: {
    hero: {
      overline: "Batch Workflow Studio",
      title: "Batch Workflow Studio",
      description:
        "Queue text files, review progress per item, and export audio with controlled concurrency.",
    },
    options: {
      title: "Batch Control Panel",
    },
    actions: {
      startAll: "Start All",
      clear: "Clear",
      concurrency: "Concurrency",
    },
    upload: {
      title: "Drop text files into the queue",
      description:
        "Click to browse, or drop `.txt`, `.md`, `.markdown`, and `.docx` files here.",
      chooseFiles: "Choose Files",
      unsupportedFileTypes: "Unsupported file types: {files}",
      filePickerFilterName: "Text Files",
    },
    list: {
      title: "Queue Progress",
      columns: {
        file: "File",
        status: "Status",
        progress: "Progress",
        actions: "Actions",
      },
      emptyTitle: "No files queued yet",
      emptyDescription:
        "You can drag and drop files here, or use the button below to select files.",
      status: {
        completed: "Completed",
        failed: "Failed",
        processing: "Processing",
        queued: "Queued",
      },
    },
    errors: {
      failedToRemoveTempFile:
        "Failed to remove temporary file {path}: {message}",
    },
  },
  settings: {
    hero: {
      overline: "Preferences",
      title: "Tune output and processing behavior",
      description:
        "Choose where converted audio is saved and how aggressively batch jobs should run.",
    },
    sections: {
      output: "Output",
      processing: "Processing",
      about: "About",
    },
    fields: {
      savePath: "Save Path",
      savePathPlaceholder: "Click to select...",
      defaultFormat: "Default Format",
      displayLanguage: "Display Language",
      autoplay: "Auto-play after conversion",
      maxRetries: "Max Retries",
      fileConcurrency: "File Concurrency",
      chunkConcurrency: "Chunk Concurrency",
    },
    languages: {
      zh: "Simplified Chinese",
      en: "English",
    },
    about: {
      description:
        "A desktop TTS application powered by Microsoft Edge TTS service and built with Vue 3, Vuetify, and Tauri.",
    },
  },
  practice: {
    hub: {
      title: "Practice Center",
      subtitle: "Choose a mode and start practicing",
      shadowing: "Shadowing",
      shadowingDesc: "Listen and repeat to improve pronunciation",
      roleplay: "Role-play",
      roleplayDesc: "Practice real-life conversations",
      pronunciation: "Pronunciation",
      pronunciationDesc: "Coming soon",
      freetalk: "Free Talk",
      freetalkDesc: "Coming soon",
      startShadowing: "Start Shadowing",
      startRoleplay: "Start Role-play",
      browseScenarios: "Browse Scenarios",
    },
    shadowing: {
      title: "Shadowing Practice",
      sentence: "Sentence",
      of: "of",
      playReference: "Play Reference",
      record: "Record",
      stop: "Stop",
      next: "Next Sentence",
      complete: "Complete Practice",
      listen: "Listen to the reference audio, then record yourself.",
      result: "Score",
    },
    roleplay: {
      title: "Role-play Practice",
      partner: "Partner",
      you: "You",
      yourTurn: "Your turn to speak!",
      playPartner: "Play Partner's Line",
      record: "Record Your Response",
      next: "Next Exchange",
      complete: "Complete Practice",
    },
    scoring: {
      overall: "Overall",
      accuracy: "Accuracy",
      completeness: "Completeness",
      correct: "Correct",
      wrong: "Wrong",
      omitted: "Omitted",
      extra: "Extra",
      noScore: "No score yet",
    },
    actions: {
      start: "Start",
      retry: "Retry",
      backToHub: "Back to Hub",
      sendToPractice: "Send to Shadowing",
    },
  },
} as const;
