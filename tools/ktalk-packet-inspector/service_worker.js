const STORAGE_KEY = "ktalkPacketInspectorEvents";
const MAX_EVENTS = 5000;
const attachedTabs = new Map();

chrome.runtime.onInstalled.addListener(async () => {
  await chrome.storage.local.set({ [STORAGE_KEY]: [] });
});

chrome.action.onClicked.addListener(async (tab) => {
  if (tab?.id && isKtalkUrl(tab.url)) {
    await toggleTabCapture(tab.id, tab.url);
  }
});

chrome.runtime.onMessage.addListener((message, _sender, sendResponse) => {
  (async () => {
    switch (message?.type) {
      case "ktalk.attach":
        sendResponse(await attachToTab(message.tabId, message.url));
        break;
      case "ktalk.detach":
        sendResponse(await detachFromTab(message.tabId));
        break;
      case "ktalk.status":
        sendResponse({ attached: attachedTabs.has(message.tabId) });
        break;
      case "ktalk.clear":
        await chrome.storage.local.set({ [STORAGE_KEY]: [] });
        sendResponse({ ok: true });
        break;
      case "ktalk.getEvents":
        sendResponse(await chrome.storage.local.get(STORAGE_KEY));
        break;
      case "ktalk.getCookies":
        sendResponse(await getCookiesForUrl(message.url, message.tabId));
        break;
      default:
        sendResponse({ ok: false, error: "unknown message" });
    }
  })();
  return true;
});

chrome.tabs.onRemoved.addListener(async (tabId) => {
  if (attachedTabs.has(tabId)) {
    await detachFromTab(tabId);
  }
});

chrome.debugger.onEvent.addListener(async (source, method, params) => {
  if (!source.tabId || !attachedTabs.has(source.tabId)) {
    return;
  }

  if (
    method === "Network.requestWillBeSent" ||
    method === "Network.responseReceived" ||
    method === "Network.webSocketCreated" ||
    method === "Network.webSocketWillSendHandshakeRequest" ||
    method === "Network.webSocketHandshakeResponseReceived" ||
    method === "Network.webSocketFrameSent" ||
    method === "Network.webSocketFrameReceived" ||
    method === "Network.webSocketClosed"
  ) {
    await appendEvent({
      tabId: source.tabId,
      timestamp: new Date().toISOString(),
      source: "debugger",
      method,
      params
    });
  }
});

chrome.debugger.onDetach.addListener(async (source) => {
  if (source.tabId) {
    attachedTabs.delete(source.tabId);
  }
});

chrome.webRequest.onBeforeRequest.addListener(
  async (details) => {
    await appendEvent({
      tabId: details.tabId,
      timestamp: new Date().toISOString(),
      source: "webRequest",
      stage: "onBeforeRequest",
      details: sanitizeDetails(details)
    });
  },
  { urls: ["*://ktalk.ru/*", "*://*.ktalk.ru/*"] },
  ["requestBody"]
);

chrome.webRequest.onBeforeSendHeaders.addListener(
  async (details) => {
    await appendEvent({
      tabId: details.tabId,
      timestamp: new Date().toISOString(),
      source: "webRequest",
      stage: "onBeforeSendHeaders",
      details: sanitizeDetails(details)
    });
  },
  { urls: ["*://ktalk.ru/*", "*://*.ktalk.ru/*"] },
  ["requestHeaders", "extraHeaders"]
);

chrome.webRequest.onHeadersReceived.addListener(
  async (details) => {
    await appendEvent({
      tabId: details.tabId,
      timestamp: new Date().toISOString(),
      source: "webRequest",
      stage: "onHeadersReceived",
      details: sanitizeDetails(details)
    });
  },
  { urls: ["*://ktalk.ru/*", "*://*.ktalk.ru/*"] },
  ["responseHeaders", "extraHeaders"]
);

async function toggleTabCapture(tabId, url) {
  if (attachedTabs.has(tabId)) {
    return detachFromTab(tabId);
  }
  return attachToTab(tabId, url);
}

async function attachToTab(tabId, url) {
  if (!isKtalkUrl(url)) {
    return { ok: false, error: "Этот инструмент работает только на доменах *.ktalk.ru" };
  }
  if (attachedTabs.has(tabId)) {
    return { ok: true, attached: true };
  }

  try {
    await chrome.debugger.attach({ tabId }, "1.3");
    await chrome.debugger.sendCommand({ tabId }, "Network.enable");
    attachedTabs.set(tabId, { attachedAt: Date.now(), url });
    await appendEvent({
      tabId,
      timestamp: new Date().toISOString(),
      source: "extension",
      stage: "attached",
      url
    });
    return { ok: true, attached: true };
  } catch (error) {
    return { ok: false, error: String(error) };
  }
}

