const statusNode = document.getElementById("status");
const toggleButton = document.getElementById("toggle");
const clearButton = document.getElementById("clearLog");
const inspectorButton = document.getElementById("openInspector");

const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });

if (!tab?.id || !isKtalkUrl(tab.url)) {
  statusNode.textContent = "Активная вкладка не относится к *.ktalk.ru";
  toggleButton.disabled = true;
} else {
  await refreshStatus();
}

toggleButton.addEventListener("click", async () => {
  if (!tab?.id) {
    return;
  }

  const { attached } = await chrome.runtime.sendMessage({
    type: "ktalk.status",
    tabId: tab.id
  });

  if (attached) {
    await chrome.runtime.sendMessage({ type: "ktalk.detach", tabId: tab.id });
  } else {
    const result = await chrome.runtime.sendMessage({
      type: "ktalk.attach",
      tabId: tab.id,
      url: tab.url
    });
    if (!result.ok) {
      statusNode.textContent = result.error;
    }
  }

  await refreshStatus();
});

clearButton.addEventListener("click", async () => {
  await chrome.runtime.sendMessage({ type: "ktalk.clear" });
  statusNode.textContent = "Журнал очищен.";
});

inspectorButton.addEventListener("click", async () => {
  await chrome.tabs.create({ url: chrome.runtime.getURL("inspector.html") });
});

async function refreshStatus() {
  const { attached } = await chrome.runtime.sendMessage({
    type: "ktalk.status",
    tabId: tab.id
  });
  statusNode.textContent = attached
    ? `Захват активен для ${new URL(tab.url).hostname}`
    : `Захват не активен для ${new URL(tab.url).hostname}`;
  toggleButton.textContent = attached ? "Остановить захват" : "Подключить захват";
}

function isKtalkUrl(url) {
  try {
    const parsed = new URL(url);
    return parsed.hostname === "ktalk.ru" || parsed.hostname.endsWith(".ktalk.ru");
  } catch (_error) {
    return false;
  }
}
