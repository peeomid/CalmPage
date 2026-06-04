<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { open } from "@tauri-apps/plugin-dialog";
  import { onMount } from "svelte";

  type FileEntry = {
    rootId: string;
    rootName: string;
    path: string;
    title: string;
    modified: number;
    size: number;
  };

  type RootEntry = {
    id: string;
    path: string;
    name: string;
  };

  type VaultSnapshot = {
    roots: RootEntry[];
    files: FileEntry[];
  };

  type RenderedNote = {
    path: string;
    title: string;
    html: string;
    modified: number;
  };

  type OpenedMarkdown = {
    snapshot: VaultSnapshot;
    note: RenderedNote;
    rootId: string;
    path: string;
  };

  type TreeFolder = {
    name: string;
    path: string;
    folders: Map<string, TreeFolder>;
    files: FileEntry[];
  };

  type TreeRow =
    | { type: "folder"; path: string; name: string; depth: number }
    | { type: "file"; file: FileEntry; depth: number };

  type ReaderPresetId = "editorial" | "notebook" | "technical" | "large" | "custom";
  type ColorPresetId = "paper" | "graphite" | "polar" | "sepia" | "midnight";

  type ReaderPreset = {
    id: ReaderPresetId;
    name: string;
    description: string;
    bodyFont: string;
    headingFont: string;
    fontSize: number;
    lineHeight: number;
    measure: number;
    weight: number;
    h1Scale: number;
    h2Scale: number;
    h3Scale: number;
    paragraphSpacing: number;
    codeScale: number;
  };

  type TocItem = {
    id: string;
    text: string;
    level: number;
  };

  type ColorPreset = {
    id: ColorPresetId;
    name: string;
    description: string;
    values: Record<string, string>;
  };

  const readerPresets: ReaderPreset[] = [
    {
      id: "editorial",
      name: "Editorial",
      description: "Warm book-like reading with strong headings.",
      bodyFont: 'Newsreader, "Iowan Old Style", Georgia, serif',
      headingFont: 'Newsreader, "Iowan Old Style", Georgia, serif',
      fontSize: 18,
      lineHeight: 1.68,
      measure: 66,
      weight: 430,
      h1Scale: 2.35,
      h2Scale: 1.65,
      h3Scale: 1.28,
      paragraphSpacing: 1.08,
      codeScale: 0.84,
    },
    {
      id: "notebook",
      name: "Notebook",
      description: "Calmer, slightly denser notes for long daily reading.",
      bodyFont: '"Source Serif 4", "Iowan Old Style", Georgia, serif',
      headingFont: '"Source Serif 4", "Iowan Old Style", Georgia, serif',
      fontSize: 17,
      lineHeight: 1.62,
      measure: 72,
      weight: 420,
      h1Scale: 2.05,
      h2Scale: 1.48,
      h3Scale: 1.22,
      paragraphSpacing: 0.98,
      codeScale: 0.84,
    },
    {
      id: "technical",
      name: "Technical",
      description: "Sharper layout for docs, code, and tables.",
      bodyFont: '"Avenir Next", Avenir, ui-sans-serif, system-ui, sans-serif',
      headingFont: '"Avenir Next", Avenir, ui-sans-serif, system-ui, sans-serif',
      fontSize: 16,
      lineHeight: 1.58,
      measure: 78,
      weight: 450,
      h1Scale: 1.85,
      h2Scale: 1.38,
      h3Scale: 1.16,
      paragraphSpacing: 0.9,
      codeScale: 0.86,
    },
    {
      id: "large",
      name: "Large",
      description: "Bigger type for relaxed reading.",
      bodyFont: 'Newsreader, "Iowan Old Style", Georgia, serif',
      headingFont: 'Newsreader, "Iowan Old Style", Georgia, serif',
      fontSize: 20,
      lineHeight: 1.72,
      measure: 62,
      weight: 430,
      h1Scale: 2,
      h2Scale: 1.42,
      h3Scale: 1.18,
      paragraphSpacing: 1.15,
      codeScale: 0.82,
    },
  ];

  const colorPresets: ColorPreset[] = [
    {
      id: "paper",
      name: "Paper",
      description: "Warm editorial paper.",
      values: {
        canvas: "#e9e0d0",
        panel: "#f5edde",
        panelStrong: "#fff8eb",
        reader: "#fffaf1",
        text: "#1f1a14",
        muted: "#6f6557",
        faint: "#a0927d",
        line: "rgba(48, 38, 25, 0.15)",
        accent: "#8f431e",
        accentStrong: "#b94d19",
        accentSoft: "rgba(143, 67, 30, 0.13)",
        highlight: "rgba(214, 134, 44, 0.24)",
        find: "rgba(238, 177, 61, 0.48)",
        findActive: "rgba(230, 103, 42, 0.54)",
        codeBg: "#efe3cf",
        link: "#a04619",
        inlineCodeText: "#7b3519",
        inlineCodeBg: "#f0dfc5",
        codeBlockBg: "#efe2cb",
        codeBorder: "rgba(126, 83, 33, 0.2)",
        blockquoteBg: "rgba(143, 67, 30, 0.1)",
        blockquoteBorder: "#a95a2a",
        tableBorder: "rgba(48, 38, 25, 0.16)",
        shadow: "0 24px 80px rgba(64, 48, 24, 0.2)",
      },
    },
    {
      id: "graphite",
      name: "Graphite",
      description: "Warm dark, lower eye strain.",
      values: {
        canvas: "#10100f",
        panel: "#181715",
        panelStrong: "#211f1b",
        reader: "#171614",
        text: "#f0e5d2",
        muted: "#b5a890",
        faint: "#796f60",
        line: "rgba(240, 229, 210, 0.13)",
        accent: "#e0a15f",
        accentStrong: "#f0b46e",
        accentSoft: "rgba(224, 161, 95, 0.14)",
        highlight: "rgba(224, 161, 95, 0.22)",
        find: "rgba(224, 161, 95, 0.36)",
        findActive: "rgba(238, 118, 75, 0.48)",
        codeBg: "#27241e",
        link: "#f0b46e",
        inlineCodeText: "#f4c183",
        inlineCodeBg: "#30291f",
        codeBlockBg: "#242119",
        codeBorder: "rgba(224, 161, 95, 0.18)",
        blockquoteBg: "rgba(224, 161, 95, 0.1)",
        blockquoteBorder: "#d99552",
        tableBorder: "rgba(240, 229, 210, 0.14)",
        shadow: "0 24px 80px rgba(0, 0, 0, 0.48)",
      },
    },
    {
      id: "polar",
      name: "Polar",
      description: "Clean neutral light mode.",
      values: {
        canvas: "#e8edf0",
        panel: "#f5f7f8",
        panelStrong: "#ffffff",
        reader: "#fbfcfc",
        text: "#172027",
        muted: "#64717b",
        faint: "#9aa6ad",
        line: "rgba(23, 32, 39, 0.13)",
        accent: "#245f73",
        accentStrong: "#0d7890",
        accentSoft: "rgba(36, 95, 115, 0.12)",
        highlight: "rgba(74, 144, 162, 0.22)",
        find: "rgba(97, 173, 190, 0.36)",
        findActive: "rgba(30, 127, 153, 0.42)",
        codeBg: "#e8eef1",
        link: "#0d7890",
        inlineCodeText: "#0f6072",
        inlineCodeBg: "#ddebf0",
        codeBlockBg: "#e4ecef",
        codeBorder: "rgba(36, 95, 115, 0.16)",
        blockquoteBg: "rgba(36, 95, 115, 0.09)",
        blockquoteBorder: "#2d7890",
        tableBorder: "rgba(23, 32, 39, 0.13)",
        shadow: "0 24px 80px rgba(24, 44, 58, 0.16)",
      },
    },
    {
      id: "sepia",
      name: "Sepia",
      description: "Softer old-book color.",
      values: {
        canvas: "#d8c7a8",
        panel: "#ecdfc5",
        panelStrong: "#f8edd6",
        reader: "#f6ead2",
        text: "#2a2118",
        muted: "#78664e",
        faint: "#a8906b",
        line: "rgba(65, 45, 24, 0.16)",
        accent: "#795028",
        accentStrong: "#9d6129",
        accentSoft: "rgba(121, 80, 40, 0.15)",
        highlight: "rgba(188, 126, 45, 0.25)",
        find: "rgba(205, 146, 50, 0.42)",
        findActive: "rgba(160, 79, 34, 0.42)",
        codeBg: "#ead9b9",
        link: "#8b5620",
        inlineCodeText: "#75451d",
        inlineCodeBg: "#ead5ad",
        codeBlockBg: "#e8d4ae",
        codeBorder: "rgba(121, 80, 40, 0.18)",
        blockquoteBg: "rgba(121, 80, 40, 0.11)",
        blockquoteBorder: "#8b5a28",
        tableBorder: "rgba(65, 45, 24, 0.16)",
        shadow: "0 24px 80px rgba(71, 48, 22, 0.19)",
      },
    },
    {
      id: "midnight",
      name: "Midnight",
      description: "Cool dark, sharper contrast.",
      values: {
        canvas: "#0c1117",
        panel: "#111923",
        panelStrong: "#172130",
        reader: "#0f1720",
        text: "#e7edf3",
        muted: "#a6b3be",
        faint: "#65717c",
        line: "rgba(231, 237, 243, 0.13)",
        accent: "#7fb7d7",
        accentStrong: "#9bd1ee",
        accentSoft: "rgba(127, 183, 215, 0.14)",
        highlight: "rgba(127, 183, 215, 0.23)",
        find: "rgba(127, 183, 215, 0.35)",
        findActive: "rgba(229, 154, 94, 0.48)",
        codeBg: "#182332",
        link: "#9bd1ee",
        inlineCodeText: "#a8d8f1",
        inlineCodeBg: "#17283a",
        codeBlockBg: "#141f2d",
        codeBorder: "rgba(127, 183, 215, 0.18)",
        blockquoteBg: "rgba(127, 183, 215, 0.1)",
        blockquoteBorder: "#7fb7d7",
        tableBorder: "rgba(231, 237, 243, 0.14)",
        shadow: "0 24px 80px rgba(0, 0, 0, 0.55)",
      },
    },
  ];

  let roots = $state<RootEntry[]>([]);
  let files = $state<FileEntry[]>([]);
  let selectedPath = $state<string | null>(null);
  let selectedRootId = $state<string | null>(null);
  let currentNote = $state<RenderedNote | null>(null);
  let isOpening = $state(false);
  let isRendering = $state(false);
  let error = $state<string | null>(null);
  let paletteOpen = $state(false);
  let paletteQuery = $state("");
  let paletteResults = $state<FileEntry[]>([]);
  let paletteActiveIndex = $state(0);
  let theme = $state<"light" | "dark">("light");
  let readerPresetId = $state<ReaderPresetId>("editorial");
  let colorPresetId = $state<ColorPresetId>("paper");
  let readerFontSize = $state(18);
  let readerLineHeight = $state(1.68);
  let readerMeasure = $state(66);
  let readerH1Scale = $state(2.35);
  let readerH2Scale = $state(1.65);
  let readerH3Scale = $state(1.28);
  let readerParagraphSpacing = $state(1.08);
  let readerCodeScale = $state(0.84);
  let settingsOpen = $state(false);
  let findOpen = $state(false);
  let findQuery = $state("");
  let findMatches = $state<HTMLElement[]>([]);
  let findActiveIndex = $state(0);
  let tocQuery = $state("");
  let explorerQuery = $state("");
  let listHeight = $state(0);
  let scrollTop = $state(0);
  let activeHeadingId = $state("");
  let collapsedFolders = $state<Set<string>>(new Set());
  let noteRequestId = 0;
  let paletteRequestId = 0;
  let paletteSearchTimeout = 0;
  let headingObserver: IntersectionObserver | null = null;

  const rowHeight = 56;
  let explorerFilteredFiles = $derived(filterExplorerFiles(files, explorerQuery));
  let explorerCollapsedFolders = $derived(explorerQuery.trim() ? new Set<string>() : collapsedFolders);
  let treeRows = $derived(buildTreeRows(explorerFilteredFiles, explorerCollapsedFolders));
  let maxStartIndex = $derived(Math.max(0, treeRows.length - Math.ceil(listHeight / rowHeight) - 16));
  let startIndex = $derived(Math.min(maxStartIndex, Math.max(0, Math.floor(scrollTop / rowHeight) - 8)));
  let endIndex = $derived(Math.min(treeRows.length, startIndex + Math.ceil(listHeight / rowHeight) + 16));
  let visibleRows = $derived(treeRows.slice(startIndex, endIndex));
  let topSpacer = $derived(startIndex * rowHeight);
  let bottomSpacer = $derived(Math.max(0, (treeRows.length - endIndex) * rowHeight));
  let rootLabel = $derived(roots.length === 0 ? "No folder" : roots.length === 1 ? roots[0].name : `${roots.length} folders`);
  let readerPreset = $derived(readerPresets.find((preset) => preset.id === readerPresetId) ?? readerPresets[0]);
  let readerPresetLabel = $derived(readerPresetId === "custom" ? "Custom" : readerPreset.name);
  let colorPreset = $derived(colorPresets.find((preset) => preset.id === colorPresetId) ?? colorPresets[0]);
  let tocItems = $derived(currentNote ? extractTocItems(currentNote.html) : []);
  let filteredTocItems = $derived(
    tocQuery.trim()
      ? tocItems.filter((item) => item.text.toLowerCase().includes(tocQuery.trim().toLowerCase()))
      : tocItems,
  );
  let readerStyle = $derived(
    [
      `--reader-body-font: ${readerPreset.bodyFont}`,
      `--reader-heading-font: ${readerPreset.headingFont}`,
      `--reader-font-size: ${readerFontSize}px`,
      `--reader-line-height: ${readerLineHeight}`,
      `--reader-measure: ${readerMeasure}ch`,
      `--reader-weight: ${readerPreset.weight}`,
      `--reader-h1-scale: ${readerH1Scale}`,
      `--reader-h2-scale: ${readerH2Scale}`,
      `--reader-h3-scale: ${readerH3Scale}`,
      `--reader-paragraph-spacing: ${readerParagraphSpacing}em`,
      `--reader-code-scale: ${readerCodeScale}em`,
    ].join("; "),
  );

  function applyReaderPreset(preset: ReaderPreset) {
    readerPresetId = preset.id;
    readerFontSize = preset.fontSize;
    readerLineHeight = preset.lineHeight;
    readerMeasure = preset.measure;
    readerH1Scale = preset.h1Scale;
    readerH2Scale = preset.h2Scale;
    readerH3Scale = preset.h3Scale;
    readerParagraphSpacing = preset.paragraphSpacing;
    readerCodeScale = preset.codeScale;
  }

  function markReaderCustom() {
    readerPresetId = "custom";
  }

  function safeJson<T>(key: string, fallback: T): T {
    try {
      return JSON.parse(localStorage.getItem(key) ?? "null") ?? fallback;
    } catch {
      localStorage.removeItem(key);
      return fallback;
    }
  }

  function makeFolder(name: string, path: string): TreeFolder {
    return { name, path, folders: new Map(), files: [] };
  }

  function buildTreeRows(sourceFiles: FileEntry[], collapsed: Set<string>): TreeRow[] {
    const rootFolder = makeFolder("", "");
    for (const file of sourceFiles) {
      const parts = [file.rootName, ...file.path.split("/")];
      let current = rootFolder;
      for (let i = 0; i < parts.length - 1; i += 1) {
        const name = parts[i];
        const folderPath = parts.slice(0, i + 1).join("/");
        let folder = current.folders.get(name);
        if (!folder) {
          folder = makeFolder(name, folderPath);
          current.folders.set(name, folder);
        }
        current = folder;
      }
      current.files.push(file);
    }

    const rows: TreeRow[] = [];
    const pushFolder = (folder: TreeFolder, depth: number) => {
      const folders = [...folder.folders.values()].sort((a, b) => a.name.localeCompare(b.name));
      const folderFiles = [...folder.files].sort((a, b) => a.title.localeCompare(b.title));
      for (const child of folders) {
        rows.push({ type: "folder", path: child.path, name: child.name, depth });
        if (!collapsed.has(child.path)) {
          pushFolder(child, depth + 1);
        }
      }
      for (const file of folderFiles) {
        rows.push({ type: "file", file, depth });
      }
    };
    pushFolder(rootFolder, 0);
    return rows;
  }

  function filterExplorerFiles(sourceFiles: FileEntry[], query: string) {
    const needle = query.trim().toLowerCase();
    if (!needle) return sourceFiles;
    return sourceFiles.filter((file) =>
      `${file.rootName} ${file.title} ${file.path}`.toLowerCase().includes(needle),
    );
  }

  function toggleFolder(path: string) {
    const next = new Set(collapsedFolders);
    if (next.has(path)) {
      next.delete(path);
    } else {
      next.add(path);
    }
    collapsedFolders = next;
  }

  function applySnapshot(snapshot: VaultSnapshot) {
    roots = snapshot.roots;
    files = snapshot.files;
    if (
      selectedPath &&
      selectedRootId &&
      !snapshot.files.some((file) => file.rootId === selectedRootId && file.path === selectedPath)
    ) {
      selectedPath = null;
      selectedRootId = null;
      currentNote = null;
    }
  }

  function collapseAllFolders(sourceFiles = files) {
    const folders = new Set<string>();
    for (const file of sourceFiles) {
      const parts = [file.rootName, ...file.path.split("/")];
      for (let i = 0; i < parts.length - 1; i += 1) {
        folders.add(parts.slice(0, i + 1).join("/"));
      }
    }
    collapsedFolders = folders;
  }

  function saveRoots(nextRoots = roots) {
    localStorage.setItem("minimal-reader:folders", JSON.stringify(nextRoots.map((root) => root.path)));
  }

  function saveReaderSettings() {
    localStorage.setItem(
      "minimal-reader:reader-settings",
      JSON.stringify({
        fontSize: readerFontSize,
        lineHeight: readerLineHeight,
        measure: readerMeasure,
        h1Scale: readerH1Scale,
        h2Scale: readerH2Scale,
        h3Scale: readerH3Scale,
        paragraphSpacing: readerParagraphSpacing,
        codeScale: readerCodeScale,
      }),
    );
  }

  function loadReaderSettings() {
    const saved = safeJson<{
      fontSize?: number;
      lineHeight?: number;
      measure?: number;
      h1Scale?: number;
      h2Scale?: number;
      h3Scale?: number;
      paragraphSpacing?: number;
      codeScale?: number;
    } | null>("minimal-reader:reader-settings", null);
    if (!saved) {
      applyReaderPreset(readerPreset);
      return;
    }
    readerFontSize = saved.fontSize ?? readerPreset.fontSize;
    readerLineHeight = saved.lineHeight ?? readerPreset.lineHeight;
    readerMeasure = saved.measure ?? readerPreset.measure;
    readerH1Scale = saved.h1Scale ?? readerPreset.h1Scale;
    readerH2Scale = saved.h2Scale ?? readerPreset.h2Scale;
    readerH3Scale = saved.h3Scale ?? readerPreset.h3Scale;
    readerParagraphSpacing = saved.paragraphSpacing ?? readerPreset.paragraphSpacing;
    readerCodeScale = saved.codeScale ?? readerPreset.codeScale;
  }

  async function chooseFolder() {
    const selected = await open({
      directory: true,
      multiple: true,
      title: "Open Markdown Folder",
    });
    const paths = Array.isArray(selected) ? selected : typeof selected === "string" ? [selected] : [];
    if (paths.length === 0) return;

    isOpening = true;
    error = null;
    try {
      const snapshot = await invoke<VaultSnapshot>("open_vaults", { paths });
      applySnapshot(snapshot);
      collapseAllFolders(snapshot.files);
      saveRoots(snapshot.roots);
    } catch (err) {
      error = err instanceof Error ? err.message : String(err);
    } finally {
      isOpening = false;
    }
  }

  async function addFolder() {
    const selected = await open({
      directory: true,
      multiple: false,
      title: "Add Markdown Folder",
    });
    if (typeof selected !== "string") return;

    isOpening = true;
    error = null;
    try {
      const snapshot = await invoke<VaultSnapshot>("add_vault", { path: selected });
      applySnapshot(snapshot);
      collapseAllFolders(snapshot.files);
      saveRoots(snapshot.roots);
    } catch (err) {
      error = err instanceof Error ? err.message : String(err);
    } finally {
      isOpening = false;
    }
  }

  async function openNote(file: FileEntry) {
    const requestId = ++noteRequestId;
    selectedPath = file.path;
    selectedRootId = file.rootId;
    isRendering = true;
    error = null;
    paletteOpen = false;
    findOpen = false;
    findQuery = "";
    try {
      const note = await invoke<RenderedNote>("render_note", { rootId: file.rootId, path: file.path });
      if (requestId === noteRequestId) {
        currentNote = note;
      }
    } catch (err) {
      if (requestId === noteRequestId) {
        error = err instanceof Error ? err.message : String(err);
      }
    } finally {
      if (requestId === noteRequestId) {
        isRendering = false;
      }
    }
  }

  async function openMarkdownFilePath(path: string) {
    const requestId = ++noteRequestId;
    isRendering = true;
    error = null;
    try {
      const opened = await invoke<OpenedMarkdown>("open_markdown_file", { path });
      if (requestId !== noteRequestId) return;
      applySnapshot(opened.snapshot);
      collapseAllFolders(opened.snapshot.files);
      saveRoots(opened.snapshot.roots);
      selectedRootId = opened.rootId;
      selectedPath = opened.path;
      currentNote = opened.note;
    } catch (err) {
      if (requestId === noteRequestId) {
        error = err instanceof Error ? err.message : String(err);
      }
    } finally {
      if (requestId === noteRequestId) {
        isRendering = false;
      }
    }
  }

  async function refreshPaletteResults(query = paletteQuery) {
    const requestId = ++paletteRequestId;
    const results = await invoke<FileEntry[]>("search_files", {
      query,
      limit: 30,
    });
    if (requestId === paletteRequestId) {
      paletteResults = results;
      paletteActiveIndex = 0;
    }
  }

  function togglePalette() {
    paletteOpen = !paletteOpen;
    if (paletteOpen) {
      paletteQuery = "";
      void refreshPaletteResults();
      setTimeout(() => {
        document.querySelector<HTMLInputElement>("#palette-input")?.focus();
      }, 0);
    }
  }

  function formatSize(size: number) {
    if (size < 1024) return `${size} B`;
    if (size < 1024 * 1024) return `${Math.round(size / 1024)} KB`;
    return `${(size / 1024 / 1024).toFixed(1)} MB`;
  }

  function handlePaletteKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      event.preventDefault();
      paletteOpen = false;
    } else if (event.key === "ArrowDown") {
      event.preventDefault();
      paletteActiveIndex = Math.min(paletteResults.length - 1, paletteActiveIndex + 1);
    } else if (event.key === "ArrowUp") {
      event.preventDefault();
      paletteActiveIndex = Math.max(0, paletteActiveIndex - 1);
    } else if (event.key === "Enter") {
      event.preventDefault();
      const file = paletteResults[paletteActiveIndex];
      if (file) void openNote(file);
    }
  }

  function handlePaletteDialogKeydown(event: KeyboardEvent) {
    event.stopPropagation();
    if (event.key === "Escape") {
      event.preventDefault();
      paletteOpen = false;
    }
  }

  function extractTocItems(html: string): TocItem[] {
    if (typeof DOMParser === "undefined") return [];
    const document = new DOMParser().parseFromString(html, "text/html");
    const used = new Map<string, number>();
    return [...document.querySelectorAll("h1, h2, h3")]
      .slice(0, 30)
      .map((heading, index) => {
        const level = Number(heading.tagName.slice(1));
        const text = heading.textContent?.trim() || `Section ${index + 1}`;
        const existingId = heading.getAttribute("id");
        const slug = text
          .toLowerCase()
          .replace(/[^a-z0-9]+/g, "-")
          .replace(/^-|-$/g, "");
        const base = existingId || `reader-heading-${slug || index}`;
        const count = used.get(base) ?? 0;
        used.set(base, count + 1);
        return {
          id: count === 0 ? base : `${base}-${count + 1}`,
          text,
          level,
        };
      });
  }

  function applyHeadingIds() {
    const reader = document.querySelector<HTMLElement>("[data-reader]");
    if (!reader) return;
    const used = new Map<string, number>();
    reader.querySelectorAll("h1, h2, h3").forEach((heading, index) => {
      const text = heading.textContent?.trim() || `Section ${index + 1}`;
      const slug = text
          .toLowerCase()
          .replace(/[^a-z0-9]+/g, "-")
          .replace(/^-|-$/g, "");
      const base = heading.id || `reader-heading-${slug || index}`;
      const count = used.get(base) ?? 0;
      used.set(base, count + 1);
      heading.id = count === 0 ? base : `${base}-${count + 1}`;
    });
  }

  function observeHeadings() {
    headingObserver?.disconnect();
    headingObserver = null;
    const headings = [...document.querySelectorAll<HTMLElement>("[data-reader] h1, [data-reader] h2, [data-reader] h3")];
    if (headings.length === 0) return;
    activeHeadingId = headings[0].id;
    headingObserver = new IntersectionObserver(
      (entries) => {
        const visible = entries
          .filter((entry) => entry.isIntersecting)
          .sort((a, b) => a.boundingClientRect.top - b.boundingClientRect.top)[0];
        if (visible?.target.id) activeHeadingId = visible.target.id;
      },
      {
        root: document.querySelector(".reader-scroll"),
        rootMargin: "-12% 0px -70% 0px",
        threshold: [0, 1],
      },
    );
    headings.forEach((heading) => headingObserver?.observe(heading));
  }

  function jumpToHeading(id: string) {
    scrollReaderTargetIntoView(document.getElementById(id), 28);
  }

  function openFind() {
    if (!currentNote) return;
    findOpen = true;
    setTimeout(() => document.querySelector<HTMLInputElement>("#find-input")?.focus(), 0);
  }

  function focusExplorerSearch() {
    setTimeout(() => document.querySelector<HTMLInputElement>("#explorer-search")?.focus(), 0);
  }

  function resetExplorerScroll() {
    scrollTop = 0;
    document.querySelector<HTMLElement>(".file-list")?.scrollTo({ top: 0 });
  }

  function focusTocSearch() {
    if (!currentNote || tocItems.length < 3) return;
    setTimeout(() => document.querySelector<HTMLInputElement>("#toc-search")?.focus(), 0);
  }

  function isTypingTarget(target: EventTarget | null) {
    if (!(target instanceof HTMLElement)) return false;
    return ["INPUT", "TEXTAREA", "SELECT"].includes(target.tagName) || target.isContentEditable;
  }

  function scrollReaderTargetIntoView(target: HTMLElement | null, offset = 84) {
    const scrollContainer = document.querySelector<HTMLElement>(".reader-scroll");
    if (!target || !scrollContainer) return;
    const containerRect = scrollContainer.getBoundingClientRect();
    const targetRect = target.getBoundingClientRect();
    const nextTop = scrollContainer.scrollTop + targetRect.top - containerRect.top - offset;
    scrollContainer.scrollTo({ top: Math.max(0, nextTop), behavior: "smooth" });
  }

  function clearFindHighlights() {
    document.querySelectorAll<HTMLElement>("[data-find-match]").forEach((mark) => {
      const parent = mark.parentNode;
      if (!parent) return;
      parent.replaceChild(document.createTextNode(mark.textContent ?? ""), mark);
      parent.normalize();
    });
    findMatches = [];
    findActiveIndex = 0;
  }

  function markTextNode(textNode: Text, query: string) {
    const text = textNode.nodeValue ?? "";
    const lowerText = text.toLowerCase();
    const lowerQuery = query.toLowerCase();
    if (!textNode.parentNode || !lowerText.includes(lowerQuery)) return;
    const fragment = document.createDocumentFragment();
    let cursor = 0;
    while (cursor < text.length) {
      const index = lowerText.indexOf(lowerQuery, cursor);
      if (index === -1) {
        fragment.appendChild(document.createTextNode(text.slice(cursor)));
        break;
      }
      if (index > cursor) {
        fragment.appendChild(document.createTextNode(text.slice(cursor, index)));
      }
      const mark = document.createElement("mark");
      mark.dataset.findMatch = "true";
      mark.textContent = text.slice(index, index + query.length);
      fragment.appendChild(mark);
      cursor = index + query.length;
    }
    textNode.parentNode.replaceChild(fragment, textNode);
  }

  function refreshFindMatches() {
    clearFindHighlights();
    const query = findQuery.trim();
    const reader = document.querySelector<HTMLElement>("[data-reader]");
    if (!reader || query.length < 2) return;
    const walker = document.createTreeWalker(reader, NodeFilter.SHOW_TEXT, {
      acceptNode(node) {
        const parent = node.parentElement;
        if (!parent || ["SCRIPT", "STYLE", "CODE", "PRE", "MARK"].includes(parent.tagName)) {
          return NodeFilter.FILTER_REJECT;
        }
        return (node.nodeValue ?? "").toLowerCase().includes(query.toLowerCase())
          ? NodeFilter.FILTER_ACCEPT
          : NodeFilter.FILTER_SKIP;
      },
    });
    const nodes: Text[] = [];
    while (walker.nextNode()) nodes.push(walker.currentNode as Text);
    nodes.forEach((node) => markTextNode(node, query));
    findMatches = [...document.querySelectorAll<HTMLElement>("[data-find-match]")];
    findActiveIndex = 0;
    scrollToFindMatch(0);
  }

  function scrollToFindMatch(index: number) {
    if (findMatches.length === 0) return;
    const nextIndex = (index + findMatches.length) % findMatches.length;
    findActiveIndex = nextIndex;
    findMatches.forEach((match) => match.classList.remove("active-find-match"));
    const match = findMatches[nextIndex];
    match.classList.add("active-find-match");
    scrollReaderTargetIntoView(match, 120);
  }

  function handleFindKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      findOpen = false;
      findQuery = "";
      clearFindHighlights();
    } else if (event.key === "Enter") {
      event.preventDefault();
      scrollToFindMatch(findActiveIndex + (event.shiftKey ? -1 : 1));
    }
  }

  onMount(() => {
    const savedTheme = localStorage.getItem("minimal-reader:theme");
    theme = savedTheme === "dark" || savedTheme === "light"
      ? savedTheme
      : window.matchMedia("(prefers-color-scheme: dark)").matches
        ? "dark"
        : "light";
    const savedPreset = localStorage.getItem("minimal-reader:reader-preset") as ReaderPresetId | null;
    if (savedPreset === "custom" || (savedPreset && readerPresets.some((preset) => preset.id === savedPreset))) {
      readerPresetId = savedPreset;
    }
    loadReaderSettings();
    const savedColorPreset = localStorage.getItem("minimal-reader:color-preset") as ColorPresetId | null;
    if (savedColorPreset && colorPresets.some((preset) => preset.id === savedColorPreset)) {
      colorPresetId = savedColorPreset;
    } else {
      colorPresetId = theme === "dark" ? "graphite" : "paper";
    }
    document.documentElement.dataset.theme = theme;

    const savedFolders = safeJson<string[]>("minimal-reader:folders", []);
    if (savedFolders.length > 0) {
      isOpening = true;
      invoke<VaultSnapshot>("open_vaults", { paths: savedFolders })
        .then((snapshot) => {
          applySnapshot(snapshot);
          collapseAllFolders(snapshot.files);
          saveRoots(snapshot.roots);
        })
        .catch(() => localStorage.removeItem("minimal-reader:folders"))
        .finally(() => {
          isOpening = false;
        });
    }

    const keydown = (event: KeyboardEvent) => {
      const isCmd = event.metaKey || event.ctrlKey;
      const isTyping = isTypingTarget(event.target);
      if (isCmd && (event.key.toLowerCase() === "k" || event.key.toLowerCase() === "p")) {
        if (isTyping) return;
        event.preventDefault();
        togglePalette();
      }
      if (isCmd && event.altKey && event.key.toLowerCase() === "f") {
        if (isTyping) return;
        event.preventDefault();
        focusExplorerSearch();
      }
      if (isCmd && !event.altKey && event.key.toLowerCase() === "f") {
        event.preventDefault();
        openFind();
      }
      if (isCmd && event.shiftKey && event.key.toLowerCase() === "o") {
        if (isTyping) return;
        event.preventDefault();
        focusTocSearch();
      }
      if (isCmd && event.key.toLowerCase() === "g" && findOpen && findMatches.length > 0) {
        event.preventDefault();
        scrollToFindMatch(findActiveIndex + (event.shiftKey ? -1 : 1));
      }
      if (!isCmd && event.key === "/" && !isTypingTarget(event.target)) {
        event.preventDefault();
        focusTocSearch();
      }
      if (event.key === "Escape") {
        paletteOpen = false;
        findOpen = false;
        findQuery = "";
        clearFindHighlights();
      }
    };
    window.addEventListener("keydown", keydown);

    const unlistenPromise = listen<VaultSnapshot>("vault-updated", (event) => {
      applySnapshot(event.payload);
      if (paletteOpen) void refreshPaletteResults();
    });
    const unlistenOpenedFilesPromise = listen<string[]>("opened-markdown-files", (event) => {
      const [path] = event.payload;
      if (path) void openMarkdownFilePath(path);
    });

    return () => {
      window.removeEventListener("keydown", keydown);
      headingObserver?.disconnect();
      void unlistenPromise.then((unlisten) => unlisten());
      void unlistenOpenedFilesPromise.then((unlisten) => unlisten());
    };
  });

  $effect(() => {
    document.documentElement.dataset.theme = theme;
    if (typeof localStorage !== "undefined") {
      localStorage.setItem("minimal-reader:theme", theme);
    }
  });

  $effect(() => {
    const values = colorPreset.values;
    for (const [key, value] of Object.entries(values)) {
      const cssName = key.replace(/[A-Z]/g, (letter) => `-${letter.toLowerCase()}`);
      document.documentElement.style.setProperty(`--${cssName}`, value);
    }
    if (typeof localStorage !== "undefined") {
      localStorage.setItem("minimal-reader:color-preset", colorPresetId);
    }
  });

  $effect(() => {
    if (typeof localStorage !== "undefined") {
      localStorage.setItem("minimal-reader:reader-preset", readerPresetId);
      saveReaderSettings();
    }
  });

  $effect(() => {
    if (!currentNote) return;
    setTimeout(() => {
      document.querySelector<HTMLElement>(".reader-scroll")?.scrollTo({ top: 0 });
      clearFindHighlights();
      applyHeadingIds();
      observeHeadings();
    }, 0);
  });

  $effect(() => {
    if (!findOpen) {
      clearFindHighlights();
      return;
    }
    const query = findQuery;
    setTimeout(() => {
      if (query === findQuery) refreshFindMatches();
    }, 40);
  });

  $effect(() => {
    if (!paletteOpen) return;
    const query = paletteQuery;
    window.clearTimeout(paletteSearchTimeout);
    paletteSearchTimeout = window.setTimeout(() => {
      void refreshPaletteResults(query);
    }, 60);
    return () => window.clearTimeout(paletteSearchTimeout);
  });

