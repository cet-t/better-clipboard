import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { listen } from "@tauri-apps/api/event";

interface ClipboardEntry {
  id: number;
  entry_type: string;
  content_hash: string;
  text_content: string | null;
  file_path: string | null;
  thumbnail_path: string | null;
  file_size: number | null;
  source_app: string | null;
  created_at: string;
  is_pinned: boolean;
  display_order: number;
}

const entryList = document.getElementById("entry-list")!;
const entryCount = document.getElementById("entry-count")!;

async function applyLocale() {
  try {
    const strings = await invoke<Record<string, string>>("get_locale_strings");
    document.title = strings.window_title_overlay || document.title;
    document.querySelectorAll<HTMLElement>("[data-locale]").forEach((el) => {
      const key = el.dataset.locale;
      if (key && strings[key]) {
        el.textContent = strings[key];
      }
    });
  } catch (err) {
    console.error("Failed to load locale:", err);
  }
}

async function loadConfig() {
  try {
    const config = await invoke<{ font_family: string | null }>("get_config");
    if (config.font_family) {
      document.body.style.fontFamily = config.font_family;
    }
  } catch (err) {
    console.error("Failed to load config:", err);
  }
}

async function loadEntries() {
  try {
    await invoke("ensure_clipboard_captured");
    const entries = await invoke<ClipboardEntry[]>("get_clipboard_entries");
    renderEntries(entries);
    entryCount.textContent = String(entries.length);
  } catch (err) {
    console.error("Failed to load entries:", err);
  }
}

function focusOverlay() {
  const overlay = document.getElementById("overlay");
  if (overlay) {
    overlay.focus();
  }
  window.focus();
}

function renderEntries(entries: ClipboardEntry[]) {
  entryList.innerHTML = "";
  const keys = "asdfjkl;";

  entries.forEach((entry, i) => {
    const item = document.createElement("div");
    item.className = "entry-item";
    item.dataset.index = String(i);

    const keyHint = document.createElement("span");
    keyHint.className = "key-hint";
    keyHint.textContent = keys[i] || "";

    const content = document.createElement("span");
    content.className = "entry-content";
    const text = entry.text_content || "";
    content.textContent = text.length > 120 ? text.slice(0, 117) + "..." : text;
    content.title = text;

    item.appendChild(keyHint);
    item.appendChild(content);
    item.addEventListener("click", () => pasteEntry(i));
    entryList.appendChild(item);
  });
}

async function pasteEntry(index: number) {
  try {
    await invoke("paste_entry", { index });
  } catch (err) {
    console.error("Failed to paste:", err);
  }
}

window.addEventListener("keydown", (e) => {
  const keys = "asdfjkl;";
  const idx = keys.indexOf(e.key);
  if (idx >= 0) {
    pasteEntry(idx);
  }
  if (e.key === "Escape") {
    getCurrentWindow().hide();
  }
});

window.addEventListener("DOMContentLoaded", () => {
  applyLocale();
  loadConfig();
  loadEntries().then(() => focusOverlay());
  listen("refresh-entries", () => {
    applyLocale();
    loadConfig();
    loadEntries().then(() => focusOverlay());
  });
});
