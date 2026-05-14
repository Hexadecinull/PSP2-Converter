import { invoke } from "@tauri-apps/api/tauri";
import { open, save } from "@tauri-apps/api/dialog";
import { appWindow } from "@tauri-apps/api/window";

let currentFile = null;
let currentPage = "convert";
let converting = false;

const pages = {
  convert: renderConvertPage,
  about: renderAboutPage,
};

function icon(paths, size = 16, color = "currentColor") {
  return `<svg width="${size}" height="${size}" viewBox="0 0 24 24" fill="none" stroke="${color}" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">${paths}</svg>`;
}

const ICONS = {
  convert: icon('<path d="M4 12v-2a6 6 0 0 1 6-6h8"/><polyline points="16 2 20 6 16 10"/><path d="M20 12v2a6 6 0 0 1-6 6H6"/><polyline points="8 22 4 18 8 14"/>'),
  about:   icon('<circle cx="12" cy="12" r="10"/><line x1="12" y1="8" x2="12" y2="12"/><line x1="12" y1="16" x2="12.01" y2="16"/>'),
  folder:  icon('<path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>'),
  file:    icon('<path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/>'),
  close:   icon('<line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/>'),
  check:   icon('<polyline points="20 6 9 17 4 12"/>'),
  alert:   icon('<circle cx="12" cy="12" r="10"/><line x1="12" y1="8" x2="12" y2="12"/><line x1="12" y1="16" x2="12.01" y2="16"/>'),
  upload:  icon('<polyline points="16 16 12 12 8 16"/><line x1="12" y1="12" x2="12" y2="21"/><path d="M20.39 18.39A5 5 0 0 0 18 9h-1.26A8 8 0 1 0 3 16.3"/>'),
  github:  icon('<path d="M9 19c-5 1.5-5-2.5-7-3m14 6v-3.87a3.37 3.37 0 0 0-.94-2.61c3.14-.35 6.44-1.54 6.44-7A5.44 5.44 0 0 0 20 4.77 5.07 5.07 0 0 0 19.91 1S18.73.65 16 2.48a13.38 13.38 0 0 0-7 0C6.27.65 5.09 1 5.09 1A5.07 5.07 0 0 0 5 4.77a5.44 5.44 0 0 0-1.5 3.78c0 5.42 3.3 6.61 6.44 7A3.37 3.37 0 0 0 9 18.13V22"/>'),
};

function mount() {
  document.getElementById("app").innerHTML = `
    <div class="titlebar">
      <div class="titlebar-left">
        <svg class="titlebar-logo" viewBox="0 0 24 24" fill="none" stroke="#7b68ee" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <rect x="2" y="6" width="20" height="12" rx="3"/>
          <circle cx="8" cy="12" r="1.5" fill="#7b68ee" stroke="none"/>
          <circle cx="12" cy="12" r="1.5" fill="#7b68ee" stroke="none"/>
          <circle cx="16" cy="12" r="1.5" fill="#7b68ee" stroke="none"/>
        </svg>
        <span class="titlebar-title">PSP2 Converter</span>
        <span class="titlebar-version">v1.0.0</span>
      </div>
      <div class="titlebar-controls">
        <button class="win-btn min" id="btn-min" title="Minimize"></button>
        <button class="win-btn max" id="btn-max" title="Maximize"></button>
        <button class="win-btn close" id="btn-close" title="Close"></button>
      </div>
    </div>
    <div class="main">
      <nav class="sidebar">
        <div class="sidebar-label">Menu</div>
        <div class="nav-item active" data-page="convert">${ICONS.convert} Convert</div>
        <div class="nav-item" data-page="about">${ICONS.about} About</div>
        <div class="sidebar-spacer"></div>
        <div class="sidebar-notice">
          <strong>⚠ Legal notice</strong>
          Only convert games you own. This tool is for personal backup use only.
        </div>
      </nav>
      <div class="content" id="page-content"></div>
    </div>
  `;

  document.getElementById("btn-close").addEventListener("click", () => appWindow.close());
  document.getElementById("btn-min").addEventListener("click", () => appWindow.minimize());
  document.getElementById("btn-max").addEventListener("click", () => appWindow.toggleMaximize());

  document.querySelectorAll(".nav-item").forEach(el => {
    el.addEventListener("click", () => {
      document.querySelectorAll(".nav-item").forEach(n => n.classList.remove("active"));
      el.classList.add("active");
      currentPage = el.dataset.page;
      renderPage();
    });
  });

  renderPage();
}

function renderPage() {
  const container = document.getElementById("page-content");
  container.innerHTML = "";
  pages[currentPage](container);
}

