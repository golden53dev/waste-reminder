window.onerror = (m) =>
  document.body.insertAdjacentHTML(
    "beforeend",
    "<pre style='color:red'>" + m + "</pre>",
  );
const { invoke } = window.__TAURI__.core;
const box = document.getElementById("imgs");
const audio = new Audio("assets/alarm.mp3");
audio.loop = true;

(async () => {
  const keys = await invoke("alarm_fractions");
  keys.forEach((k) => {
    const i = document.createElement("img");
    i.src = `assets/${k}.png`;
    box.appendChild(i);
  });
  audio.play().catch((e) => console.log("audio:", e));
})();

document.getElementById("x").onclick = async () => {
  audio.pause();
  await invoke("ack_alarm");
};
