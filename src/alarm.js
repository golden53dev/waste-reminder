window.onerror = (m) =>
  document.body.insertAdjacentHTML(
    "beforeend",
    "<pre style='color:red'>" + m + "</pre>",
  );

const { invoke } = window.__TAURI__.core;
const imgContainer = document.getElementById("img-container");
const extraFractions = document.getElementById("extra-fractions");

let audio = null;

(async () => {
  try {
    const items = await invoke("alarm_fractions");
    if (items && items.length > 0) {
      const first = items[0];
      const firstKey = typeof first === "string" ? first : first.key;
      const firstLabel = typeof first === "string" ? first : first.label;

      const img = document.createElement("img");
      img.src = `assets/${firstKey}.png`;
      img.alt = firstLabel;
      imgContainer.appendChild(img);

      if (items.length > 1) {
        const others = items
          .slice(1)
          .map((f) => (typeof f === "string" ? f : f.label))
          .join(", ");
        extraFractions.textContent = `Oraz: ${others}`;
      }
    }
  } catch (e) {
    console.error("alarm_fractions error:", e);
  }

  try {
    const cfg = await invoke("get_config");
    let audioSrc = "assets/alarm.mp3";
    if (cfg && cfg.sound_file) {
      if (window.__TAURI__.core.convertFileSrc) {
        audioSrc = window.__TAURI__.core.convertFileSrc(cfg.sound_file);
      } else {
        audioSrc = cfg.sound_file;
      }
    }
    audio = new Audio(audioSrc);
    audio.loop = true;
    if (cfg && typeof cfg.volume === "number") {
      audio.volume = Math.max(0, Math.min(1, cfg.volume));
    }
    audio.play().catch((e) => console.log("audio play error:", e));
  } catch (e) {
    console.log("fallback to default audio:", e);
    audio = new Audio("assets/alarm.mp3");
    audio.loop = true;
    audio.play().catch((err) => console.log("audio fallback error:", err));
  }
})();

document.getElementById("btn-options").onclick = async () => {
  await invoke("open_options");
};

document.getElementById("x").onclick = async () => {
  if (audio) {
    audio.pause();
  }
  await invoke("ack_alarm");
};