function renderConvertPage(container) {
  container.innerHTML = `
    <div>
      <div class="section-title">Input File</div>
      <div class="drop-zone ${currentFile ? "has-file" : ""}" id="drop-zone">
        ${currentFile ? `
          <div class="file-info" style="width:100%;text-align:left;">
            <span class="file-badge">${currentFile.ext}</span>
            <span class="file-name">${currentFile.name}</span>
            <span class="file-size">${formatBytes(currentFile.size)}</span>
            <button class="file-clear" id="btn-clear" title="Remove">${ICONS.close}</button>
          </div>
        ` : `
          ${ICONS.upload}
          <span class="drop-zone-title">Drop a PSP game file here</span>
          <span class="drop-zone-sub">.iso  .cso  .zso  .pbp</span>
          <button class="btn btn-ghost" id="btn-browse" style="margin-top:6px;">${ICONS.folder} Browse…</button>
        `}
      </div>
    </div>

    <div>
      <div class="section-title">Metadata <span style="font-weight:400;text-transform:none;letter-spacing:0;color:var(--text-muted);font-size:11px;">— leave blank to auto-detect</span></div>
      <div class="form-grid">
        <div class="form-field">
          <label class="form-label" for="inp-title">Title</label>
          <input class="form-input" id="inp-title" placeholder="Auto-detected" spellcheck="false">
        </div>
        <div class="form-field">
          <label class="form-label" for="inp-titleid">Title ID <span class="form-hint">e.g. ULUS00000</span></label>
          <input class="form-input" id="inp-titleid" placeholder="Auto-detected" spellcheck="false" maxlength="9">
        </div>
        <div class="form-field full">
          <label class="form-label" for="inp-output">Output Folder</label>
          <div class="output-row">
            <input class="form-input" id="inp-output" placeholder="Select output directory…" spellcheck="false" readonly>
            <button class="btn btn-ghost" id="btn-outdir">${ICONS.folder}</button>
          </div>
        </div>
      </div>
    </div>

    <div>
      <div class="section-title">Log</div>
      <div class="log-panel" id="log-panel">
        <span class="log-line dim">Ready.</span>
      </div>
    </div>

    <div id="status-area"></div>

    <div class="convert-bar">
      <button class="btn btn-primary btn-lg" id="btn-convert" disabled>
        ${ICONS.convert} Convert to VPK
      </button>
      <div class="progress-wrap" id="progress-wrap" style="display:none;">
        <div class="progress-fill indeterminate" id="progress-fill"></div>
      </div>
    </div>
  `;

  setupConvertPage();
}

function setupConvertPage() {
  const dropZone = document.getElementById("drop-zone");
  const btnConvert = document.getElementById("btn-convert");
  const btnOutdir = document.getElementById("btn-outdir");

  if (!currentFile) {
    document.getElementById("btn-browse")?.addEventListener("click", pickFile);
  } else {
    document.getElementById("btn-clear")?.addEventListener("click", () => {
      currentFile = null;
      renderPage();
    });
  }

  dropZone.addEventListener("dragover", e => {
    e.preventDefault();
    dropZone.classList.add("drag-over");
  });
  dropZone.addEventListener("dragleave", () => dropZone.classList.remove("drag-over"));
  dropZone.addEventListener("drop", e => {
    e.preventDefault();
    dropZone.classList.remove("drag-over");
    const file = e.dataTransfer.files[0];
    if (file) handleFilePath(file.path, file.name, file.size);
  });
  dropZone.addEventListener("click", e => {
    if (e.target.closest("#btn-clear") || e.target.closest("#btn-browse")) return;
    if (!currentFile) pickFile();
  });

  btnOutdir.addEventListener("click", async () => {
    const dir = await open({ directory: true, multiple: false });
    if (dir) document.getElementById("inp-output").value = dir;
    updateConvertBtn();
  });

  document.getElementById("inp-title").addEventListener("input", updateConvertBtn);
  document.getElementById("inp-titleid").addEventListener("input", updateConvertBtn);
  document.getElementById("inp-output").addEventListener("input", updateConvertBtn);

  btnConvert.addEventListener("click", doConvert);
  updateConvertBtn();
}

function updateConvertBtn() {
  const btnConvert = document.getElementById("btn-convert");
  if (!btnConvert) return;
  const hasFile = !!currentFile;
  const hasOutput = !!document.getElementById("inp-output")?.value?.trim();
  btnConvert.disabled = !hasFile || !hasOutput || converting;
}

async function pickFile() {
  const path = await open({
    multiple: false,
    filters: [{ name: "PSP Game", extensions: ["iso", "cso", "zso", "pbp"] }],
  });
  if (!path) return;
  const parts = path.replace(/\\/g, "/").split("/");
  const name = parts[parts.length - 1];
  handleFilePath(path, name, null);
}

function handleFilePath(path, name, size) {
  const extMatch = name.match(/\.(\w+)$/);
  const ext = extMatch ? extMatch[1].toUpperCase() : "?";
  const allowed = ["ISO", "CSO", "ZSO", "PBP"];
  if (!allowed.includes(ext)) {
    showToast(`Unsupported format: .${ext.toLowerCase()}`, "error");
    return;
  }
  currentFile = { path, name, size, ext };
  renderPage();
}

