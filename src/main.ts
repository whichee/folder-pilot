import { invoke } from "@tauri-apps/api/core";

const isTauri = typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

interface FolderEntry {
  path: string;
  name: string;
  depth: number;
  is_favorite: boolean;
  exists: boolean;
}

interface AppConfig {
  favorites: string[];
  root_dirs: string[];
  hotkey: string;
  scan_depth: number;
  autostart: boolean;
}

interface ArchiveResult {
  src: string;
  dest: string;
  ok: boolean;
  error?: string | null;
}

let config: AppConfig = {
  favorites: [],
  root_dirs: [],
  hotkey: "Alt+Shift+F",
  scan_depth: 3,
  autostart: true,
};

let folders: FolderEntry[] = [];
let selectedFiles: string[] = [];
let searchTerm = "";
let currentTab: "favorites" | "all" | "settings" = "favorites";

const $ = <T extends HTMLElement>(sel: string) => document.querySelector(sel) as T;

const searchEl = $<HTMLInputElement>("#search");
const favoritesList = $("#favorites-list");
const allList = $("#all-list");
const rootDirsEl = $("#root-dirs");
const archCountEl = $("#arch-count");
const archBar = $("#arch-bar");
const statusBar = $("#status-bar");

async function loadConfig(): Promise<void> {
  if (isTauri) {
    config = await invoke<AppConfig>("get_config");
  } else {
    config = {
      favorites: [
        "D:\\工作\\零售业务部",
        "D:\\工作\\零售业务部\\02-省行需求单",
        "D:\\工作\\零售业务部\\06-投诉",
      ],
      root_dirs: ["D:\\工作\\零售业务部"],
      hotkey: "Alt+Shift+F",
      scan_depth: 3,
      autostart: true,
    };
  }
  $<HTMLInputElement>("#hotkey-input").value = config.hotkey;
  $<HTMLInputElement>("#depth-input").value = String(config.scan_depth);
  $<HTMLInputElement>("#autostart-toggle").checked = config.autostart;
  renderRootDirs();
  await refreshFolders();
}

async function refreshFolders(): Promise<void> {
  if (isTauri) {
    folders = await invoke<FolderEntry[]>("scan_folders");
  } else {
    folders = [
      { path: "D:\\工作\\零售业务部", name: "零售业务部", depth: 0, is_favorite: true, exists: true },
      { path: "D:\\工作\\零售业务部\\01-报表", name: "01-报表", depth: 1, is_favorite: false, exists: true },
      { path: "D:\\工作\\零售业务部\\02-省行需求单", name: "02-省行需求单", depth: 1, is_favorite: true, exists: true },
      { path: "D:\\工作\\零售业务部\\06-投诉", name: "06-投诉", depth: 1, is_favorite: true, exists: true },
      { path: "D:\\工作\\零售业务部\\06-投诉\\台账", name: "台账", depth: 2, is_favorite: false, exists: true },
      { path: "D:\\工作\\零售业务部\\07-需求单", name: "07-需求单", depth: 1, is_favorite: false, exists: true },
      { path: "D:\\工作\\零售业务部\\10-银企智联项目", name: "10-银企智联项目", depth: 1, is_favorite: false, exists: true },
      { path: "D:\\工作\\零售业务部\\14-招投标", name: "14-招投标", depth: 1, is_favorite: false, exists: true },
      { path: "D:\\工作\\零售业务部\\已归档", name: "已归档（旧）", depth: 1, is_favorite: false, exists: false },
    ];
  }
  render();
}

function filteredFolders(): FolderEntry[] {
  const term = searchTerm.trim().toLowerCase();
  if (!term) return folders;
  return folders.filter(
    (f) => f.name.toLowerCase().includes(term) || f.path.toLowerCase().includes(term),
  );
}

function render(): void {
  renderFavorites();
  renderAll();
  renderArchBar();
}