async function detachFromTab(tabId) {
  if (!attachedTabs.has(tabId)) {
    return { ok: true, attached: false };
  }

  try {
    await chrome.debugger.detach({ tabId });
  } catch (_error) {
  } finally {
    attachedTabs.delete(tabId);
  }

  await appendEvent({
    tabId,
    timestamp: new Date().toISOString(),
    source: "extension",
    stage: "detached"
  });
  return { ok: true, attached: false };
}

async function appendEvent(event) {
  const current = await chrome.storage.local.get(STORAGE_KEY);
  const events = current[STORAGE_KEY] ?? [];
  events.push(event);
  if (events.length > MAX_EVENTS) {
    events.splice(0, events.length - MAX_EVENTS);
  }
  await chrome.storage.local.set({ [STORAGE_KEY]: events });
}

function sanitizeDetails(details) {
  return {
    requestId: details.requestId,
    url: details.url,
    method: details.method,
    type: details.type,
    initiator: details.initiator,
    statusCode: details.statusCode,
    requestHeaders: details.requestHeaders,
    responseHeaders: details.responseHeaders,
    requestBody: details.requestBody
  };
}

function cookieMatchesHost(cookie, hostname) {
  const domain = (cookie.domain || "").replace(/^\./, "");
  return hostname === domain || hostname.endsWith(`.${domain}`);
}

function isKtalkUrl(url) {
  try {
    const parsed = new URL(url);
    return parsed.hostname === "ktalk.ru" || parsed.hostname.endsWith(".ktalk.ru");
  } catch (_error) {
    return false;
  }
}

async function getCookiesForUrl(url, tabId) {
  if (!isKtalkUrl(url)) {
    return { ok: false, error: "Cookies можно копировать только для доменов *.ktalk.ru" };
  }

  try {
    const parsed = new URL(url);
    const cookies = await chrome.cookies.getAll({});
    const neededNames = ["ngtoken", "kontur_ngtoken"];

    const selected = neededNames
      .map((name) => cookies.find((cookie) => cookie.name === name && cookieMatchesHost(cookie, parsed.hostname)))
      .filter(Boolean);

    if (selected.length === 0) {
      return { ok: false, error: "Не удалось найти ngtoken/kontur_ngtoken для текущего KTalk-домена." };
    }

    const cookieHeader = selected
      .map((cookie) => `${cookie.name}=${cookie.value}`)
      .join("; ");

    await appendEvent({
      timestamp: new Date().toISOString(),
      source: "extension",
      stage: "cookies_copied",
      domain: parsed.hostname,
      names: selected.map((cookie) => cookie.name)
    });

    const sessionToken = Number.isInteger(tabId) ? await getSessionTokenFromLocalStorage(tabId) : null;

    return {
      ok: true,
      cookieHeader,
      sessionToken
    };
  } catch (error) {
    return { ok: false, error: String(error) };
  }
}


async function getSessionTokenFromLocalStorage(tabId) {
  try {
    const [{ result }] = await chrome.scripting.executeScript({
      target: { tabId },
      func: () => {
        const raw = window.localStorage.getItem("session");
        if (!raw) {
          return null;
        }

        const findToken = (value) => {
          if (!value) {
            return null;
          }
          if (typeof value === "string") {
            return null;
          }
          if (Array.isArray(value)) {
            for (const item of value) {
              const nested = findToken(item);
              if (nested) {
                return nested;
              }
            }
            return null;
          }
          if (typeof value === "object") {
            if (typeof value.token === "string" && value.token) {
              return value.token;
            }
            for (const nestedValue of Object.values(value)) {
              const nested = findToken(nestedValue);
              if (nested) {
                return nested;
              }
            }
          }
          return null;
        };

        try {
          const parsed = JSON.parse(raw);
          return findToken(parsed);
        } catch (_error) {
          return null;
        }
      }
    });

    if (typeof result === "string" && result) {
      await appendEvent({
        timestamp: new Date().toISOString(),
        source: "extension",
        stage: "session_token_extracted",
        tabId
      });
      return result;
    }

    return null;
  } catch (_error) {
    return null;
  }
}
