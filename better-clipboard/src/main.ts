import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { listen } from "@tauri-apps/api/event";

const PAGE_SIZE = 8;
const MAX_TEXT_LENGTH = 120;
const SELECT_KEYS = "asdfjkl;";

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

type Mode = "normal" | "editSelect" | "deleteSelect";

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

let mode: Mode = "normal";
let editKey = "e";
let deleteKey = "q";
let pageUpKey = "w";
let pageDownKey = "r";
let entries: ClipboardEntry[] = [];
let editingEntryId: number | null = null;
let page = 0;
let localeStrings: Record<string, string> = {};

// ── Locale ──────────────────────────────────────────────

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

// ── Config & Data ───────────────────────────────────────

async function loadConfig() {
  try {
    const config = await invoke<{
      font_family: string | null;
      hotkeys: { edit_key: string; delete_key: string; page_up: string; page_down: string };
    }>("get_config");
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

// ── Rendering ───────────────────────────────────────────

function focusOverlay() {
  document.getElementById("overlay")?.focus();
  window.focus();
}

function totalPages(): number {
  return Math.max(1, Math.ceil(entries.length / PAGE_SIZE));
}

function updatePageInfo() {
  const total = totalPages();
  footerPage.textContent = total > 1
    ? locale("overlay_footer_page", { current: String(page + 1), total: String(total) })
    : "";
}

function renderEntries() {
  entryList.innerHTML = "";
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
    keyHint.textContent = SELECT_KEYS[i] || "";

    const content = document.createElement("span");
    content.className = "entry-content";
    const text = entry.text_content || "";
    content.textContent = text.length > MAX_TEXT_LENGTH ? text.slice(0, MAX_TEXT_LENGTH - 3) + "..." : text;
    content.title = text;

    item.appendChild(keyHint);
    item.appendChild(content);
    item.addEventListener("click", () => handleEntryClick(start + i));
    entryList.appendChild(item);
  });

  updatePageInfo();
}

function handleEntryClick(index: number) {
  if (mode === "editSelect") {
    openEditPanel(index);
  } else if (mode === "deleteSelect") {
    deleteEntry(index);
  } else {
    pasteEntry(index);
  }
}

// ── Pagination ──────────────────────────────────────────

function changePage(delta: -1 | 1) {
  const total = totalPages();
  page = (page + delta + total) % total;
  renderEntries();
}

// ── Mode Management ─────────────────────────────────────

function exitAllSelectModes() {
  if (mode === "editSelect") exitEditSelect();
  else if (mode === "deleteSelect") exitDeleteSelect();
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

// ── Entry Actions ───────────────────────────────────────

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

// ── Keyboard ────────────────────────────────────────────

function selectKeyIndex(key: string): number {
  return SELECT_KEYS.indexOf(key);
}

function handleEditPanelKeys(e: KeyboardEvent) {
  if (e.key === "Escape") {
    e.stopPropagation();
    closeEditPanel();
    loadEntries();
  } else if (e.key === "s" && (e.ctrlKey || e.metaKey)) {
    e.preventDefault();
    saveEdit();
  }
}

function handleSelectModeKeys(e: KeyboardEvent) {
  if (e.key === pageUpKey) { changePage(-1); return; }
  if (e.key === pageDownKey) { changePage(1); return; }

  const idx = selectKeyIndex(e.key);
  if (idx >= 0) {
    e.preventDefault();
    const target = page * PAGE_SIZE + idx;
    if (mode === "editSelect") openEditPanel(target);
    else deleteEntry(target);
  } else if (e.key === "Escape") {
    exitAllSelectModes();
  }
}

function handleNormalKeys(e: KeyboardEvent) {
  if (e.key === pageUpKey) { changePage(-1); return; }
  if (e.key === pageDownKey) { changePage(1); return; }

  const idx = selectKeyIndex(e.key);
  if (idx >= 0) pasteEntry(page * PAGE_SIZE + idx);
  if (e.key === "Escape") getCurrentWindow().hide();
}

window.addEventListener("keydown", (e) => {
  if (!editPanel.classList.contains("hidden")) {
    handleEditPanelKeys(e);
    return;
  }

  if (e.key === editKey) {
    if (mode === "editSelect") exitEditSelect();
    else { exitAllSelectModes(); enterEditSelect(); }
    return;
  }

  if (e.key === deleteKey) {
    if (mode === "deleteSelect") exitDeleteSelect();
    else { exitAllSelectModes(); enterDeleteSelect(); }
    return;
  }

  if (mode === "editSelect" || mode === "deleteSelect") {
    handleSelectModeKeys(e);
    return;
  }

  handleNormalKeys(e);
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