function renderFavorites(): void {
  const favs = folders.filter((f) => f.is_favorite);
  favoritesList.innerHTML = "";
  const empty = $("#favorites-empty");
  empty.style.display = favs.length === 0 ? "block" : "none";
  for (const f of favs) {
    favoritesList.appendChild(folderRow(f));
  }
}

function renderAll(): void {
  const shown = filteredFolders();
  allList.innerHTML = "";
  const empty = $("#all-empty");
  empty.style.display = shown.length === 0 ? "block" : "none";
  for (const f of shown) {
    allList.appendChild(folderRow(f));
  }
}

function folderRow(f: FolderEntry): HTMLElement {
  const row = document.createElement("div");
  row.className = "folder-row" + (f.exists ? "" : " dead") + (f.is_favorite ? " fav" : "");
  row.style.paddingLeft = `${8 + f.depth * 16}px`;
  row.dataset.path = f.path;

  const name = document.createElement("span");
  name.className = "f-name";
  name.textContent = f.name;
  name.title = f.path;
  name.style.fontWeight = f.depth === 0 ? "600" : "400";

  const path = document.createElement("span");
  path.className = "f-path";
  path.textContent = f.path;
  path.title = f.path;

  const actions = document.createElement("span");
  actions.className = "f-actions";

  const btnStar = document.createElement("button");
  btnStar.className = "mini-btn";
  btnStar.textContent = f.is_favorite ? "★" : "☆";
  btnStar.title = f.is_favorite ? "取消收藏" : "收藏";
  btnStar.onclick = (e) => {
    e.stopPropagation();
    toggleFavorite(f.path, !f.is_favorite);
  };

  const btnOpen = document.createElement("button");
  btnOpen.className = "mini-btn open";
  btnOpen.textContent = "打开";
  btnOpen.title = "在资源管理器中打开";
  btnOpen.onclick = (e) => {
    e.stopPropagation();
    if (!f.exists) {
      flash("该目录已失效，无法打开");
      return;
    }
    safeInvoke("open_folder", { path: f.path });
  };

  const btnArch = document.createElement("button");
  btnArch.className = "mini-btn arch";
  btnArch.textContent = "归档到此";
  btnArch.title = "把已选文件移到该文件夹";
  btnArch.disabled = selectedFiles.length === 0 || !f.exists;
  btnArch.onclick = (e) => {
    e.stopPropagation();
    doArchive(f);
  };

  actions.append(btnStar, btnOpen, btnArch);
  row.append(name, path, actions);

  // 双击：打开
  row.addEventListener("dblclick", () => {
    if (f.exists) safeInvoke("open_folder", { path: f.path });
  });

  return row;
}

async function toggleFavorite(path: string, fav: boolean): Promise<void> {
  const list = config.favorites.filter((p) => p !== path);
  if (fav) list.push(path);
  config.favorites = list;
  await saveConfig();
  await refreshFolders();
}

function renderRootDirs(): void {
  rootDirsEl.innerHTML = "";
  for (const dir of config.root_dirs) {
    const row = document.createElement("div");
    row.className = "root-dir";
    const span = document.createElement("span");
    span.textContent = dir;
    const del = document.createElement("button");
    del.className = "mini-btn";
    del.textContent = "✕";
    del.onclick = async () => {
      config.root_dirs = config.root_dirs.filter((d) => d !== dir);
      await saveConfig();
      await refreshFolders();
    };
    row.append(span, del);
    rootDirsEl.appendChild(row);
  }
}

function renderArchBar(): void {
  if (selectedFiles.length === 0) {
    archBar.style.display = "none";
    archCountEl.textContent = "未选择文件";
    return;
  }
  archBar.style.display = "flex";
  archCountEl.textContent = `已选 ${selectedFiles.length} 个文件，点列表里的「归档到此」`;
}

async function saveConfig(): Promise<void> {
  await safeInvoke("save_config", { config });
  $<HTMLInputElement>("#autostart-toggle").checked = config.autostart;
}