async function doConvert() {
  if (converting) return;
  converting = true;

  const title = document.getElementById("inp-title").value.trim() || null;
  const titleId = document.getElementById("inp-titleid").value.trim() || null;
  const outputDir = document.getElementById("inp-output").value.trim();

  setProgress(true);
  clearStatus();
  log(`Converting ${currentFile.name}…`, "dim");

  document.getElementById("btn-convert").disabled = true;

  try {
    const result = await invoke("convert_game", {
      opts: {
        input_path: currentFile.path,
        output_dir: outputDir,
        title_override: title,
        title_id_override: titleId,
      },
    });

    log(`Format detected: ${result.format_detected}`, "dim");
    log(`Title: ${result.title}`, "dim");
    log(`Title ID: ${result.title_id}`, "dim");
    log(`Output: ${result.output_path}`, "ok");
    log("Done.", "ok");

    showStatus(
      "success",
      `Converted successfully — <strong>${result.title}</strong> (${result.title_id})`,
      result.output_path
    );
  } catch (err) {
    log(`Error: ${err}`, "err");
    showStatus("error", String(err), null);
  } finally {
    converting = false;
    setProgress(false);
    updateConvertBtn();
  }
}

function log(msg, cls = "") {
  const panel = document.getElementById("log-panel");
  if (!panel) return;
  const el = document.createElement("div");
  el.className = `log-line ${cls}`;
  el.textContent = msg;
  panel.appendChild(el);
  panel.scrollTop = panel.scrollHeight;
}

function setProgress(active) {
  const wrap = document.getElementById("progress-wrap");
  if (wrap) wrap.style.display = active ? "block" : "none";
}

function clearStatus() {
  const area = document.getElementById("status-area");
  if (area) area.innerHTML = "";
}

function showStatus(type, message, detail) {
  const area = document.getElementById("status-area");
  if (!area) return;
  const iconEl = type === "success" ? ICONS.check : ICONS.alert;
  area.innerHTML = `
    <div class="status-block ${type}">
      ${iconEl}
      <div>
        <div>${message}</div>
        ${detail ? `<div class="output-detail">${detail}</div>` : ""}
      </div>
    </div>
  `;
}

function showToast(msg, type) {
  const area = document.getElementById("status-area");
  if (!area) return;
  showStatus(type, msg, null);
}

function formatBytes(bytes) {
  if (!bytes) return "";
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 ** 2) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 ** 3) return `${(bytes / 1024 ** 2).toFixed(1)} MB`;
  return `${(bytes / 1024 ** 3).toFixed(2)} GB`;
}

function renderAboutPage(container) {
  container.innerHTML = `
    <div class="about-content">
      <div class="about-card">
        <h2>PSP2 Converter</h2>
        <p>
          Convert PSP game backups into installable VPK files for the PS Vita.
          Games run through the Vita's native PSP hardware emulation layer — no Adrenaline required,
          though installing Adrenaline may improve compatibility for certain titles.
        </p>
        <p style="margin-top:10px;">
          Requires a PS Vita running <strong>HENkaku</strong> or <strong>Ensō</strong> custom firmware.
          Install the generated VPK using VitaShell or a similar file manager.
        </p>
      </div>

      <div class="about-card">
        <div class="section-title" style="margin-bottom:14px;">Supported Formats</div>
        <table class="format-table">
          <thead>
            <tr><th>Ext</th><th>Description</th><th>Notes</th></tr>
          </thead>
          <tbody>
            <tr><td>.iso</td><td>Raw UMD disc image</td><td>Direct conversion, no decompression needed</td></tr>
            <tr><td>.cso</td><td>Compressed ISO (zlib)</td><td>Decompressed to ISO before packaging</td></tr>
            <tr><td>.zso</td><td>Compressed ISO (LZ4)</td><td>Decompressed to ISO before packaging</td></tr>
            <tr><td>.pbp</td><td>PSP executable package</td><td>Metadata and assets extracted automatically</td></tr>
          </tbody>
        </table>
      </div>

      <div class="about-card">
        <div class="section-title" style="margin-bottom:10px;">How It Works</div>
        <p>
          Input files are normalized to a raw ISO, then wrapped in an EBOOT.PBP with the correct PSN structure.
          A Vita-format <code>param.sfo</code> and livearea assets are generated alongside it, then everything
          is zipped into a <code>.vpk</code> ready to install.
        </p>
        <p style="margin-top:10px;">
          Metadata (title, Title ID, version) is read directly from the disc's internal <code>PARAM.SFO</code>
          or <code>UMD_DATA.BIN</code> when available. You can override either field before converting.
        </p>
      </div>

      <div class="about-card">
        <div style="display:flex;align-items:center;justify-content:space-between;flex-wrap:wrap;gap:10px;">
          <div>
            <p style="font-size:13px;color:var(--text-dim);">Licensed under <strong style="color:var(--text);">Apache-2.0</strong></p>
            <p style="font-size:12px;color:var(--text-muted);margin-top:3px;">Copyright © 2024 Hexadecinull</p>
          </div>
          <a href="https://github.com/Hexadecinull/PSP2-Converter" target="_blank" class="btn btn-ghost">
            ${ICONS.github} GitHub
          </a>
        </div>
      </div>
    </div>
  `;
}

mount();
