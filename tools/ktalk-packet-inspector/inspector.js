const STORAGE_KEY = "ktalkPacketInspectorEvents";
const eventsNode = document.getElementById("events");
const summaryNode = document.getElementById("summary");
const filterNode = document.getElementById("filter");

document.getElementById("refresh").addEventListener("click", render);
document.getElementById("clear").addEventListener("click", async () => {
  await chrome.runtime.sendMessage({ type: "ktalk.clear" });
  await render();
});
document.getElementById("export").addEventListener("click", async () => {
  const { [STORAGE_KEY]: events = [] } = await chrome.storage.local.get(STORAGE_KEY);
  const blob = new Blob([JSON.stringify(events, null, 2)], { type: "application/json" });
  const url = URL.createObjectURL(blob);
  await chrome.downloads.download({
    url,
    filename: "ktalk-packet-inspector-log.json",
    saveAs: true
  });
});
filterNode.addEventListener("input", render);

await render();
setInterval(render, 2000);

async function render() {
  const { [STORAGE_KEY]: events = [] } = await chrome.storage.local.get(STORAGE_KEY);
  const filter = filterNode.value.trim().toLowerCase();
  const filtered = events.filter((event) => {
    if (!filter) {
      return true;
    }
    return JSON.stringify(event).toLowerCase().includes(filter);
  });

  summaryNode.textContent = `Событий: ${filtered.length} из ${events.length}`;
  eventsNode.replaceChildren(...filtered.slice().reverse().map(renderEvent));
}

function renderEvent(event) {
  const article = document.createElement("article");
  article.className = "event";

  const title = document.createElement("h2");
  title.textContent = event.method || event.stage || "event";

  const meta = document.createElement("p");
  meta.className = "meta";
  meta.textContent = `${event.timestamp} | tab=${event.tabId ?? "-"} | source=${event.source ?? "-"}`;

  const pre = document.createElement("pre");
  pre.textContent = JSON.stringify(event, null, 2);

  article.append(title, meta, pre);
  return article;
}
