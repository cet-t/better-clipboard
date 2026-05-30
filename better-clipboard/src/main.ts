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
const footerEdit = document.getElementById("footer-edit")!;
const footerDelete = document.getElementById("footer-delete")!;
const footerPage = document.getElementById("footer-page")!;
const editPanel = document.getElementById("edit-panel")!;
const editLabel = document.getElementById("edit-label")!;
const editTextarea = document.getElementById("edit-textarea") as HTMLTextAreaElement;
const editSave = document.getElementById("edit-save")!;
const editCancel = document.getElementById("edit-cancel")!;

const PAGE_SIZE = 8;
type Mode = "normal" | "editSelect" | "deleteSelect";
let mode: Mode = "normal";
let editKey = "e";
let deleteKey = "q";
let pageUpKey = "w";
let pageDownKey = "r";
let entries: ClipboardEntry[] = [];
let editingEntryId: number | null = null;
let page = 0;

let localeStrings: Record<string, string> = {};

function locale(key: string, params?: Record<string, string>): string {
  let s = localeStrings[key] || key;
  if (params) {
    for (const [k, v] of Object.entries(params)) {
      s = s.replace(new RegExp(`\\{${k}\\}`, "g"), v);
    }
  }
  return s;
}

async function applyLocale() {
  try {
    localeStrings = await invoke<Record<string, string>>("get_locale_strings");
    document.title = locale("window_title_overlay");
    document.querySelectorAll<HTMLElement>("[data-locale]").forEach((el) => {
      const key = el.dataset.locale;
      if (key) el.textContent = locale(key);
    });
  } catch (err) {
    console.error("Failed to load locale:", err);
  }
}

async function loadConfig() {
  try {
    const config = await invoke<{ font_family: string | null; hotkeys: { edit_key: string; delete_key: string; page_up: string; page_down: string } }>("get_config");
    if (config.font_family) {
      document.body.style.fontFamily = config.font_family;
    }
    editKey = config.hotkeys.edit_key || "e";
    deleteKey = config.hotkeys.delete_key || "q";
    pageUpKey = config.hotkeys.page_up || "w";
    pageDownKey = config.hotkeys.page_down || "r";
    footerEdit.textContent = locale("overlay_footer_edit", { key: editKey });
    footerDelete.textContent = locale("overlay_footer_delete", { key: deleteKey });
  } catch (err) {
    console.error("Failed to load config:", err);
  }
}

