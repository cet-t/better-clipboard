import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Combo } from "./combo";

interface Hotkeys {
  overlay: string;
  select_keys: string;
  edit_key: string;
  page_up: string;
  page_down: string;
}

interface DbConfig {
  path: string;
}

interface Config {
  hotkeys: Hotkeys;
  persistence: string;
  db: DbConfig;
  max_entries: number;
  font_family: string | null;
  locale: string | null;
}

let strings: Record<string, string> = {};

const ALPHABET = "abcdefghijklmnopqrstuvwxyz".split("");
const NUMBERS = "0123456789".split("");

const KEY_OPTIONS = [
  ...ALPHABET,
  ...NUMBERS,
  "Tab", "Space", "Backspace", "Delete", "Insert",
  "Home", "End", "PageUp", "PageDown",
  "ArrowUp", "ArrowDown", "ArrowLeft", "ArrowRight",
  "F1", "F2", "F3", "F4", "F5", "F6",
  "F7", "F8", "F9", "F10", "F11", "F12",
];

const MODIFIERS = ["ctrl", "alt", "shift"];
const MODIFIER_PAIRS = ["ctrl+alt", "ctrl+shift", "alt+shift"];

const MODIFIER_COMBOS = [
  ...MODIFIERS.flatMap((mod) => ALPHABET.map((key) => `${mod}+${key}`)),
  ...MODIFIER_PAIRS.flatMap((mod) => ALPHABET.map((key) => `${mod}+${key}`)),
];

const HOTKEY_OPTIONS = [...KEY_OPTIONS, ...MODIFIER_COMBOS].sort();

const FONT_FALLBACKS = ["monospace", "sans-serif", "serif", "cursive", "fantasy"];

const overlayCombo = new Combo(document.getElementById("overlay-key")!, { placeholder: "検索..." });
const pageUpCombo = new Combo(document.getElementById("page-up-key")!, { placeholder: "検索..." });
const pageDownCombo = new Combo(document.getElementById("page-down-key")!, { placeholder: "検索..." });
const fontCombo = new Combo(document.getElementById("font-family")!, { placeholder: "検索..." });

const selectKeys = document.getElementById("select-keys") as HTMLInputElement;
const persistence = document.getElementById("persistence") as HTMLSelectElement;
const dbPath = document.getElementById("db-path") as HTMLInputElement;
const dbField = document.getElementById("db-field") as HTMLElement;
const maxEntries = document.getElementById("max-entries") as HTMLInputElement;
const localeSelect = document.getElementById("locale") as HTMLSelectElement;
const clearDisplayBtn = document.getElementById("clear-display") as HTMLButtonElement;
const clearAllBtn = document.getElementById("clear-all") as HTMLButtonElement;
const clearOlderBtn = document.getElementById("clear-older") as HTMLButtonElement;
const clearOlderDays = document.getElementById("clear-older-days") as HTMLInputElement;
const saveBtn = document.getElementById("save") as HTMLButtonElement;
const cancelBtn = document.getElementById("cancel") as HTMLButtonElement;
const status = document.getElementById("status") as HTMLParagraphElement;

overlayCombo.setOptions(HOTKEY_OPTIONS);
pageUpCombo.setOptions(KEY_OPTIONS);
pageDownCombo.setOptions(KEY_OPTIONS);

async function loadFonts() {
  try {
    const fonts = await invoke<string[]>("get_system_fonts");
    fontCombo.setOptions(["", ...fonts, ...FONT_FALLBACKS]);
  } catch (err) {
    console.error("Failed to load fonts:", err);
    fontCombo.setOptions(["", ...FONT_FALLBACKS]);
  }
}

async function applyLocale() {
  try {
    strings = await invoke<Record<string, string>>("get_locale_strings");
    document.title = strings.window_title_settings || document.title;
    document.querySelectorAll<HTMLElement>("[data-locale]").forEach((el) => {
      const key = el.dataset.locale;
      if (key && strings[key]) el.textContent = strings[key];
    });
  } catch (err) {
    console.error("Failed to load locale:", err);
  }
}

async function loadConfig() {
  try {
    const config = await invoke<Config>("get_config");
    overlayCombo.setValue(config.hotkeys.overlay);
    selectKeys.value = config.hotkeys.select_keys;
    pageUpCombo.setValue(config.hotkeys.page_up || "w");
    pageDownCombo.setValue(config.hotkeys.page_down || "r");
    persistence.value = config.persistence;
    dbPath.value = config.db.path;
    maxEntries.value = String(config.max_entries);
    fontCombo.setValue(config.font_family || "");
    localeSelect.value = config.locale || "";
    if (config.font_family) document.body.style.fontFamily = config.font_family;
    toggleDbField();
  } catch (err) {
    status.textContent = strings.status_load_failed || "Failed to load settings";
    console.error(err);
  }
}

function toggleDbField() {
  dbField.style.display = persistence.value === "db" ? "block" : "none";
}

persistence.addEventListener("change", toggleDbField);

async function saveConfig() {
  const config: Config = {
    hotkeys: {
      overlay: overlayCombo.getValue(),
      select_keys: selectKeys.value,
      edit_key: "e",
      page_up: pageUpCombo.getValue(),
      page_down: pageDownCombo.getValue(),
    },
    persistence: persistence.value,
    db: { path: dbPath.value },
    max_entries: parseInt(maxEntries.value, 10) || 100,
    font_family: fontCombo.getValue() || null,
    locale: localeSelect.value || null,
  };

  try {
    await invoke("save_config", { config });
    status.textContent = strings.status_saved || "Saved";
    status.style.color = "#30d158";
    applyLocale();
  } catch (err) {
    status.textContent = strings.status_save_failed || "Save failed";
    status.style.color = "#ff453a";
    console.error(err);
  }
}

async function clearEntries(mode: string, days: number | null, message: string) {
  try {
    await invoke("clear_entries", { mode, days });
    status.textContent = message;
  } catch (err) {
    console.error(err);
  }
}

saveBtn.addEventListener("click", saveConfig);
cancelBtn.addEventListener("click", () => getCurrentWindow().hide());
clearAllBtn.addEventListener("click", () =>
  clearEntries("all", null, strings.status_cleared_all || "Cleared all entries")
);
clearDisplayBtn.addEventListener("click", () =>
  clearEntries("display", null, strings.status_cleared_display || "Cleared display entries")
);
clearOlderBtn.addEventListener("click", () => {
  const days = parseInt(clearOlderDays.value, 10) || 30;
  const msg = (strings.status_cleared_older || "Cleared entries older than {days} days")
    .replace("{days}", String(days));
  clearEntries("older", days, msg);
});

window.addEventListener("DOMContentLoaded", async () => {
  await applyLocale();
  await loadFonts();
  loadConfig();
});