function flash(msg: string): void {
  statusBar.textContent = msg;
  statusBar.classList.add("show");
  setTimeout(() => statusBar.classList.remove("show"), 2500);
}

async function safeInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  if (isTauri) {
    return invoke<T>(cmd, args);
  }
  // demo 模式：静默成功，便于纯浏览器预览
  return Promise.resolve({} as T);
}

async function doArchive(f: FolderEntry): Promise<void> {
  if (selectedFiles.length === 0) return;
  const results = await safeInvoke<ArchiveResult[]>("archive_files", {
    files: selectedFiles,
    dest: f.path,
  });
  const ok = results.filter((r) => r.ok).length;
  const fail = results.length - ok;
  flash(`归档完成：成功 ${ok} 个${fail ? `，失败 ${fail} 个` : ""}`);
  selectedFiles = [];
  renderArchBar();
}

// ---- 事件绑定 ----

searchEl.addEventListener("input", () => {
  searchTerm = searchEl.value;
  renderAll();
});
searchEl.addEventListener("keydown", (e) => {
  if (e.key !== "Enter") return;
  const shown = filteredFolders().filter((f) => f.exists);
  if (shown.length > 0) safeInvoke("open_folder", { path: shown[0].path });
});

document.querySelectorAll<HTMLButtonElement>(".tab").forEach((tab) => {
  tab.addEventListener("click", () => {
    switchTab(tab.dataset.tab as typeof currentTab);
    render();
  });
});

$("#btn-settings").addEventListener("click", () => {
  switchTab("settings");
  renderRootDirs();
});

$("#btn-add-root").addEventListener("click", async () => {
  const dirs = await safeInvoke<string[]>("pick_folders");
  if (!dirs.length) return;
  for (const d of dirs) {
    if (!config.root_dirs.includes(d)) config.root_dirs.push(d);
  }
  await saveConfig();
  await refreshFolders();
});

$("#btn-save-hotkey").addEventListener("click", async () => {
  const hotkey = $<HTMLInputElement>("#hotkey-input").value.trim();
  if (!hotkey) return;
  try {
    config.hotkey = hotkey;
    await saveConfig();
    flash(`热键已更新为 ${hotkey}`);
  } catch (err) {
    flash(`热键保存失败：${err}`);
  }
});

$("#btn-save-depth").addEventListener("click", async () => {
  const depth = Number($<HTMLInputElement>("#depth-input").value);
  if (!Number.isFinite(depth) || depth < 1 || depth > 8) {
    flash("深度需在 1~8 之间");
    return;
  }
  config.scan_depth = depth;
  await saveConfig();
  await refreshFolders();
  flash("扫描深度已保存");
});

$("#autostart-toggle").addEventListener("change", async () => {
  config.autostart = $<HTMLInputElement>("#autostart-toggle").checked;
  await saveConfig();
});

$("#btn-pick-files").addEventListener("click", async () => {
  selectedFiles = await safeInvoke<string[]>("pick_files");
  renderArchBar();
  flash(`已选择 ${selectedFiles.length} 个文件`);
});

$("#btn-clear-arch").addEventListener("click", () => {
  selectedFiles = [];
  renderArchBar();
  renderAll();
});

window.addEventListener("DOMContentLoaded", async () => {
  // demo 模式支持 ?tab=all|settings 用于预览/截图
  const urlTab = new URLSearchParams(window.location.search).get("tab");
  if (urlTab === "all" || urlTab === "settings") {
    switchTab(urlTab);
  }
  await loadConfig();
});

function switchTab(tab: typeof currentTab): void {
  currentTab = tab;
  document.querySelectorAll(".tab").forEach((t) => t.classList.remove("active"));
  const btn = document.querySelector<HTMLButtonElement>(`.tab[data-tab="${tab}"]`)!;
  btn.classList.add("active");
  document.querySelectorAll(".view").forEach((v) => v.classList.remove("active"));
  $(`#view-${tab}`).classList.add("active");
}
