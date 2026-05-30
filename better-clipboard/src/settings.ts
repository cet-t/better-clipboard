import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";

interface Hotkeys {
  overlay: string;
  select_keys: string;
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

const overlayKey = document.getElementById("overlay-key") as HTMLInputElement;
const selectKeys = document.getElementById("select-keys") as HTMLInputElement;
const persistence = document.getElementById("persistence") as HTMLSelectElement;
const dbPath = document.getElementById("db-path") as HTMLInputElement;
const dbField = document.getElementById("db-field") as HTMLElement;
const maxEntries = document.getElementById("max-entries") as HTMLInputElement;
const fontFamily = document.getElementById("font-family") as HTMLInputElement;
const localeSelect = document.getElementById("locale") as HTMLSelectElement;
const clearDisplayBtn = document.getElementById("clear-display") as HTMLButtonElement;
const clearAllBtn = document.getElementById("clear-all") as HTMLButtonElement;
const clearOlderBtn = document.getElementById("clear-older") as HTMLButtonElement;
const clearOlderDays = document.getElementById("clear-older-days") as HTMLInputElement;
const saveBtn = document.getElementById("save") as HTMLButtonElement;
const cancelBtn = document.getElementById("cancel") as HTMLButtonElement;
const status = document.getElementById("status") as HTMLParagraphElement;

async function applyLocale() {
  try {
    strings = await invoke<Record<string, string>>("get_locale_strings");
    document.title = strings.window_title_settings || document.title;
    document.querySelectorAll<HTMLElement>("[data-locale]").forEach((el) => {
      const key = el.dataset.locale;
      if (key && strings[key]) {
        el.textContent = strings[key];
      }
    });
    document.querySelectorAll<HTMLInputElement>("[data-locale-ph]").forEach((el) => {
      const key = el.dataset.localePh;
      if (key && strings[key]) {
        el.placeholder = strings[key];
      }
    });
  } catch (err) {
    console.error("Failed to load locale:", err);
  }
}

async function loadConfig() {
  try {
    const config = await invoke<Config>("get_config");
    overlayKey.value = config.hotkeys.overlay;
    selectKeys.value = config.hotkeys.select_keys;
    persistence.value = config.persistence;
    dbPath.value = config.db.path;
    maxEntries.value = String(config.max_entries);
    fontFamily.value = config.font_family || "";
    localeSelect.value = config.locale || "";
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
      overlay: overlayKey.value,
      select_keys: selectKeys.value,
    },
    persistence: persistence.value,
    db: { path: dbPath.value },
    max_entries: parseInt(maxEntries.value, 10) || 100,
    font_family: fontFamily.value || null,
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

async function clearAll() {
  try {
    await invoke("clear_entries", { mode: "all", days: null });
    status.textContent = strings.status_cleared_all || "Cleared all entries";
  } catch (err) {
    console.error(err);
  }
}

async function clearDisplay() {
  try {
    await invoke("clear_entries", { mode: "display", days: null });
    status.textContent = strings.status_cleared_display || "Cleared display entries";
  } catch (err) {
    console.error(err);
  }
}

async function clearOlder() {
  const days = parseInt(clearOlderDays.value, 10) || 30;
  try {
    await invoke("clear_entries", { mode: "older", days });
    status.textContent = (strings.status_cleared_older || "Cleared entries older than {days} days").replace("{days}", String(days));
  } catch (err) {
    console.error(err);
  }
}

saveBtn.addEventListener("click", saveConfig);
cancelBtn.addEventListener("click", () => getCurrentWindow().hide());
clearAllBtn.addEventListener("click", clearAll);
clearDisplayBtn.addEventListener("click", clearDisplay);
clearOlderBtn.addEventListener("click", clearOlder);

window.addEventListener("DOMContentLoaded", async () => {
  await applyLocale();
  loadConfig();
});