</script>

<svelte:head>
  <meta
    name="description"
    content="A minimal, fast Markdown reader with premium typography."
  />
</svelte:head>

<main class="app-shell">
  <aside class="sidebar">
    <div class="brand">
      <div>
        <p class="eyebrow">Minimal Reader</p>
        <h1>{rootLabel}</h1>
      </div>
      <button class="icon-button" aria-label="Toggle theme" onclick={() => (theme = theme === "light" ? "dark" : "light")}>
        {theme === "light" ? "Dark" : "Light"}
      </button>
    </div>

    <button class="primary-button" onclick={chooseFolder} disabled={isOpening}>
      {isOpening ? "Opening..." : "Open Folder(s)"}
    </button>

    <button class="ghost-button sidebar-action" onclick={addFolder} disabled={isOpening}>
      Add Folder
    </button>

    <button class="palette-button" onclick={togglePalette}>
      <span>Quick open</span>
      <kbd>⌘K</kbd>
    </button>

    <label class="explorer-search">
      <span>Explorer filter</span>
      <input
        id="explorer-search"
        bind:value={explorerQuery}
        oninput={resetExplorerScroll}
        placeholder="Filter tree..."
        autocomplete="off"
      />
      <kbd>⌥⌘F</kbd>
    </label>

    <div class="status-line">
      <span>{explorerQuery.trim() ? `${explorerFilteredFiles.length}/${files.length}` : files.length} markdown files</span>
      <span>{roots.length > 0 ? "Watching" : "Idle"}</span>
    </div>

    {#if error}
      <p class="error">{error}</p>
    {/if}

    <nav
      class="file-list"
      aria-label="Markdown files"
      bind:clientHeight={listHeight}
      onscroll={(event) => (scrollTop = event.currentTarget.scrollTop)}
    >
      {#if files.length === 0}
        <div class="empty-sidebar">
          Open a folder with Markdown files. The first scan reads file names only.
        </div>
      {:else if explorerFilteredFiles.length === 0}
        <div class="empty-sidebar">
          No files match "{explorerQuery}".
        </div>
      {:else}
        <div style={`height: ${topSpacer}px`} aria-hidden="true"></div>
        {#each visibleRows as row (row.type === "folder" ? `folder:${row.path}` : `file:${row.file.rootId}:${row.file.path}`)}
          {#if row.type === "folder"}
            <button
              class="file-row folder-row"
              onclick={() => !explorerQuery.trim() && toggleFolder(row.path)}
              disabled={Boolean(explorerQuery.trim())}
              title={row.path}
              style={`padding-left: ${10 + row.depth * 14}px`}
            >
              <span class="file-title">
                <span class="chevron">{explorerCollapsedFolders.has(row.path) ? "›" : "⌄"}</span>
                {row.name}
              </span>
              <span class="file-path">{row.path}</span>
            </button>
          {:else}
            <button
              class:selected={row.file.rootId === selectedRootId && row.file.path === selectedPath}
              class="file-row"
              onclick={() => openNote(row.file)}
              title={row.file.path}
              style={`padding-left: ${10 + row.depth * 14}px`}
            >
              <span class="file-title">{row.file.title}</span>
              <span class="file-path">{row.file.path}</span>
            </button>
          {/if}
        {/each}
        <div style={`height: ${bottomSpacer}px`} aria-hidden="true"></div>
      {/if}
    </nav>
  </aside>

  <section class="reader-frame">
    <header class="reader-topbar">
      <div>
        <p class="eyebrow">{currentNote ? "Reading" : "Ready"}</p>
        <h2>{currentNote?.title ?? "Open a Markdown note"}</h2>
      </div>
      <div class="topbar-actions">
        {#if currentNote}
          <span>{formatSize(files.find((file) => file.path === currentNote?.path)?.size ?? 0)}</span>
        {/if}
        <button class="ghost-button" onclick={() => (settingsOpen = !settingsOpen)}>
          Type: {readerPresetLabel}
        </button>
        <button class="ghost-button" onclick={openFind} disabled={!currentNote}>Find <kbd>⌘F</kbd></button>
        <button class="ghost-button" onclick={togglePalette}>Command Palette</button>
      </div>
    </header>

    {#if findOpen}
      <section class="find-bar" aria-label="Find in document">
        <input
          id="find-input"
          bind:value={findQuery}
          onkeydown={handleFindKeydown}
          placeholder="Find in this note..."
          autocomplete="off"
        />
        <span>{findQuery.trim().length < 2 ? "Type 2+ characters" : `${findMatches.length === 0 ? 0 : findActiveIndex + 1}/${findMatches.length}`}</span>
        <button onclick={() => scrollToFindMatch(findActiveIndex - 1)} disabled={findMatches.length === 0}>Prev</button>
        <button onclick={() => scrollToFindMatch(findActiveIndex + 1)} disabled={findMatches.length === 0}>Next</button>
        <button onclick={() => { findOpen = false; findQuery = ""; clearFindHighlights(); }}>Close</button>
      </section>
    {/if}

    {#if settingsOpen}
      <section class="type-panel" aria-label="Reader typography settings">
        <div>
          <p class="eyebrow">Reader Settings</p>
          <h3>Type and color testing</h3>
        </div>
        <div class="settings-groups">
          <div>
            <p class="settings-label">Typography</p>
            <div class="preset-grid">
              {#each readerPresets as preset (preset.id)}
                <button class:active={preset.id === readerPresetId} onclick={() => applyReaderPreset(preset)}>
                  <strong>{preset.name}</strong>
                  <span>{preset.description}</span>
                  <small>{preset.fontSize}px / {preset.lineHeight} / {preset.measure}ch</small>
                </button>
              {/each}
            </div>
            <div class="settings-sliders">
              <label>
                <span>Body size <b>{readerFontSize}px</b></span>
                <input type="range" min="15" max="22" step="0.5" bind:value={readerFontSize} oninput={markReaderCustom} />
              </label>
              <label>
                <span>Line height <b>{readerLineHeight}</b></span>
                <input type="range" min="1.35" max="1.8" step="0.01" bind:value={readerLineHeight} oninput={markReaderCustom} />
              </label>
              <label>
                <span>Width <b>{readerMeasure}ch</b></span>
                <input type="range" min="54" max="84" step="1" bind:value={readerMeasure} oninput={markReaderCustom} />
              </label>
              <label>
                <span>H1 scale <b>{readerH1Scale}x</b></span>
                <input type="range" min="1.5" max="2.8" step="0.05" bind:value={readerH1Scale} oninput={markReaderCustom} />
              </label>
              <label>
                <span>H2 scale <b>{readerH2Scale}x</b></span>
                <input type="range" min="1.2" max="2.1" step="0.05" bind:value={readerH2Scale} oninput={markReaderCustom} />
              </label>
              <label>
                <span>H3 scale <b>{readerH3Scale}x</b></span>
                <input type="range" min="1.05" max="1.6" step="0.05" bind:value={readerH3Scale} oninput={markReaderCustom} />
              </label>
              <label>
                <span>Paragraph gap <b>{readerParagraphSpacing}em</b></span>
                <input type="range" min="0.7" max="1.5" step="0.02" bind:value={readerParagraphSpacing} oninput={markReaderCustom} />
              </label>
              <label>
                <span>Code size <b>{readerCodeScale}em</b></span>
                <input type="range" min="0.72" max="1" step="0.01" bind:value={readerCodeScale} oninput={markReaderCustom} />
              </label>
            </div>
          </div>
          <div>
            <p class="settings-label">Color</p>
            <div class="preset-grid color-grid">
              {#each colorPresets as preset (preset.id)}
                <button
                  class:active={preset.id === colorPresetId}
                  onclick={() => {
                    colorPresetId = preset.id;
                    theme = preset.id === "graphite" || preset.id === "midnight" ? "dark" : "light";
                  }}
                >
                  <strong>{preset.name}</strong>
                  <span>{preset.description}</span>
                  <small>{preset.values.reader} / {preset.values.accent}</small>
                </button>
              {/each}
            </div>
          </div>
        </div>
      </section>
    {/if}

    <div class="reader-workspace">
      <article class="reader-scroll">
        {#if isRendering}
          <div class="loading-card">
            <span class="pulse"></span>
            Rendering note...
          </div>
        {:else if currentNote}
          <div class="reader" data-reader style={readerStyle}>
            {@html currentNote.html}
          </div>
        {:else}
          <div class="hero-empty">
            <p class="eyebrow">Static Markdown Rendering</p>
            <h2>Premium reading first. Editing later.</h2>
            <p>
              Open a folder, then use the sidebar or <kbd>⌘K</kbd> to jump by file name.
              The app watches for new Markdown files while you read.
            </p>
            <button class="primary-button" onclick={chooseFolder}>Open Folder</button>
          </div>
        {/if}
      </article>

      {#if currentNote && tocItems.length >= 3}
        <aside class="toc-rail" aria-label="Table of contents">
          <p class="eyebrow">On This Page</p>
          <input
            id="toc-search"
            class="toc-search"
            bind:value={tocQuery}
            placeholder="Search headings..."
            autocomplete="off"
          />
          <nav>
            {#each filteredTocItems as item (item.id)}
              <button
                class:active={item.id === activeHeadingId}
                class={`toc-level-${item.level}`}
                onclick={() => jumpToHeading(item.id)}
              >
                {item.text}
              </button>
            {:else}
              <p class="toc-empty">No heading match</p>
            {/each}
          </nav>
        </aside>
      {/if}
    </div>
  </section>

  {#if paletteOpen}
    <div
      class="palette-backdrop"
      role="button"
      tabindex="0"
      aria-label="Close command palette"
      onkeydown={(event) => event.key === "Enter" && (paletteOpen = false)}
      onclick={() => (paletteOpen = false)}
    >
      <div
        class="palette"
        role="dialog"
        tabindex="-1"
        aria-modal="true"
        aria-label="Command palette"
        onkeydown={handlePaletteDialogKeydown}
        onclick={(event) => event.stopPropagation()}
      >
        <input
          id="palette-input"
          bind:value={paletteQuery}
          onkeydown={handlePaletteKeydown}
          placeholder="Search Markdown files by name..."
          autocomplete="off"
        />
        <div class="palette-meta">
          {paletteQuery ? `${paletteResults.length} matches for "${paletteQuery}"` : "Recent markdown files"}
        </div>
        <div class="palette-results">
          {#each paletteResults as file, index (file.path)}
            <button class:active={index === paletteActiveIndex} onclick={() => openNote(file)}>
              <span>{file.title}</span>
              <small>{file.rootName} / {file.path}</small>
            </button>
          {:else}
            <p>No matching files</p>
          {/each}
        </div>
      </div>
    </div>
  {/if}
</main>

<style>
  :global(*) {
    box-sizing: border-box;
  }

  :global(html),
  :global(body) {
    margin: 0;
    width: 100%;
    height: 100%;
    min-height: 100%;
    overflow: hidden;
    font-family: "Avenir Next", Avenir, ui-sans-serif, system-ui, sans-serif;
    color: var(--text);
    background: var(--canvas);
    text-rendering: optimizeLegibility;
    -webkit-font-smoothing: antialiased;
  }

  :global(html[data-theme="light"]) {
    --canvas: #e9e0d0;
    --panel: #f5edde;
    --panel-strong: #fff8eb;
    --reader: #fffaf1;
    --text: #1f1a14;
    --muted: #6f6557;
    --faint: #a0927d;
    --line: rgba(48, 38, 25, 0.15);
    --accent: #8f431e;
    --accent-strong: #b94d19;
    --accent-soft: rgba(143, 67, 30, 0.13);
    --highlight: rgba(214, 134, 44, 0.24);
    --code-bg: #efe3cf;
    --shadow: 0 24px 80px rgba(64, 48, 24, 0.2);
  }

  :global(html[data-theme="dark"]) {
    --canvas: #10100f;
    --panel: #181715;
    --panel-strong: #211f1b;
    --reader: #171614;
    --text: #f0e5d2;
    --muted: #b5a890;
    --faint: #796f60;
    --line: rgba(240, 229, 210, 0.13);
    --accent: #e0a15f;
    --accent-strong: #f0b46e;
    --accent-soft: rgba(224, 161, 95, 0.14);
    --highlight: rgba(224, 161, 95, 0.22);
    --code-bg: #27241e;
    --shadow: 0 24px 80px rgba(0, 0, 0, 0.48);
  }

  .app-shell {
    display: grid;
    grid-template-columns: 320px minmax(0, 1fr);
    height: 100vh;
    background:
      radial-gradient(circle at top left, var(--accent-soft), transparent 34rem),
      var(--canvas);
  }

  .sidebar {
    min-height: 0;
    display: flex;
    flex-direction: column;
    gap: 14px;
    padding: 18px;
    border-right: 1px solid var(--line);
    background: color-mix(in srgb, var(--panel) 92%, transparent);
  }

  .brand,
  .reader-topbar {
    display: flex;
    justify-content: space-between;
    gap: 16px;
    align-items: flex-start;
  }

  .eyebrow {
    margin: 0 0 4px;
    color: var(--muted);
    text-transform: uppercase;
    letter-spacing: 0.12em;
    font-size: 11px;
    font-weight: 700;
  }

  h1,
  h2 {
    margin: 0;
    color: var(--text);
  }

  h1 {
    font-size: 20px;
    letter-spacing: -0.035em;
  }

  .primary-button,
  .palette-button,
  .ghost-button,
  .icon-button {
    border: 1px solid var(--line);
    border-radius: 14px;
    color: var(--text);
    background: var(--panel-strong);
    cursor: pointer;
    font: inherit;
  }

  .primary-button {
    padding: 12px 14px;
    font-weight: 750;
    background: var(--text);
    color: var(--reader);
  }

  .palette-button {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 10px 12px;
    color: var(--muted);
  }

  .ghost-button,
  .icon-button {
    padding: 8px 10px;
    color: var(--muted);
  }

  .sidebar-action {
    width: 100%;
  }

  kbd {
    border: 1px solid var(--line);
    border-bottom-width: 2px;
    border-radius: 7px;
    padding: 1px 6px;
    color: var(--muted);
    background: var(--panel);
    font-size: 12px;
  }

  .status-line {
    display: flex;
    justify-content: space-between;
    color: var(--muted);
    font-size: 12px;
  }

  .explorer-search {
    position: relative;
    display: grid;
    gap: 6px;
    color: var(--muted);
    font-size: 12px;
  }

  .explorer-search input {
    width: 100%;
    padding: 10px 64px 10px 11px;
    border: 1px solid var(--line);
    border-radius: 13px;
    color: var(--text);
    background: var(--panel-strong);
    font: inherit;
    outline: none;
  }

  .explorer-search kbd {
    position: absolute;
    right: 8px;
    bottom: 8px;
  }

  .error {
    margin: 0;
    padding: 10px;
    border-radius: 12px;
    background: rgba(190, 52, 52, 0.12);
    color: #d15a4f;
    font-size: 13px;
  }

  .file-list {
    min-height: 0;
    overflow: auto;
    display: flex;
    flex-direction: column;
    padding-right: 4px;
  }

  .file-row {
    display: grid;
    gap: 2px;
    width: 100%;
    height: 56px;
    min-height: 56px;
    padding: 9px 10px;
    border: 0;
    border-radius: 12px;
    background: transparent;
    color: var(--text);
    text-align: left;
    cursor: pointer;
  }

  .file-row:hover,
  .file-row.selected {
    background: var(--accent-soft);
  }

  .folder-row {
    color: var(--muted);
  }

  .folder-row:disabled {
    cursor: default;
    opacity: 0.86;
  }

  .chevron {
    display: inline-block;
    width: 16px;
    color: var(--faint);
  }

  .file-title,
  .file-path {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .file-title {
    font-size: 14px;
    font-weight: 650;
  }

  .file-path {
    color: var(--muted);
    font-size: 12px;
  }

  .empty-sidebar {
    color: var(--muted);
    padding: 14px;
    line-height: 1.5;
    border: 1px dashed var(--line);
    border-radius: 16px;
  }

  .reader-frame {
    height: 100vh;
    min-width: 0;
    min-height: 0;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    background: color-mix(in srgb, var(--reader) 72%, transparent);
  }

  .reader-topbar {
    padding: 18px 26px;
    border-bottom: 1px solid var(--line);
    background: color-mix(in srgb, var(--reader) 86%, transparent);
  }

  .reader-topbar h2 {
    font-size: 19px;
    letter-spacing: -0.025em;
  }

  .topbar-actions {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    justify-content: flex-end;
    gap: 12px;
    color: var(--muted);
    font-size: 13px;
  }

  .type-panel {
    display: grid;
    grid-template-columns: 190px minmax(0, 1fr);
    gap: 18px;
    padding: 14px 26px;
    border-bottom: 1px solid var(--line);
    background:
      linear-gradient(90deg, var(--accent-soft), transparent 42%),
      color-mix(in srgb, var(--panel-strong) 76%, transparent);
  }

  .settings-groups {
    display: grid;
    gap: 14px;
  }

  .settings-label {
    margin: 0 0 8px;
    color: var(--muted);
    font-size: 12px;
    font-weight: 750;
  }

  .type-panel h3 {
    margin: 0;
    font-size: 17px;
    letter-spacing: -0.03em;
  }

  .preset-grid {
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    gap: 10px;
  }

  .color-grid {
    grid-template-columns: repeat(5, minmax(0, 1fr));
  }

  .preset-grid button {
    display: grid;
    gap: 5px;
    padding: 12px;
    border: 1px solid var(--line);
    border-radius: 16px;
    color: var(--text);
    background: color-mix(in srgb, var(--panel) 82%, transparent);
    text-align: left;
    cursor: pointer;
  }

  .preset-grid button.active {
    border-color: var(--accent);
    background: var(--accent-soft);
  }

  .preset-grid span,
  .preset-grid small {
    color: var(--muted);
    font-size: 12px;
    line-height: 1.35;
  }

  .settings-sliders {
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    gap: 10px 14px;
    margin-top: 12px;
  }

  .settings-sliders label {
    display: grid;
    gap: 6px;
    color: var(--muted);
    font-size: 12px;
  }

  .settings-sliders span {
    display: flex;
    justify-content: space-between;
    gap: 8px;
  }

  .settings-sliders b {
    color: var(--text);
    font-weight: 650;
  }

  .settings-sliders input {
    width: 100%;
    accent-color: var(--accent);
  }

  .find-bar {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 26px;
    border-bottom: 1px solid var(--line);
    background: color-mix(in srgb, var(--panel-strong) 82%, transparent);
  }

  .find-bar input {
    width: min(360px, 45vw);
    padding: 9px 11px;
    border: 1px solid var(--line);
    border-radius: 12px;
    color: var(--text);
    background: var(--panel);
    font: inherit;
    outline: none;
  }

  .find-bar span {
    min-width: 96px;
    color: var(--muted);
    font-size: 12px;
  }

  .find-bar button {
    padding: 7px 9px;
    border: 1px solid var(--line);
    border-radius: 10px;
    color: var(--muted);
    background: var(--panel);
    cursor: pointer;
  }

  .find-bar button:disabled,
  .ghost-button:disabled {
    cursor: not-allowed;
    opacity: 0.5;
  }

  .reader-workspace {
    flex: 1 1 auto;
    min-height: 0;
    height: 0;
    display: grid;
    grid-template-columns: minmax(0, 1fr) 220px;
  }

  .reader-scroll {
    min-height: 0;
    height: 100%;
    overflow-y: auto;
    overflow-x: hidden;
    padding: 52px 32px 80px;
    -webkit-overflow-scrolling: touch;
    scroll-behavior: smooth;
    scroll-padding-top: 32px;
  }

  .reader {
    max-width: var(--reader-measure);
    margin: 0 auto;
    color: var(--text);
    font-family: var(--reader-body-font);
    font-size: var(--reader-font-size);
    font-weight: var(--reader-weight);
    line-height: var(--reader-line-height);
    letter-spacing: -0.002em;
    text-wrap: pretty;
  }

  .reader :global(::selection) {
    background: var(--highlight);
  }

  .reader :global(:target) {
    scroll-margin-top: 32px;
  }

  .reader :global(h1),
  .reader :global(h2),
  .reader :global(h3) {
    margin: 1.7em 0 0.48em;
    font-family: var(--reader-heading-font);
    color: color-mix(in srgb, var(--text) 96%, var(--accent));
    line-height: 1.13;
    letter-spacing: -0.045em;
    text-wrap: balance;
  }

  .reader :global(h1) {
    margin-top: 0;
    font-size: calc(var(--reader-font-size) * var(--reader-h1-scale));
    line-height: 1.03;
  }

  .reader :global(h2) {
    font-size: calc(var(--reader-font-size) * var(--reader-h2-scale));
  }

  .reader :global(h3) {
    color: var(--accent);
    font-size: calc(var(--reader-font-size) * var(--reader-h3-scale));
    letter-spacing: -0.035em;
  }

  .reader :global(p),
  .reader :global(ul),
  .reader :global(ol),
  .reader :global(blockquote) {
    margin: 0 0 var(--reader-paragraph-spacing);
  }

  .reader :global(ul),
  .reader :global(ol) {
    padding-left: 1.3em;
  }

  .reader :global(li) {
    margin: 0.28em 0;
  }

  .reader :global(a) {
    color: var(--link, var(--accent-strong));
    text-decoration-thickness: 0.08em;
    text-underline-offset: 0.18em;
  }

  .reader :global(blockquote) {
    margin: calc(var(--reader-paragraph-spacing) * 1.25) 0;
    padding: 0.9em 1.1em;
    border-left: 4px solid var(--blockquote-border, var(--accent));
    border-radius: 0 18px 18px 0;
    color: color-mix(in srgb, var(--text) 74%, var(--muted));
    background: var(--blockquote-bg, var(--accent-soft));
  }

  .reader :global(mark) {
    border-radius: 0.25em;
    background: var(--highlight);
    color: inherit;
  }

  .reader :global(mark[data-find-match]) {
    background: var(--find, var(--highlight));
    box-shadow: 0 0 0 2px var(--find, var(--highlight));
  }

  .reader :global(mark.active-find-match) {
    background: var(--find-active, var(--accent-soft));
    box-shadow: 0 0 0 3px var(--find-active, var(--accent-soft));
  }

  .reader :global(hr) {
    width: 34%;
    margin: 2.4em auto;
    border: 0;
    border-top: 1px solid var(--line);
  }

  .reader :global(code) {
    padding: 0.12em 0.32em;
    border-radius: 0.34em;
    color: var(--inline-code-text, var(--text));
    background: var(--inline-code-bg, var(--code-bg));
    font-family: "SF Mono", ui-monospace, monospace;
    font-size: var(--reader-code-scale);
    letter-spacing: -0.015em;
  }

  .reader :global(pre) {
    overflow: auto;
    padding: 1.1em;
    border: 1px solid var(--code-border, var(--line));
    border-radius: 16px;
    background: var(--code-block-bg, var(--code-bg));
  }

  .reader :global(pre code) {
    padding: 0;
    background: transparent;
  }

  .reader :global(table) {
    width: 100%;
    border-collapse: collapse;
    margin: 1.2em 0;
    font-size: 0.92em;
  }

  .reader :global(th),
  .reader :global(td) {
    border-bottom: 1px solid var(--table-border, var(--line));
    padding: 0.55em 0.7em;
    text-align: left;
  }

  .reader :global(img) {
    max-width: 100%;
    height: auto;
    border-radius: 18px;
    box-shadow: 0 14px 42px rgba(0, 0, 0, 0.16);
  }

  .toc-rail {
    min-height: 0;
    padding: 52px 18px 32px 0;
    border-left: 1px solid var(--line);
    background: color-mix(in srgb, var(--reader) 72%, transparent);
  }

  .toc-rail > .eyebrow {
    position: sticky;
    top: 20px;
    margin-left: 18px;
  }

  .toc-search {
    position: sticky;
    top: 42px;
    z-index: 1;
    width: calc(100% - 18px);
    margin: 8px 0 8px 18px;
    padding: 8px 9px;
    border: 1px solid var(--line);
    border-radius: 11px;
    color: var(--text);
    background: var(--panel);
    font: inherit;
    font-size: 12px;
    outline: none;
  }

  .toc-rail nav {
    position: sticky;
    top: 84px;
    display: grid;
    gap: 3px;
    max-height: calc(100vh - 160px);
    overflow: auto;
    padding: 4px 0 0 18px;
  }

  .toc-rail button {
    width: 100%;
    padding: 6px 8px;
    border: 0;
    border-left: 2px solid transparent;
    color: var(--muted);
    background: transparent;
    text-align: left;
    font: inherit;
    font-size: 12px;
    line-height: 1.35;
    cursor: pointer;
  }

  .toc-rail button:hover,
  .toc-rail button.active {
    border-left-color: var(--accent);
    color: var(--text);
    background: var(--accent-soft);
  }

  .toc-empty {
    margin: 6px 8px;
    color: var(--muted);
    font-size: 12px;
  }

  .toc-level-2 {
    padding-left: 16px !important;
  }

  .toc-level-3 {
    padding-left: 28px !important;
    font-size: 11px !important;
  }

  .hero-empty,
  .loading-card {
    max-width: 620px;
    margin: 14vh auto 0;
    padding: 42px;
    border: 1px solid var(--line);
    border-radius: 28px;
    background: var(--reader);
    box-shadow: var(--shadow);
  }

  .hero-empty h2 {
    margin: 0 0 10px;
    font-family: Newsreader, "Iowan Old Style", Georgia, serif;
    font-size: clamp(36px, 6vw, 58px);
    line-height: 1;
    letter-spacing: -0.05em;
  }

  .hero-empty p {
    color: var(--muted);
    line-height: 1.65;
  }

  .palette-backdrop {
    position: fixed;
    inset: 0;
    display: grid;
    place-items: start center;
    padding-top: 14vh;
    background: rgba(0, 0, 0, 0.26);
    backdrop-filter: blur(10px);
  }

  .palette {
    width: min(680px, calc(100vw - 32px));
    overflow: hidden;
    border: 1px solid var(--line);
    border-radius: 22px;
    background: var(--panel-strong);
    box-shadow: var(--shadow);
  }

  .palette input {
    width: 100%;
    padding: 18px 20px;
    border: 0;
    border-bottom: 1px solid var(--line);
    color: var(--text);
    background: transparent;
    font: inherit;
    font-size: 18px;
    outline: none;
  }

  .palette-meta {
    padding: 8px 14px 6px;
    border-bottom: 1px solid var(--line);
    color: var(--muted);
    font-size: 12px;
  }

  .palette-results {
    max-height: 420px;
    overflow: auto;
    padding: 8px;
  }

  .palette-results button {
    display: grid;
    gap: 3px;
    width: 100%;
    padding: 12px 14px;
    border: 0;
    border-radius: 13px;
    color: var(--text);
    background: transparent;
    text-align: left;
    cursor: pointer;
  }

  .palette-results button:hover,
  .palette-results button.active {
    background: var(--accent-soft);
  }

  .palette-results small,
  .palette-results p {
    color: var(--muted);
  }

  @media (max-width: 820px) {
    .app-shell {
      grid-template-columns: 1fr;
    }

    .sidebar {
      display: none;
    }

    .reader-scroll {
      padding: 32px 18px 64px;
    }

    .reader-workspace {
      grid-template-columns: 1fr;
    }

    .type-panel {
      grid-template-columns: 1fr;
    }

    .preset-grid {
      grid-template-columns: 1fr;
    }

    .toc-rail {
      display: none;
    }
  }
</style>
