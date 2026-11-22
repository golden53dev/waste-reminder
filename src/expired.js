const { invoke } = window.__TAURI__.core;

const btnImport = document.getElementById("btn-import");
const btnClose = document.getElementById("btn-close");
const statusMsg = document.getElementById("status-msg");

btnImport.addEventListener("click", async () => {
  statusMsg.textContent = "Wybieranie pliku...";
  statusMsg.style.color = "#9ca3af";
  try {
    const res = await invoke("import_schedule_dialog");
    if (res.success) {
      statusMsg.style.color = "#10b981";
      statusMsg.textContent = res.message;
      btnImport.disabled = true;
      setTimeout(async () => {
        try {
          await invoke("close_window");
        } catch (e) {
          window.close();
        }
      }, 1500);
    } else {
      statusMsg.style.color = "#ef4444";
      statusMsg.textContent = res.message;
    }
  } catch (err) {
    statusMsg.style.color = "#ef4444";
    statusMsg.textContent = `Błąd: ${err}`;
  }
});

btnClose.addEventListener("click", async () => {
  try {
    await invoke("close_window");
  } catch (e) {
    window.close();
  }
});