async function loadEntries() {
  try {
    await invoke("ensure_clipboard_captured");
    entries = await invoke<ClipboardEntry[]>("get_clipboard_entries");
    page = 0;
    renderEntries();
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

function updatePageInfo() {
  const totalPages = Math.max(1, Math.ceil(entries.length / PAGE_SIZE));
  footerPage.textContent = totalPages > 1
    ? locale("overlay_footer_page", { current: String(page + 1), total: String(totalPages) })
    : "";
}

function renderEntries() {
  entryList.innerHTML = "";
  const keys = "asdfjkl;";
  const start = page * PAGE_SIZE;
  const visible = entries.slice(start, start + PAGE_SIZE);

  visible.forEach((entry, i) => {
    const item = document.createElement("div");
    item.className = "entry-item";
    if (mode === "editSelect") item.classList.add("edit-mode");
    if (mode === "deleteSelect") item.classList.add("delete-mode");
    item.dataset.index = String(start + i);

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
    item.addEventListener("click", () => {
      if (mode === "editSelect") {
        openEditPanel(start + i);
      } else if (mode === "deleteSelect") {
        deleteEntry(start + i);
      } else {
        pasteEntry(start + i);
      }
    });
    entryList.appendChild(item);
  });

  updatePageInfo();
}

function prevPage() {
  const totalPages = Math.max(1, Math.ceil(entries.length / PAGE_SIZE));
  if (page > 0) {
    page--;
  } else {
    page = totalPages - 1;
  }
  renderEntries();
}

function nextPage() {
  const totalPages = Math.max(1, Math.ceil(entries.length / PAGE_SIZE));
  if (page < totalPages - 1) {
    page++;
  } else {
    page = 0;
  }
  renderEntries();
}

function enterEditSelect() {
  mode = "editSelect";
  footerEdit.textContent = locale("overlay_footer_edit_select", { key: editKey });
  renderEntries();
}

function exitEditSelect() {
  mode = "normal";
  footerEdit.textContent = locale("overlay_footer_edit", { key: editKey });
  renderEntries();
}

function enterDeleteSelect() {
  mode = "deleteSelect";
  footerEdit.textContent = locale("overlay_footer_delete_select", { key: deleteKey });
  renderEntries();
}

function exitDeleteSelect() {
  mode = "normal";
  footerEdit.textContent = locale("overlay_footer_edit", { key: editKey });
  footerDelete.textContent = locale("overlay_footer_delete", { key: deleteKey });
  renderEntries();
}

async function deleteEntry(index: number) {
  const entry = entries[index];
  if (!entry) return;
  try {
    await invoke("delete_entry", { id: entry.id });
    entries = await invoke<ClipboardEntry[]>("get_clipboard_entries");
    renderEntries();
    entryCount.textContent = String(entries.length);
  } catch (err) {
    console.error("Failed to delete entry:", err);
  }
}

function openEditPanel(index: number) {
  const entry = entries[index];
  if (!entry || !entry.text_content) return;

  mode = "normal";
  editingEntryId = entry.id;
  footerEdit.textContent = "Ctrl+S save | Esc cancel";

  editLabel.textContent = "#" + (index + 1);
  editTextarea.value = entry.text_content;
  editPanel.classList.remove("hidden");
  editTextarea.focus();
  editTextarea.select();
}

function closeEditPanel() {
  editingEntryId = null;
  editPanel.classList.add("hidden");
  footerEdit.textContent = locale("overlay_footer_edit", { key: editKey });
  footerDelete.textContent = locale("overlay_footer_delete", { key: deleteKey });
}

async function saveEdit() {
  if (editingEntryId === null) return;
  try {
    await invoke("save_edited_entry", { id: editingEntryId, text: editTextarea.value });
    closeEditPanel();
    await loadEntries();
  } catch (err) {
    console.error("Failed to save edit:", err);
  }
}

async function pasteEntry(index: number) {
  try {
    await invoke("paste_entry", { index });
  } catch (err) {
    console.error("Failed to paste:", err);
  }
}

function entryIndexByKey(key: string): number {
  const keys = "asdfjkl;";
  return keys.indexOf(key);
}

window.addEventListener("keydown", (e) => {
  if (editPanel.classList.contains("hidden") === false) {
    if (e.key === "Escape") {
      e.stopPropagation();
      closeEditPanel();
      loadEntries();
    }
    if (e.key === "s" && (e.ctrlKey || e.metaKey)) {
      e.preventDefault();
      saveEdit();
    }
    return;
  }

  if (e.key === editKey) {
    if (mode === "editSelect") {
      exitEditSelect();
    } else {
      if (mode === "deleteSelect") exitDeleteSelect();
      enterEditSelect();
    }
    return;
  }

  if (e.key === deleteKey) {
    if (mode === "deleteSelect") {
      exitDeleteSelect();
    } else {
      if (mode === "editSelect") exitEditSelect();
      enterDeleteSelect();
    }
    return;
  }

  if (mode === "editSelect" || mode === "deleteSelect") {
    if (e.key === pageUpKey) {
      prevPage();
      return;
    }
    if (e.key === pageDownKey) {
      nextPage();
      return;
    }
    const idx = entryIndexByKey(e.key);
    if (idx >= 0) {
      e.preventDefault();
      if (mode === "editSelect") {
        openEditPanel(page * PAGE_SIZE + idx);
      } else {
        deleteEntry(page * PAGE_SIZE + idx);
      }
    } else if (e.key === "Escape") {
      if (mode === "editSelect") {
        exitEditSelect();
      } else {
        exitDeleteSelect();
      }
    }
    return;
  }

  if (e.key === pageUpKey) {
    prevPage();
    return;
  }
  if (e.key === pageDownKey) {
    nextPage();
    return;
  }

  const idx = entryIndexByKey(e.key);
  if (idx >= 0) {
    pasteEntry(page * PAGE_SIZE + idx);
  }
  if (e.key === "Escape") {
    getCurrentWindow().hide();
  }
});

editSave.addEventListener("click", saveEdit);
editCancel.addEventListener("click", () => {
  closeEditPanel();
  loadEntries();
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
