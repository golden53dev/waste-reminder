const { invoke } = window.__TAURI__.core;

const soundSelect = document.getElementById("sound-select");
const btnPlaySound = document.getElementById("btn-play-sound");
const btnPickSound = document.getElementById("btn-pick-sound");
const volumeInput = document.getElementById("volume");
const volumeVal = document.getElementById("volume-val");
const alertTimeInput = document.getElementById("alert-time");
const fullscreenInput = document.getElementById("fullscreen");
const btnSave = document.getElementById("btn-save");
const btnCancel = document.getElementById("btn-cancel");
const statusMsg = document.getElementById("status-msg");

let currentConfig = null;
let previewAudio = null;

function updateVolumeDisplay() {
  const percent = Math.round(parseFloat(volumeInput.value) * 100);
  volumeVal.textContent = `${percent}%`;
  if (previewAudio) {
    previewAudio.volume = parseFloat(volumeInput.value);
  }
}

volumeInput.addEventListener("input", updateVolumeDisplay);

function stopPreview() {
  if (previewAudio) {
    previewAudio.pause();
    previewAudio = null;
  }
  btnPlaySound.textContent = "▶ Odtwórz";
}

btnPlaySound.addEventListener("click", () => {
  if (previewAudio) {
    stopPreview();
    return;
  }

  const selectedPath = soundSelect.value;
  let src = "assets/alarm.mp3";
  if (selectedPath) {
    if (window.__TAURI__.core.convertFileSrc) {
      src = window.__TAURI__.core.convertFileSrc(selectedPath);
    } else {
      src = selectedPath;
    }
  }

  previewAudio = new Audio(src);
  previewAudio.volume = parseFloat(volumeInput.value);
  btnPlaySound.textContent = "⏹ Zatrzymaj";

  previewAudio.onended = () => {
    stopPreview();
  };
  previewAudio.onerror = (e) => {
    console.error("Błąd odtwarzania:", e);
    stopPreview();
    statusMsg.style.color = "#ef4444";
    statusMsg.textContent = "Nie udało się odtworzyć dźwięku.";
  };

  previewAudio.play().catch((err) => {
    console.error("Audio play error:", err);
    stopPreview();
  });
});

soundSelect.addEventListener("change", () => {
  if (previewAudio) {
    stopPreview();
  }
});

btnPickSound.addEventListener("click", async () => {
  try {
    const pickedPath = await invoke("pick_sound_file");
    if (pickedPath) {
      // Check if already in select
      let existingOption = Array.from(soundSelect.options).find(
        (opt) => opt.value === pickedPath
      );
      if (!existingOption) {
        existingOption = document.createElement("option");
        existingOption.value = pickedPath;
        const filename = pickedPath.split(/[/\\]/).pop();
        existingOption.textContent = `Własny: ${filename}`;
        soundSelect.appendChild(existingOption);
      }
      soundSelect.value = pickedPath;
      if (previewAudio) {
        stopPreview();
      }
    }
  } catch (err) {
    console.error("pick_sound_file error:", err);
  }
});

async function init() {
  try {
    currentConfig = await invoke("get_config");
    const sounds = await invoke("get_available_sounds");

    soundSelect.innerHTML = "";
    sounds.forEach((s) => {
      const opt = document.createElement("option");
      opt.value = s.path;
      opt.textContent = s.name;
      soundSelect.appendChild(opt);
    });

    const activeSound = currentConfig.sound_file || "";
    let found = Array.from(soundSelect.options).some(
      (opt) => opt.value === activeSound
    );
    if (activeSound && !found) {
      const opt = document.createElement("option");
      opt.value = activeSound;
      const filename = activeSound.split(/[/\\]/).pop();
      opt.textContent = `Własny: ${filename}`;
      soundSelect.appendChild(opt);
    }
    soundSelect.value = activeSound;

    volumeInput.value =
      typeof currentConfig.volume === "number" ? currentConfig.volume : 0.8;
    updateVolumeDisplay();

    if (currentConfig.alert_time) {
      alertTimeInput.value = currentConfig.alert_time.slice(0, 5);
    } else {
      alertTimeInput.value = "06:30";
    }

    fullscreenInput.checked = !!currentConfig.fullscreen;
  } catch (err) {
    console.error("Błąd inicjalizacji opcji:", err);
  }
}

btnSave.addEventListener("click", async () => {
  stopPreview();
  try {
    const timeVal = alertTimeInput.value || "06:30";
    const alertTimeFormatted = timeVal.length === 5 ? `${timeVal}:00` : timeVal;
    const selectedSound = soundSelect.value || null;

    const updated = {
      ...currentConfig,
      sound_file: selectedSound,
      volume: parseFloat(volumeInput.value),
      alert_time: alertTimeFormatted,
      fullscreen: fullscreenInput.checked,
    };

    await invoke("save_config", { updated });
    statusMsg.style.color = "#10b981";
    statusMsg.textContent = "Zapisano pomyślnie!";

    setTimeout(async () => {
      try {
        await invoke("close_window");
      } catch (e) {
        window.close();
      }
    }, 400);
  } catch (err) {
    console.error("Błąd zapisu:", err);
    statusMsg.style.color = "#ef4444";
    statusMsg.textContent = `Błąd zapisu: ${err}`;
  }
});

btnCancel.addEventListener("click", async () => {
  stopPreview();
  try {
    await invoke("close_window");
  } catch (e) {
    window.close();
  }
});

init();
