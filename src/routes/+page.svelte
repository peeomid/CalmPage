<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { open } from "@tauri-apps/plugin-dialog";
  import { onMount, tick } from "svelte";
  import { fade } from "svelte/transition";

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
    root?: RootEntry;
    folders: Map<string, TreeFolder>;
    files: FileEntry[];
  };

  type TreeRow =
    | { type: "folder"; path: string; name: string; depth: number; root?: RootEntry }
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

  type OpenNoteTab = {
    id: string;
    file: FileEntry;
    note: RenderedNote;
    scrollTop: number;
  };

  type ColorPreset = {
    id: ColorPresetId;
    name: string;
    description: string;
    values: Record<string, string>;
  };

  type SettingsSection = "general" | "shortcuts" | "files" | "appearance" | "toc" | "markdown" | "advanced";
  type RailMode = "library" | "workspaces";
  type PaletteMode = "smart" | "actions" | "files" | "tabs" | "headings" | "settings" | "workspaces";
  type PaletteGroupId = "open-tabs" | "files" | "headings" | "actions" | "settings" | "workspaces";

  type WorkspaceEntry = {
    id: string;
    name: string;
    rootIds: string[];
    createdAt: number;
    updatedAt: number;
  };

  type CustomThemePreset = {
    id: string;
    name: string;
    colorPresetId: ColorPresetId;
    readerPresetId: ReaderPresetId;
    readerSettings: {
      fontSize: number;
      lineHeight: number;
      measure: number;
      h1Scale: number;
      h2Scale: number;
      h3Scale: number;
      paragraphSpacing: number;
      codeScale: number;
    };
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
      bodyFont: 'Outfit, ui-sans-serif, system-ui, sans-serif',
      headingFont: 'Outfit, ui-sans-serif, system-ui, sans-serif',
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
  let openTabs = $state<OpenNoteTab[]>([]);
  let activeTabId = $state<string | null>(null);
  let isOpening = $state(false);
  let isRendering = $state(false);
  let error = $state<string | null>(null);
  
  // Modals & Panels State
  let sidebarCollapsed = $state(false);
  let tocCollapsed = $state(false);
  let focusMode = $state(false);
  let settingsOpen = $state(false);
  let settingsSection = $state<SettingsSection>("appearance");
  let hudOpen = $state(false);
  let railMode = $state<RailMode>("library");
  let rootMenu = $state<{ root: RootEntry; x: number; y: number } | null>(null);
  let fileMenu = $state<{ file: FileEntry; x: number; y: number } | null>(null);
  let workspaceMenu = $state<{ workspace: WorkspaceEntry; x: number; y: number } | null>(null);
  let workspaces = $state<WorkspaceEntry[]>([]);
  let activeWorkspaceId = $state("default");
  let isCreatingWorkspace = $state(false);
  let newWorkspaceName = $state("");
  let newWorkspaceInput: HTMLInputElement | null = $state(null);
  let renamingWorkspaceId = $state<string | null>(null);
  let renamingWorkspaceName = $state("");
  let renamingWorkspaceInput: HTMLInputElement | null = $state(null);

  // Command Palette
  let paletteOpen = $state(false);
  let paletteQuery = $state("");
  let paletteMode = $state<PaletteMode>("smart");
  let paletteResults = $state<FileEntry[]>([]);
  let paletteActiveIndex = $state(0);
  let paletteResultsElement: HTMLDivElement | null = $state(null);

  // Heading Jump Palette
  let headingPaletteOpen = $state(false);
  let headingPaletteQuery = $state("");
  let headingPaletteActiveIndex = $state(0);

  // Onboarding Walkthrough
  let tourOpen = $state(false);
  let tourStep = $state(0);
  let highlightStyle = $state("");
  let cardStyle = $state("");

  // Focus mode discoverability helpers
  let focusToastVisible = $state(false);
  let focusToastTimeout: any = null;
  let focusExitHintVisible = $state(false);
  let focusExitHintTimeout: any = null;

  // Typo & Presets
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

  // Find inside doc
  let findOpen = $state(false);
  let findQuery = $state("");
  let findMatches = $state<HTMLElement[]>([]);
  let findActiveIndex = $state(0);

  // TOC Rail / Searches
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
  let readerScrolling = $state(false);
  let readerScrollTimeout = 0;
  let customThemePresets = $state<CustomThemePreset[]>([]);
  let selectedPresetKey = $state("system:paper");

  const rowHeight = 32;
  let activeWorkspace = $derived(workspaces.find((workspace) => workspace.id === activeWorkspaceId) ?? workspaces[0]);
  let activeWorkspaceRootIds = $derived(new Set(activeWorkspace?.rootIds ?? roots.map((root) => root.id)));
  let activeWorkspaceRoots = $derived(roots.filter((root) => activeWorkspaceRootIds.has(root.id)));
  let activeWorkspaceFiles = $derived(files.filter((file) => activeWorkspaceRootIds.has(file.rootId)));
  let explorerFilteredFiles = $derived(filterExplorerFiles(activeWorkspaceFiles, explorerQuery));
  let explorerTreeRoots = $derived(
    explorerQuery.trim()
      ? activeWorkspaceRoots.filter((root) => explorerFilteredFiles.some((file) => file.rootId === root.id))
      : activeWorkspaceRoots,
  );
  let explorerCollapsedFolders = $derived(explorerQuery.trim() ? new Set<string>() : collapsedFolders);
  let treeRows = $derived(buildTreeRows(explorerFilteredFiles, explorerTreeRoots, explorerCollapsedFolders));
  let maxStartIndex = $derived(Math.max(0, treeRows.length - Math.ceil(listHeight / rowHeight) - 16));
  let startIndex = $derived(Math.min(maxStartIndex, Math.max(0, Math.floor(scrollTop / rowHeight) - 8)));
  let endIndex = $derived(Math.min(treeRows.length, startIndex + Math.ceil(listHeight / rowHeight) + 16));
  let visibleRows = $derived(treeRows.slice(startIndex, endIndex));
  let topSpacer = $derived(startIndex * rowHeight);
  let bottomSpacer = $derived(Math.max(0, (treeRows.length - endIndex) * rowHeight));
  let rootLabel = $derived(activeWorkspaceRoots.length === 0 ? "No folder" : activeWorkspaceRoots.length === 1 ? activeWorkspaceRoots[0].name : `${activeWorkspaceRoots.length} folders`);
  let workspaceLabel = $derived(activeWorkspace?.name ?? "Default Workspace");
  let readerPreset = $derived(readerPresets.find((preset) => preset.id === readerPresetId) ?? readerPresets[0]);
  let readerPresetLabel = $derived(readerPresetId === "custom" ? "Custom" : readerPreset.name);
  let colorPreset = $derived(colorPresets.find((preset) => preset.id === colorPresetId) ?? colorPresets[0]);
  let tocItems = $derived(currentNote ? extractTocItems(currentNote.html) : []);
  let effectiveSidebarCollapsed = $derived(sidebarCollapsed || focusMode);
  let shouldShowToc = $derived(Boolean(currentNote && tocItems.length >= 3 && (!tocCollapsed || headingPaletteOpen)));
  let activeTabIndex = $derived(openTabs.findIndex((tab) => tab.id === activeTabId));
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

  // Walkthrough steps
  type TourStepSpec = {
    target: string;
    title: string;
    text: string;
  };

  const tourSteps: TourStepSpec[] = [
    {
      target: ".sidebar-footer",
      title: "Sidebar Actions Footer",
      text: 'All folder management tasks are placed down here. Use "Open Folder" or "Add Folder" to scan files, or click "Quick Open" to jump directly using the ⌘K command palette.'
    },
    {
      target: "#explorer-search",
      title: "Explorer Search Filter",
      text: "Type here or press ⇧⌘F at any time to dynamically filter the Markdown file list below while keeping the structural directory folders expanded."
    },
    {
      target: ".reader-topbar",
      title: "Clutter-Free Header Bar",
      text: "The top bar is simplified to the absolute minimum, keeping focus entirely on the note title. Use ⌘B to hide the left panel to maximize text width."
    },
    {
      target: 'button[title="Toggle Focus Mode"]',
      title: "Focus Reading Mode (⌘.)",
      text: "Press ⌘. to enter Focus Mode. All sidebars and headers vanish. While reading, you can quickly jump to any section by pressing / to search headings, and press ⌘. again to exit."
    },
    {
      target: 'button[title="Appearance Settings"]',
      title: "Appearance settings (⌘,)",
      text: "Click Settings or press ⌘, to configure colors, fonts, widths, and line-heights dynamically, or share preset configs using the Presets Sync tab!"
    }
  ];

  // Unified command items in Command Palette
  type CommandItem = {
    text: string;
    shortcut: string[];
    action: () => void;
  };

  type SettingItem = {
    id: string;
    title: string;
    section: SettingsSection;
    keywords: string[];
  };

  type PaletteSelectableRow =
    | { type: "command"; command: CommandItem }
    | { type: "tab"; tab: OpenNoteTab }
    | { type: "file"; file: FileEntry }
    | { type: "heading"; heading: TocItem }
    | { type: "setting"; setting: SettingItem }
    | { type: "workspace"; workspace: WorkspaceEntry };

  type PaletteGroup = {
    id: PaletteGroupId;
    label: string;
    rows: PaletteSelectableRow[];
  };

  const commandItems: CommandItem[] = [
    { text: "Toggle Left Sidebar", shortcut: ["⌘", "B"], action: toggleSidebar },
    { text: "Toggle Table of Contents", shortcut: ["⌘", "J"], action: toggleToc },
    { text: "Toggle Focus Reading Mode", shortcut: ["⌘", "."], action: toggleFocusMode },
    { text: "Next Open Tab", shortcut: ["⌘", "]"], action: () => moveTab(1) },
    { text: "Previous Open Tab", shortcut: ["⌘", "["], action: () => moveTab(-1) },
    { text: "Close Current Tab", shortcut: ["⌘", "W"], action: closeActiveTab },
    { text: "Close All Open Files", shortcut: [], action: closeAllOpenTabs },
    { text: "Search Open Tabs", shortcut: ["⌘", "O"], action: () => openPalette("tabs") },
    { text: "Find in Current Note", shortcut: ["⌘", "F"], action: openFind },
    { text: "Open Settings Studio", shortcut: ["⌘", ","], action: () => { settingsOpen = true; settingsSection = "appearance"; } },
    { text: "Restart Onboarding Guide", shortcut: [], action: () => startTour() },
    { text: "Switch Theme: Paper", shortcut: [], action: () => { colorPresetId = "paper"; theme = "light"; } },
    { text: "Switch Theme: Graphite", shortcut: [], action: () => { colorPresetId = "graphite"; theme = "dark"; } },
    { text: "Switch Theme: Polar", shortcut: [], action: () => { colorPresetId = "polar"; theme = "light"; } },
    { text: "Switch Theme: Sepia", shortcut: [], action: () => { colorPresetId = "sepia"; theme = "light"; } },
    { text: "Switch Theme: Midnight", shortcut: [], action: () => { colorPresetId = "midnight"; theme = "dark"; } }
  ];

  const settingItems: SettingItem[] = [
    { id: "general", title: "General Settings", section: "general", keywords: ["startup", "app", "behavior"] },
    { id: "shortcuts", title: "Keyboard Shortcuts", section: "shortcuts", keywords: ["keys", "command", "navigation"] },
    { id: "files", title: "Files And Library", section: "files", keywords: ["folders", "workspace", "library"] },
    { id: "appearance", title: "Appearance And Theme Presets", section: "appearance", keywords: ["theme", "font", "reader", "preview"] },
    { id: "toc", title: "Table Of Contents", section: "toc", keywords: ["toc", "headings", "outline"] },
    { id: "markdown", title: "Markdown Rendering", section: "markdown", keywords: ["html", "code", "render"] },
    { id: "advanced", title: "Advanced Settings", section: "advanced", keywords: ["debug", "data", "reset"] },
  ];

  let effectivePaletteMode = $derived(detectPaletteMode(paletteQuery, paletteMode));
  let paletteSearchText = $derived(stripPalettePrefix(paletteQuery).toLowerCase());
  let palettePlaceholder = $derived(getPalettePlaceholder(effectivePaletteMode));

  let paletteGroups = $derived.by<PaletteGroup[]>(() => {
    const q = paletteSearchText;
    const mode = effectivePaletteMode;
    const groups: PaletteGroup[] = [];

    const tabRows: PaletteSelectableRow[] = openTabs
      .filter((tab) => matchText(`${tab.note.title} ${tab.file.rootName} ${tab.file.path}`, q))
      .map((tab) => ({ type: "tab", tab }));
    const fileRows: PaletteSelectableRow[] = paletteResults
      .filter((file) => activeWorkspaceRootIds.has(file.rootId))
      .map((file) => ({ type: "file", file }));
    const headingRows: PaletteSelectableRow[] = tocItems
      .filter((heading) => matchText(heading.text, q))
      .map((heading) => ({ type: "heading", heading }));
    const commandRows: PaletteSelectableRow[] = commandItems
      .filter((command) => matchText(command.text, q))
      .map((command) => ({ type: "command", command }));
    const settingRows: PaletteSelectableRow[] = settingItems
      .filter((setting) => matchText(`${setting.title} ${setting.section} ${setting.keywords.join(" ")}`, q))
      .map((setting) => ({ type: "setting", setting }));
    const workspaceRows: PaletteSelectableRow[] = workspaces
      .filter((workspace) => matchText(workspace.name, q))
      .map((workspace) => ({ type: "workspace", workspace }));

    const addGroup = (id: PaletteGroupId, label: string, rows: PaletteSelectableRow[]) => {
      if (rows.length > 0) groups.push({ id, label, rows });
    };

    if (mode === "tabs") addGroup("open-tabs", "Open Tabs", tabRows);
    else if (mode === "files") addGroup("files", "Files", fileRows);
    else if (mode === "headings") addGroup("headings", "Headings", headingRows);
    else if (mode === "actions") addGroup("actions", "Actions", commandRows);
    else if (mode === "settings") addGroup("settings", "Settings", settingRows);
    else if (mode === "workspaces") addGroup("workspaces", "Workspaces", workspaceRows);
    else {
      addGroup("open-tabs", "Open Tabs", tabRows.slice(0, 8));
      addGroup("files", "Files", fileRows.slice(0, 12));
      addGroup("headings", "Headings", headingRows.slice(0, 8));
      addGroup("actions", "Actions", commandRows.slice(0, 8));
      addGroup("settings", "Settings", settingRows.slice(0, 8));
      addGroup("workspaces", "Workspaces", workspaceRows.slice(0, 8));
    }

    return groups;
  });

  let selectablePaletteRows = $derived(paletteGroups.flatMap((group) => group.rows));

  // Filtered Heading Palette items
  let filteredHeadingPaletteItems = $derived.by<TocItem[]>(() => {
    const q = headingPaletteQuery.trim().toLowerCase();
    if (!q) return tocItems;
    return tocItems.filter(item => item.text.toLowerCase().includes(q));
  });

  // Auto-syncing Settings JSON string
  let presetsJsonText = $state("");

  $effect(() => {
    const config = {
      theme: colorPresetId,
      typography: {
        preset: readerPresetId,
        fontSize: readerFontSize,
        lineHeight: readerLineHeight,
        measure: readerMeasure,
        paragraphSpacing: `${readerParagraphSpacing}em`,
        codeScale: `${readerCodeScale}em`
      }
    };
    presetsJsonText = JSON.stringify(config, null, 2);
  });

  function applyPresetsConfig() {
    try {
      const config = JSON.parse(presetsJsonText);
      if (config.theme && colorPresets.some((p) => p.id === config.theme)) {
        colorPresetId = config.theme;
        theme = config.theme === "graphite" || config.theme === "midnight" ? "dark" : "light";
      }
      if (config.typography) {
        const t = config.typography;
        if (t.preset && readerPresets.some((p) => p.id === t.preset)) {
          readerPresetId = t.preset;
        }
        if (typeof t.fontSize === "number") readerFontSize = t.fontSize;
        if (typeof t.lineHeight === "number") readerLineHeight = t.lineHeight;
        if (typeof t.measure === "number") readerMeasure = t.measure;
        if (t.paragraphSpacing) {
          const val = parseFloat(t.paragraphSpacing);
          if (!isNaN(val)) readerParagraphSpacing = val;
        }
        if (t.codeScale) {
          const val = parseFloat(t.codeScale);
          if (!isNaN(val)) readerCodeScale = val;
        }
      }
      alert("Configuration presets applied successfully!");
    } catch (err: any) {
      alert("Invalid JSON configuration block: " + err.message);
    }
  }

  function copyPresetsConfig() {
    navigator.clipboard.writeText(presetsJsonText);
    alert("Preset configuration copied to clipboard!");
  }

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

  function currentReaderSettings() {
    return {
      fontSize: readerFontSize,
      lineHeight: readerLineHeight,
      measure: readerMeasure,
      h1Scale: readerH1Scale,
      h2Scale: readerH2Scale,
      h3Scale: readerH3Scale,
      paragraphSpacing: readerParagraphSpacing,
      codeScale: readerCodeScale,
    };
  }

  function saveCustomThemePresets(nextPresets = customThemePresets) {
    localStorage.setItem("minimal-reader:custom-theme-presets", JSON.stringify(nextPresets));
  }

  function applyCustomThemePreset(preset: CustomThemePreset) {
    colorPresetId = preset.colorPresetId;
    theme = preset.colorPresetId === "graphite" || preset.colorPresetId === "midnight" ? "dark" : "light";
    readerPresetId = preset.readerPresetId;
    readerFontSize = preset.readerSettings.fontSize;
    readerLineHeight = preset.readerSettings.lineHeight;
    readerMeasure = preset.readerSettings.measure;
    readerH1Scale = preset.readerSettings.h1Scale;
    readerH2Scale = preset.readerSettings.h2Scale;
    readerH3Scale = preset.readerSettings.h3Scale;
    readerParagraphSpacing = preset.readerSettings.paragraphSpacing;
    readerCodeScale = preset.readerSettings.codeScale;
    selectedPresetKey = `custom:${preset.id}`;
  }

  function applySystemThemePreset(preset: ColorPreset) {
    colorPresetId = preset.id;
    theme = preset.id === "graphite" || preset.id === "midnight" ? "dark" : "light";
    selectedPresetKey = `system:${preset.id}`;
  }

  function duplicateSystemThemePreset(preset: ColorPreset) {
    const nextPreset: CustomThemePreset = {
      id: `${Date.now()}`,
      name: `${preset.name} Copy`,
      colorPresetId: preset.id,
      readerPresetId,
      readerSettings: currentReaderSettings(),
    };
    customThemePresets = [...customThemePresets, nextPreset];
    saveCustomThemePresets(customThemePresets);
    applyCustomThemePreset(nextPreset);
  }

  function saveCurrentAsPreset() {
    const name = prompt("Preset name", "My Reading Theme");
    if (!name?.trim()) return;
    const nextPreset: CustomThemePreset = {
      id: `${Date.now()}`,
      name: name.trim(),
      colorPresetId,
      readerPresetId,
      readerSettings: currentReaderSettings(),
    };
    customThemePresets = [...customThemePresets, nextPreset];
    saveCustomThemePresets(customThemePresets);
    selectedPresetKey = `custom:${nextPreset.id}`;
  }

  function updateSelectedCustomPreset() {
    if (!selectedPresetKey.startsWith("custom:")) return;
    const id = selectedPresetKey.slice("custom:".length);
    customThemePresets = customThemePresets.map((preset) =>
      preset.id === id
        ? { ...preset, colorPresetId, readerPresetId, readerSettings: currentReaderSettings() }
        : preset,
    );
    saveCustomThemePresets(customThemePresets);
  }

  function selectedPresetJson() {
    const key = selectedPresetKey;
    if (key.startsWith("custom:")) {
      const preset = customThemePresets.find((candidate) => candidate.id === key.slice("custom:".length));
      return JSON.stringify(preset ?? {}, null, 2);
    }
    const colorPreset = colorPresets.find((preset) => `system:${preset.id}` === key) ?? colorPresets[0];
    return JSON.stringify({
      name: colorPreset.name,
      locked: true,
      colors: colorPreset.values,
      typography: currentReaderSettings(),
    }, null, 2);
  }

  function copySelectedPresetJson() {
    void navigator.clipboard.writeText(selectedPresetJson());
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

  function createDefaultWorkspace(rootIds: string[] = roots.map((root) => root.id)): WorkspaceEntry {
    const now = Date.now();
    return {
      id: "default",
      name: "Default Workspace",
      rootIds,
      createdAt: now,
      updatedAt: now,
    };
  }

  function saveWorkspaces(nextWorkspaces = workspaces, nextActiveId = activeWorkspaceId) {
    localStorage.setItem("minimal-reader:workspaces", JSON.stringify(nextWorkspaces));
    localStorage.setItem("minimal-reader:active-workspace", nextActiveId);
  }

  function normalizeWorkspaces(nextRoots = roots, savedWorkspaces = workspaces, savedActiveId = activeWorkspaceId) {
    const rootIds = new Set(nextRoots.map((root) => root.id));
    let nextWorkspaces = savedWorkspaces
      .map((workspace) => ({
        ...workspace,
        rootIds: workspace.rootIds.filter((rootId) => rootIds.has(rootId)),
      }));

    if (nextWorkspaces.length === 0) {
      nextWorkspaces = [createDefaultWorkspace([...rootIds])];
    }

    if (!nextWorkspaces.some((workspace) => workspace.id === "default")) {
      nextWorkspaces = [createDefaultWorkspace([...rootIds]), ...nextWorkspaces];
    }

    const nextActiveId = nextWorkspaces.some((workspace) => workspace.id === savedActiveId)
      ? savedActiveId
      : nextWorkspaces[0].id;

    workspaces = nextWorkspaces;
    activeWorkspaceId = nextActiveId;
    saveWorkspaces(nextWorkspaces, nextActiveId);
  }

  function updateActiveWorkspaceRootIds(rootIds: string[]) {
    updateWorkspaceRootIds(activeWorkspaceId, rootIds);
  }

  function updateWorkspaceRootIds(workspaceId: string, rootIds: string[]) {
    const now = Date.now();
    workspaces = workspaces.map((workspace) =>
      workspace.id === workspaceId
        ? { ...workspace, rootIds: [...new Set(rootIds)], updatedAt: now }
        : workspace,
    );
    saveWorkspaces();
  }

  function includeRootsInActiveWorkspace(rootIds: string[]) {
    includeRootsInWorkspace(activeWorkspaceId, rootIds);
  }

  function includeRootsInWorkspace(workspaceId: string, rootIds: string[]) {
    const workspace = workspaces.find((candidate) => candidate.id === workspaceId);
    const currentIds = workspace?.rootIds ?? [];
    updateWorkspaceRootIds(workspaceId, [...currentIds, ...rootIds]);
  }

  async function startCreateWorkspace() {
    workspaceMenu = null;
    isCreatingWorkspace = true;
    newWorkspaceName = "";
    await tick();
    newWorkspaceInput?.focus();
  }

  function cancelCreateWorkspace() {
    isCreatingWorkspace = false;
    newWorkspaceName = "";
  }

  function createWorkspace() {
    const name = newWorkspaceName.trim();
    if (!name) return;
    const now = Date.now();
    const workspace: WorkspaceEntry = {
      id: `workspace-${now}`,
      name,
      rootIds: [],
      createdAt: now,
      updatedAt: now,
    };
    workspaces = [...workspaces, workspace];
    isCreatingWorkspace = false;
    newWorkspaceName = "";
    saveWorkspaces();
  }

  function switchWorkspace(id: string) {
    activeWorkspaceId = id;
    railMode = "library";
    explorerQuery = "";
    scrollTop = 0;
    rootMenu = null;
    workspaceMenu = null;
    saveWorkspaces();
  }

  async function startRenameWorkspace(workspace: WorkspaceEntry) {
    workspaceMenu = null;
    renamingWorkspaceId = workspace.id;
    renamingWorkspaceName = workspace.name;
    await tick();
    renamingWorkspaceInput?.focus();
    renamingWorkspaceInput?.select();
  }

  function cancelRenameWorkspace() {
    renamingWorkspaceId = null;
    renamingWorkspaceName = "";
  }

  function commitRenameWorkspace() {
    if (!renamingWorkspaceId) return;
    const name = renamingWorkspaceName.trim();
    if (!name) return;
    workspaces = workspaces.map((candidate) =>
      candidate.id === renamingWorkspaceId ? { ...candidate, name, updatedAt: Date.now() } : candidate,
    );
    cancelRenameWorkspace();
    saveWorkspaces();
  }

  function duplicateWorkspace(workspace: WorkspaceEntry) {
    workspaceMenu = null;
    const now = Date.now();
    const copy: WorkspaceEntry = {
      ...workspace,
      id: `workspace-${now}`,
      name: `${workspace.name} Copy`,
      createdAt: now,
      updatedAt: now,
    };
    workspaces = [...workspaces, copy];
    saveWorkspaces();
  }

  function deleteWorkspace(workspace: WorkspaceEntry) {
    workspaceMenu = null;
    if (workspace.id === "default" && workspaces.length === 1) return;
    if (!window.confirm(`Delete workspace "${workspace.name}"? Folders stay in the app.`)) return;
    const nextWorkspaces = workspaces.filter((candidate) => candidate.id !== workspace.id);
    const nextActiveId = activeWorkspaceId === workspace.id ? (nextWorkspaces[0]?.id ?? "default") : activeWorkspaceId;
    workspaces = nextWorkspaces.length > 0 ? nextWorkspaces : [createDefaultWorkspace([])];
    activeWorkspaceId = nextActiveId;
    saveWorkspaces();
  }

  function makeFolder(name: string, path: string, root?: RootEntry): TreeFolder {
    return { name, path, root, folders: new Map(), files: [] };
  }

  function buildTreeRows(sourceFiles: FileEntry[], sourceRoots: RootEntry[], collapsed: Set<string>): TreeRow[] {
    const rootFolder = makeFolder("", "");
    for (const root of sourceRoots) {
      rootFolder.folders.set(root.id, makeFolder(root.name, `root:${root.id}`, root));
    }

    for (const file of sourceFiles) {
      const root = sourceRoots.find((candidate) => candidate.id === file.rootId);
      if (!root) continue;
      let rootTreeFolder = rootFolder.folders.get(root.id);
      if (!rootTreeFolder) {
        rootTreeFolder = makeFolder(root.name, `root:${root.id}`, root);
        rootFolder.folders.set(root.id, rootTreeFolder);
      }
      let current: TreeFolder = rootTreeFolder;
      const parts = file.path.split("/");
      for (let i = 0; i < parts.length - 1; i += 1) {
        const name = parts[i];
        const folderPath = `root:${file.rootId}/${parts.slice(0, i + 1).join("/")}`;
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
        rows.push({ type: "folder", path: child.path, name: child.name, depth, root: child.root });
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
    const needle = normalizeSearchText(query);
    if (!needle) return sourceFiles;
    return sourceFiles.filter((file) =>
      normalizeSearchText(`${file.rootName} ${file.title} ${file.path}`).includes(needle),
    );
  }

  function parentFolderLabel(path: string) {
    const parts = path.split("/");
    if (parts.length <= 1) return "Workspace root";
    return parts[parts.length - 2] || "Workspace root";
  }

  function fileFullPath(file: FileEntry) {
    const root = roots.find((candidate) => candidate.id === file.rootId);
    if (!root) return file.path;
    return `${root.path.replace(/\/$/, "")}/${file.path.replace(/^\//, "")}`;
  }

  function fileRelativePath(file: FileEntry) {
    return file.path;
  }

  function rootFullPath(root: RootEntry) {
    return root.path;
  }

  function rootRelativePath(root: RootEntry) {
    return root.name;
  }

  function noteTabId(rootId: string, path: string) {
    return `${rootId}:${path}`;
  }

  function saveOpenTabsState(nextTabs = openTabs, nextActiveTabId = activeTabId) {
    localStorage.setItem(
      "minimal-reader:open-tabs",
      JSON.stringify(
        nextTabs.map((tab) => ({
          rootId: tab.file.rootId,
          path: tab.file.path,
          scrollTop: tab.scrollTop,
        })),
      ),
    );
    localStorage.setItem("minimal-reader:active-tab", nextActiveTabId ?? "");
  }

  function currentReaderScrollTop() {
    return document.querySelector<HTMLElement>(".reader-scroll")?.scrollTop ?? 0;
  }

  function rememberActiveTabScroll() {
    if (!activeTabId) return;
    const nextScrollTop = currentReaderScrollTop();
    openTabs = openTabs.map((tab) =>
      tab.id === activeTabId ? { ...tab, scrollTop: nextScrollTop } : tab,
    );
  }

  function restoreTabScroll(tab: OpenNoteTab) {
    setTimeout(() => {
      document.querySelector<HTMLElement>(".reader-scroll")?.scrollTo({ top: tab.scrollTop });
    }, 0);
  }

  function activateTab(tabId: string) {
    const tab = openTabs.find((candidate) => candidate.id === tabId);
    if (!tab) return;
    rememberActiveTabScroll();
    activeTabId = tab.id;
    currentNote = tab.note;
    selectedRootId = tab.file.rootId;
    selectedPath = tab.file.path;
    findOpen = false;
    findQuery = "";
    closeTocSearch();
    restoreTabScroll(tab);
    saveOpenTabsState();
  }

  function upsertTab(file: FileEntry, note: RenderedNote, makeActive = true) {
    const id = noteTabId(file.rootId, file.path);
    const existing = openTabs.find((tab) => tab.id === id);
    const scrollTop = existing?.scrollTop ?? 0;
    if (existing) {
      openTabs = openTabs.map((tab) =>
        tab.id === id ? { ...tab, file, note } : tab,
      );
    } else {
      openTabs = [...openTabs, { id, file, note, scrollTop }];
    }
    if (makeActive) {
      activeTabId = id;
      currentNote = note;
      selectedRootId = file.rootId;
      selectedPath = file.path;
      setTimeout(() => {
        document.querySelector<HTMLElement>(".reader-scroll")?.scrollTo({ top: scrollTop });
      }, 0);
    }
    saveOpenTabsState();
  }

  function closeTab(tabId: string) {
    const tabIndex = openTabs.findIndex((tab) => tab.id === tabId);
    if (tabIndex === -1) return;
    const wasActive = tabId === activeTabId;
    const nextTabs = openTabs.filter((tab) => tab.id !== tabId);
    openTabs = nextTabs;
    if (!wasActive) return;
    const nextTab = nextTabs[Math.min(tabIndex, nextTabs.length - 1)];
    if (nextTab) {
      activeTabId = nextTab.id;
      currentNote = nextTab.note;
      selectedRootId = nextTab.file.rootId;
      selectedPath = nextTab.file.path;
      restoreTabScroll(nextTab);
    } else {
      activeTabId = null;
      currentNote = null;
      selectedRootId = null;
      selectedPath = null;
    }
    saveOpenTabsState(nextTabs, nextTab?.id ?? null);
  }

  function closeActiveTab() {
    if (activeTabId) closeTab(activeTabId);
  }

  function closeAllOpenTabs() {
    rememberActiveTabScroll();
    openTabs = [];
    activeTabId = null;
    currentNote = null;
    selectedRootId = null;
    selectedPath = null;
    findOpen = false;
    findQuery = "";
    closeTocSearch();
    saveOpenTabsState([], null);
  }

  function moveTab(delta: number) {
    if (openTabs.length < 2) return;
    const currentIndex = activeTabIndex >= 0 ? activeTabIndex : 0;
    const nextIndex = (currentIndex + delta + openTabs.length) % openTabs.length;
    activateTab(openTabs[nextIndex].id);
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
    normalizeWorkspaces(snapshot.roots);
    const existingFiles = new Set(snapshot.files.map((file) => noteTabId(file.rootId, file.path)));
    openTabs = openTabs.filter((tab) => existingFiles.has(tab.id));
    if (activeTabId && !openTabs.some((tab) => tab.id === activeTabId)) {
      const nextTab = openTabs[0];
      if (nextTab) {
        activeTabId = nextTab.id;
        currentNote = nextTab.note;
        selectedRootId = nextTab.file.rootId;
        selectedPath = nextTab.file.path;
      } else {
        activeTabId = null;
        currentNote = null;
        selectedRootId = null;
        selectedPath = null;
      }
    }
    if (
      selectedPath &&
      selectedRootId &&
      !snapshot.files.some((file) => file.rootId === selectedRootId && file.path === selectedPath)
    ) {
      selectedPath = null;
      selectedRootId = null;
      currentNote = null;
      activeTabId = null;
    }
    saveOpenTabsState();
  }

  function collapseAllFolders(sourceFiles = files) {
    const folders = new Set<string>();
    for (const root of roots) {
      folders.add(`root:${root.id}`);
    }
    for (const file of sourceFiles) {
      const parts = [`root:${file.rootId}`, ...file.path.split("/")];
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
    const targetWorkspaceId = activeWorkspaceId;
    try {
      const snapshot = await invoke<VaultSnapshot>("open_vaults", { paths });
      applySnapshot(snapshot);
      collapseAllFolders(snapshot.files);
      saveRoots(snapshot.roots);
      includeRootsInWorkspace(targetWorkspaceId, snapshot.roots.map((root) => root.id));
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
    const targetWorkspaceId = activeWorkspaceId;
    const previousRootIds = new Set(roots.map((root) => root.id));
    try {
      const snapshot = await invoke<VaultSnapshot>("add_vault", { path: selected });
      applySnapshot(snapshot);
      collapseAllFolders(snapshot.files);
      saveRoots(snapshot.roots);
      const addedRootIds = snapshot.roots
        .filter((root) => !previousRootIds.has(root.id) || root.path === selected)
        .map((root) => root.id);
      includeRootsInWorkspace(targetWorkspaceId, addedRootIds);
    } catch (err) {
      error = err instanceof Error ? err.message : String(err);
    } finally {
      isOpening = false;
    }
  }

  async function removeFolder(root: RootEntry) {
    rootMenu = null;
    isOpening = true;
    error = null;
    try {
      const snapshot = await invoke<VaultSnapshot>("remove_vault", { rootId: root.id });
      applySnapshot(snapshot);
      collapseAllFolders(snapshot.files);
      saveRoots(snapshot.roots);
      normalizeWorkspaces(snapshot.roots);
    } catch (err) {
      error = err instanceof Error ? err.message : String(err);
    } finally {
      isOpening = false;
    }
  }

  async function openNote(file: FileEntry) {
    const tabId = noteTabId(file.rootId, file.path);
    if (openTabs.some((tab) => tab.id === tabId)) {
      activateTab(tabId);
      paletteOpen = false;
      return;
    }
    const requestId = ++noteRequestId;
    rememberActiveTabScroll();
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
        upsertTab(file, note);
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
    const targetWorkspaceId = activeWorkspaceId;
    isRendering = true;
    error = null;
    try {
      const opened = await invoke<OpenedMarkdown>("open_markdown_file", { path });
      if (requestId !== noteRequestId) return;
      applySnapshot(opened.snapshot);
      collapseAllFolders(opened.snapshot.files);
      saveRoots(opened.snapshot.roots);
      includeRootsInWorkspace(targetWorkspaceId, [opened.rootId]);
      selectedRootId = opened.rootId;
      selectedPath = opened.path;
      const file = opened.snapshot.files.find((candidate) => candidate.rootId === opened.rootId && candidate.path === opened.path) ?? {
        rootId: opened.rootId,
        rootName: opened.snapshot.roots.find((root) => root.id === opened.rootId)?.name ?? "Folder",
        path: opened.path,
        title: opened.note.title,
        modified: opened.note.modified,
        size: 0,
      };
      upsertTab(file, opened.note);
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
    const searchText = normalizeSearchText(stripPalettePrefix(query));
    const results = await invoke<FileEntry[]>("search_files", {
      query: searchText,
      limit: 30,
    });
    if (requestId === paletteRequestId) {
      paletteResults = results;
      paletteActiveIndex = 0;
    }
  }

  function openPalette(mode: PaletteMode = "smart") {
    paletteOpen = true;
    paletteMode = mode;
    rootMenu = null;
    fileMenu = null;
    workspaceMenu = null;
    paletteQuery = getPalettePrefix(mode);
    paletteActiveIndex = 0;
    void refreshPaletteResults(paletteQuery);
    setTimeout(() => {
      document.querySelector<HTMLInputElement>("#palette-input")?.focus();
    }, 0);
  }

  function togglePalette() {
    if (paletteOpen) {
      paletteOpen = false;
      return;
    }
    openPalette("smart");
  }

  function showRootMenu(event: MouseEvent, root: RootEntry) {
    event.preventDefault();
    workspaceMenu = null;
    fileMenu = null;
    rootMenu = {
      root,
      x: Math.min(event.clientX, window.innerWidth - 220),
      y: Math.min(event.clientY, window.innerHeight - 150),
    };
  }

  function showFileMenu(event: MouseEvent, file: FileEntry) {
    event.preventDefault();
    rootMenu = null;
    workspaceMenu = null;
    fileMenu = {
      file,
      x: Math.min(event.clientX, window.innerWidth - 220),
      y: Math.min(event.clientY, window.innerHeight - 150),
    };
  }

  function showWorkspaceMenu(event: MouseEvent, workspace: WorkspaceEntry) {
    event.preventDefault();
    rootMenu = null;
    fileMenu = null;
    workspaceMenu = {
      workspace,
      x: Math.min(event.clientX, window.innerWidth - 230),
      y: Math.min(event.clientY, window.innerHeight - 170),
    };
  }

  function showWorkspaces() {
    railMode = "workspaces";
    sidebarCollapsed = false;
    rootMenu = null;
    fileMenu = null;
    workspaceMenu = null;
  }

  function showLibrary() {
    railMode = "library";
    sidebarCollapsed = false;
    rootMenu = null;
    fileMenu = null;
    workspaceMenu = null;
  }

  function handleWindowClick() {
    rootMenu = null;
    fileMenu = null;
    workspaceMenu = null;
  }

  function detectPaletteMode(query: string, fallback: PaletteMode): PaletteMode {
    const trimmed = query.trimStart();
    const prefix = trimmed[0];
    if (!prefix) return fallback === "smart" ? "smart" : fallback;
    if (prefix === ">") return "actions";
    if (prefix === "/") return "files";
    if (prefix === "@") return "tabs";
    if (prefix === "#") return "headings";
    if (prefix === "?") return "settings";
    if (prefix === ":") return "workspaces";
    return "smart";
  }

  function stripPalettePrefix(query: string) {
    const trimmed = query.trimStart();
    return /^[>/@#?:]/.test(trimmed) ? trimmed.slice(1).trimStart() : query.trim();
  }

  function getPalettePrefix(mode: PaletteMode) {
    if (mode === "actions") return "> ";
    if (mode === "files") return "/ ";
    if (mode === "tabs") return "@ ";
    if (mode === "headings") return "# ";
    if (mode === "settings") return "? ";
    if (mode === "workspaces") return ": ";
    return "";
  }

  function getPalettePlaceholder(mode: PaletteMode) {
    if (mode === "actions") return "> Run action...";
    if (mode === "files") return "/ Open markdown file...";
    if (mode === "tabs") return "@ Switch open tab...";
    if (mode === "headings") return "# Jump to heading...";
    if (mode === "settings") return "? Search settings...";
    if (mode === "workspaces") return ": Switch workspace...";
    return "Search files, actions, tabs...";
  }

  function matchText(text: string, query: string) {
    return !query || normalizeSearchText(text).includes(normalizeSearchText(query));
  }

  function normalizeSearchText(value: string) {
    return value
      .toLowerCase()
      .replace(/-\s+/g, "-")
      .replace(/\s*\/\s*/g, "/")
      .replace(/\s+/g, " ")
      .trim();
  }

  function runPaletteRow(row: PaletteSelectableRow) {
    if (row.type === "command") {
      row.command.action();
    } else if (row.type === "tab") {
      activateTab(row.tab.id);
    } else if (row.type === "file") {
      void openNote(row.file);
    } else if (row.type === "heading") {
      jumpToHeading(row.heading.id);
    } else if (row.type === "setting") {
      settingsOpen = true;
      settingsSection = row.setting.section;
    } else if (row.type === "workspace") {
      switchWorkspace(row.workspace.id);
    }
    paletteOpen = false;
  }

  function paletteRowIndex(row: PaletteSelectableRow) {
    return selectablePaletteRows.indexOf(row);
  }

  async function scrollActivePaletteRowIntoView() {
    if (!paletteOpen || !paletteResultsElement) return;
    await tick();

    const activeRow = paletteResultsElement.querySelector<HTMLElement>(`[data-palette-index="${paletteActiveIndex}"]`);
    if (!activeRow) return;

    const containerRect = paletteResultsElement.getBoundingClientRect();
    const rowRect = activeRow.getBoundingClientRect();
    const topGap = rowRect.top - containerRect.top;
    const bottomGap = rowRect.bottom - containerRect.bottom;

    if (topGap < 0) {
      paletteResultsElement.scrollTop += topGap - 8;
    } else if (bottomGap > 0) {
      paletteResultsElement.scrollTop += bottomGap + 8;
    }
  }

  function scrollTabs(delta: number) {
    document.querySelector<HTMLElement>(".tab-strip")?.scrollBy({ left: delta, behavior: "smooth" });
  }

  // Heading Jump Palette Toggle
  function toggleHeadingPalette() {
    headingPaletteOpen = !headingPaletteOpen;
    if (headingPaletteOpen) {
      tocQuery = "";
      headingPaletteQuery = "";
      headingPaletteActiveIndex = 0;
      setTimeout(() => {
        document.querySelector<HTMLInputElement>("#toc-search")?.focus();
      }, 100);
    }
  }

  function exitFocusMode() {
    focusMode = false;
    focusToastVisible = false;
    focusExitHintVisible = false;
    if (focusToastTimeout) clearTimeout(focusToastTimeout);
    if (focusExitHintTimeout) clearTimeout(focusExitHintTimeout);
  }

  function toggleSidebar() {
    if (focusMode) {
      showFocusToast();
      return;
    }
    sidebarCollapsed = !sidebarCollapsed;
  }

  function toggleToc() {
    if (!currentNote || tocItems.length < 3) return;
    if (headingPaletteOpen) {
      closeTocSearch();
      return;
    }
    tocCollapsed = !tocCollapsed;
  }

  function closeTocSearch() {
    headingPaletteOpen = false;
    tocQuery = "";
    headingPaletteActiveIndex = 0;
  }

  function hideToc() {
    closeTocSearch();
    tocCollapsed = true;
  }

  function scrollReaderBy(delta: number) {
    document.querySelector<HTMLElement>(".reader-scroll")?.scrollBy({ top: delta, behavior: "smooth" });
  }

  function scrollReaderTo(position: "top" | "bottom") {
    const scrollContainer = document.querySelector<HTMLElement>(".reader-scroll");
    if (!scrollContainer) return;
    scrollContainer.scrollTo({ top: position === "top" ? 0 : scrollContainer.scrollHeight, behavior: "smooth" });
  }

  function handleReaderScroll() {
    readerScrolling = true;
    if (activeTabId) {
      const nextScrollTop = currentReaderScrollTop();
      openTabs = openTabs.map((tab) =>
        tab.id === activeTabId ? { ...tab, scrollTop: nextScrollTop } : tab,
      );
      saveOpenTabsState();
    }
    window.clearTimeout(readerScrollTimeout);
    readerScrollTimeout = window.setTimeout(() => {
      readerScrolling = false;
    }, 700);
  }

  function handleReaderNavigation(event: KeyboardEvent) {
    if (!currentNote || isTypingTarget(event.target)) return false;
    if (paletteOpen || findOpen || settingsOpen || hudOpen || tourOpen || headingPaletteOpen) return false;
    if (event.metaKey || event.ctrlKey || event.altKey) return false;
    const key = event.key;
    const lowerKey = key.toLowerCase();
    const line = 72;
    const page = Math.max(260, Math.floor(window.innerHeight * 0.72));
    if (key === "ArrowDown" || lowerKey === "j") {
      scrollReaderBy(line);
    } else if (key === "ArrowUp" || lowerKey === "k") {
      scrollReaderBy(-line);
    } else if (key === "PageDown" || key === " ") {
      scrollReaderBy(page);
    } else if (key === "PageUp") {
      scrollReaderBy(-page);
    } else if (key === "g" && !event.metaKey && !event.ctrlKey && !event.altKey && !event.shiftKey) {
      scrollReaderTo("top");
    } else if (key === "G") {
      scrollReaderTo("bottom");
    } else {
      return false;
    }
    event.preventDefault();
    return true;
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
      paletteActiveIndex = Math.min(selectablePaletteRows.length - 1, paletteActiveIndex + 1);
    } else if (event.key === "ArrowUp") {
      event.preventDefault();
      paletteActiveIndex = Math.max(0, paletteActiveIndex - 1);
    } else if (event.key === "Enter") {
      event.preventDefault();
      const activeItem = selectablePaletteRows[paletteActiveIndex];
      if (activeItem) {
        runPaletteRow(activeItem);
      }
    }
  }

  function handlePaletteDialogKeydown(event: KeyboardEvent) {
    event.stopPropagation();
    if (event.key === "Escape") {
      event.preventDefault();
      paletteOpen = false;
    }
  }

  function handleHeadingPaletteKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      event.preventDefault();
      closeTocSearch();
    } else if (event.key === "ArrowDown") {
      event.preventDefault();
      headingPaletteActiveIndex = Math.min(filteredHeadingPaletteItems.length - 1, headingPaletteActiveIndex + 1);
    } else if (event.key === "ArrowUp") {
      event.preventDefault();
      headingPaletteActiveIndex = Math.max(0, headingPaletteActiveIndex - 1);
    } else if (event.key === "Enter") {
      event.preventDefault();
      const activeItem = filteredHeadingPaletteItems[headingPaletteActiveIndex];
      if (activeItem) {
        jumpToHeading(activeItem.id);
        closeTocSearch();
      }
    }
  }

  function handleTocSearchKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      event.preventDefault();
      closeTocSearch();
    } else if (event.key === "ArrowDown") {
      event.preventDefault();
      headingPaletteActiveIndex = Math.min(filteredTocItems.length - 1, headingPaletteActiveIndex + 1);
    } else if (event.key === "ArrowUp") {
      event.preventDefault();
      headingPaletteActiveIndex = Math.max(0, headingPaletteActiveIndex - 1);
    } else if (event.key === "Enter") {
      event.preventDefault();
      const activeItem = filteredTocItems[headingPaletteActiveIndex];
      if (activeItem) {
        jumpToHeading(activeItem.id);
        closeTocSearch();
      }
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

  // Walkthrough Onboarding guides
  function startTour() {
    tourOpen = true;
    tourStep = 0;
    settingsOpen = false;
    findOpen = false;
    hudOpen = false;
    headingPaletteOpen = false;
    paletteOpen = false;
    sidebarCollapsed = false;
    updateTourLayout();
  }

  function exitTour() {
    tourOpen = false;
    localStorage.setItem("minimal-reader:onboarded", "true");
  }

  function updateTourLayout() {
    if (!tourOpen) return;
    const step = tourSteps[tourStep];
    if (!step) return;

    if (step.target.includes("sidebar") || step.target.includes("explorer") || step.target.includes("footer")) {
      sidebarCollapsed = false;
    }

    setTimeout(() => {
      const targetEl = document.querySelector(step.target);
      if (targetEl) {
        const rect = targetEl.getBoundingClientRect();
        highlightStyle = `
          top: ${rect.top - 5}px;
          left: ${rect.left - 5}px;
          width: ${rect.width + 10}px;
          height: ${rect.height + 10}px;
          opacity: 1;
        `;

        let top = 0;
        let left = Math.min(window.innerWidth - 320, Math.max(20, rect.left + rect.width / 2 - 145));

        if (rect.bottom + 200 < window.innerHeight) {
          top = rect.bottom + 12;
        } else {
          top = rect.top - 180;
        }

        cardStyle = `
          top: ${top}px;
          left: ${left}px;
          opacity: 1;
        `;
      } else {
        highlightStyle = "display: none;";
        cardStyle = `
          top: 50%;
          left: 50%;
          transform: translate(-50%, -50%);
          opacity: 1;
        `;
      }
    }, 100);
  }

  function nextTourStep() {
    if (tourStep === tourSteps.length - 1) {
      exitTour();
    } else {
      tourStep++;
      updateTourLayout();
    }
  }

  function prevTourStep() {
    if (tourStep > 0) {
      tourStep--;
      updateTourLayout();
    }
  }

  // Toast controllers for Focus mode
  function showFocusToast() {
    focusToastVisible = true;
    if (focusToastTimeout) clearTimeout(focusToastTimeout);
    focusToastTimeout = setTimeout(() => {
      focusToastVisible = false;
    }, 4000);
  }

  function toggleFocusMode() {
    if (focusMode) {
      exitFocusMode();
    } else {
      focusMode = true;
      tocCollapsed = true;
      closeTocSearch();
      showFocusToast();
    }
  }

  function openAppearanceSettings() {
    settingsOpen = true;
    settingsSection = "appearance";
  }

  function handleMouseMove() {
    if (!focusMode) return;
    focusExitHintVisible = true;
    if (focusExitHintTimeout) clearTimeout(focusExitHintTimeout);
    focusExitHintTimeout = setTimeout(() => {
      focusExitHintVisible = false;
    }, 3000);
  }

  function snapshotFile(rootId: string, path: string) {
    return files.find((file) => file.rootId === rootId && file.path === path) ?? null;
  }

  async function restoreOpenTabsFromStorage() {
    const savedOpenTabs = safeJson<Array<{ rootId: string; path: string; scrollTop?: number }>>("minimal-reader:open-tabs", []);
    const savedActiveTabId = localStorage.getItem("minimal-reader:active-tab") || null;
    if (savedOpenTabs.length === 0) return;

    openTabs = [];
    activeTabId = null;
    currentNote = null;
    selectedRootId = null;
    selectedPath = null;

    for (const savedTab of savedOpenTabs) {
      const file = snapshotFile(savedTab.rootId, savedTab.path);
      if (!file) continue;
      try {
        const note = await invoke<RenderedNote>("render_note", { rootId: savedTab.rootId, path: savedTab.path });
        upsertTab(file, note, false);
        if (savedTab.scrollTop !== undefined) {
          const tabId = noteTabId(file.rootId, file.path);
          openTabs = openTabs.map((tab) => (tab.id === tabId ? { ...tab, scrollTop: savedTab.scrollTop ?? 0 } : tab));
        }
      } catch {
        continue;
      }
    }

    if (savedActiveTabId && openTabs.some((tab) => tab.id === savedActiveTabId)) {
      activateTab(savedActiveTabId);
    } else if (openTabs.length > 0) {
      activateTab(openTabs[0].id);
    } else {
      saveOpenTabsState([], null);
    }
  }

  onMount(() => {
    const savedTheme = localStorage.getItem("minimal-reader:theme") as "light" | "dark" | null;
    if (savedTheme) theme = savedTheme;

    const savedColorPreset = localStorage.getItem("minimal-reader:color-preset") as ColorPresetId | null;
    if (savedColorPreset && colorPresets.some((preset) => preset.id === savedColorPreset)) {
      colorPresetId = savedColorPreset;
    } else {
      colorPresetId = theme === "dark" ? "graphite" : "paper";
    }

    const savedReaderPreset = localStorage.getItem("minimal-reader:reader-preset") as ReaderPresetId | null;
    if (savedReaderPreset && readerPresets.some((preset) => preset.id === savedReaderPreset)) {
      readerPresetId = savedReaderPreset;
    }
    customThemePresets = safeJson<CustomThemePreset[]>("minimal-reader:custom-theme-presets", []);
    loadReaderSettings();

    workspaces = safeJson<WorkspaceEntry[]>("minimal-reader:workspaces", []);
    activeWorkspaceId = localStorage.getItem("minimal-reader:active-workspace") ?? "default";
    if (workspaces.length === 0) {
      workspaces = [createDefaultWorkspace([])];
      saveWorkspaces();
    }

    const folders = safeJson<string[]>("minimal-reader:folders", []);
    if (folders.length > 0) {
      isOpening = true;
      invoke<VaultSnapshot>("open_vaults", { paths: folders })
        .then((snapshot) => {
          applySnapshot(snapshot);
          collapseAllFolders(snapshot.files);
          normalizeWorkspaces(snapshot.roots);
          return restoreOpenTabsFromStorage();
        })
        .catch((err) => {
          error = err instanceof Error ? err.message : String(err);
        })
        .finally(() => {
          isOpening = false;
        });
    }

    const keydown = (event: KeyboardEvent) => {
      const isCmd = event.metaKey || event.ctrlKey;
      const isTyping = isTypingTarget(event.target);

      // Escape closes overlays only. Focus mode exits only via Cmd+.
      if (event.key === "Escape") {
        if (rootMenu) {
          rootMenu = null;
          event.preventDefault();
        } else if (focusMode) {
          closeTocSearch();
          paletteOpen = false;
          hudOpen = false;
          event.preventDefault();
        } else if (headingPaletteOpen) {
          closeTocSearch();
          event.preventDefault();
        } else if (paletteOpen) {
          paletteOpen = false;
          event.preventDefault();
        } else if (findOpen) {
          findOpen = false;
          findQuery = "";
          clearFindHighlights();
          event.preventDefault();
        } else if (settingsOpen) {
          settingsOpen = false;
          event.preventDefault();
        } else if (hudOpen) {
          hudOpen = false;
          event.preventDefault();
        }
        return;
      }

      if (isCmd && event.shiftKey && event.key.toLowerCase() === "k") {
        event.preventDefault();
        openPalette("actions");
        return;
      }

      if (isCmd && !event.shiftKey && event.key.toLowerCase() === "k") {
        event.preventDefault();
        openPalette("smart");
        return;
      }

      if (isCmd && !event.shiftKey && event.key.toLowerCase() === "p") {
        event.preventDefault();
        openPalette("files");
        return;
      }

      if (isCmd && event.shiftKey && event.key.toLowerCase() === "f") {
        if (isTyping) return;
        event.preventDefault();
        focusExplorerSearch();
        return;
      }

      if (isCmd && !event.altKey && !event.shiftKey && event.key.toLowerCase() === "f") {
        event.preventDefault();
        openFind();
        return;
      }

      if (isCmd && event.shiftKey && event.key.toLowerCase() === "o") {
        event.preventDefault();
        openPalette("headings");
        return;
      }

      if (isCmd && !event.shiftKey && event.key.toLowerCase() === "o") {
        event.preventDefault();
        openPalette("tabs");
        return;
      }

      if (isCmd && event.key.toLowerCase() === "b") {
        event.preventDefault();
        toggleSidebar();
      }

      if ((isCmd && event.key.toLowerCase() === "j") || (isCmd && event.altKey && event.key.toLowerCase() === "t")) {
        event.preventDefault();
        toggleToc();
      }

      if (isCmd && (event.key === "." || event.code === "Period")) {
        event.preventDefault();
        toggleFocusMode();
      }

      if (isCmd && (event.key === "]" || event.code === "BracketRight")) {
        event.preventDefault();
        moveTab(1);
      }

      if (isCmd && (event.key === "[" || event.code === "BracketLeft")) {
        event.preventDefault();
        moveTab(-1);
      }

      if (isCmd && event.key.toLowerCase() === "w") {
        event.preventDefault();
        closeActiveTab();
      }

      if (isCmd && event.key === ",") {
        event.preventDefault();
        settingsOpen = !settingsOpen;
        settingsSection = "appearance";
      }

      if (event.key === "?") {
        if (isTyping) return;
        event.preventDefault();
        hudOpen = !hudOpen;
      }

      if (!isCmd && event.key === "/" && !isTyping) {
        event.preventDefault();
        toggleHeadingPalette();
      }

      if (isCmd && event.key.toLowerCase() === "g" && findOpen && findMatches.length > 0) {
        event.preventDefault();
        scrollToFindMatch(findActiveIndex + (event.shiftKey ? -1 : 1));
      }

      handleReaderNavigation(event);
    };
    window.addEventListener("keydown", keydown, { capture: true });

    const unlistenPromise = listen<VaultSnapshot>("vault-updated", (event) => {
      applySnapshot(event.payload);
      if (paletteOpen) void refreshPaletteResults();
    });
    const unlistenOpenedFilesPromise = listen<string[]>("opened-markdown-files", (event) => {
      const [path] = event.payload;
      if (path) void openMarkdownFilePath(path);
    });

    // Auto walkthrough tour guide on first visit
    const hasSeenTour = localStorage.getItem("minimal-reader:onboarded") === "true";
    if (!hasSeenTour) {
      setTimeout(() => {
        startTour();
      }, 800);
    }

    return () => {
      window.removeEventListener("keydown", keydown, { capture: true });
      headingObserver?.disconnect();
      void unlistenPromise.then((unlisten) => unlisten());
      void unlistenOpenedFilesPromise.then((unlisten) => unlisten());
      if (focusToastTimeout) clearTimeout(focusToastTimeout);
      if (focusExitHintTimeout) clearTimeout(focusExitHintTimeout);
      window.clearTimeout(readerScrollTimeout);
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
    }, 40);
  });

  $effect(() => {
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

  $effect(() => {
    if (paletteActiveIndex >= selectablePaletteRows.length) {
      paletteActiveIndex = Math.max(0, selectablePaletteRows.length - 1);
    }
  });

  $effect(() => {
    paletteOpen;
    paletteActiveIndex;
    selectablePaletteRows.length;
    void scrollActivePaletteRowIntoView();
  });
</script>

<svelte:head>
  <meta
    name="description"
    content="A minimal, fast Markdown reader with premium typography."
  />
</svelte:head>

<svelte:window onclick={handleWindowClick} onmousemove={handleMouseMove} onresize={updateTourLayout} />

<main
  class="app-shell"
  class:focus-mode={focusMode}
  class:sidebar-collapsed={effectiveSidebarCollapsed}
>
  <!-- LEFT RAIL -->
  <nav class="rail" aria-label="Primary navigation">
    <div class="rail-brand" title="Minimal Reader">M</div>
    <button class="rail-button rail-icon-library" class:active={railMode === "library" && !settingsOpen} onclick={showLibrary} title="Library">
      <span aria-hidden="true">▰</span>
    </button>
    <button class="rail-button rail-icon-workspaces" class:active={railMode === "workspaces"} onclick={showWorkspaces} title="Workspaces">
      <span aria-hidden="true">▣</span>
    </button>
    <button class="rail-button" class:active={paletteOpen} onclick={togglePalette} title="Command Palette">⌘</button>
    <button
      class="rail-button"
      class:active={settingsOpen}
      onclick={() => { settingsOpen = !settingsOpen; if (settingsOpen) settingsSection = "appearance"; rootMenu = null; }}
      title="Settings"
    >⚙</button>
    <button class="rail-button rail-bottom rail-icon-help" class:active={hudOpen} onclick={() => { hudOpen = true; rootMenu = null; }} title="Shortcut Help">⌨</button>
  </nav>

  <!-- LEFT SIDEBAR -->
  <aside class="sidebar">
    <div class="brand">
      <div>
        <p class="eyebrow">{railMode === "workspaces" ? "Workspaces" : "Library"}</p>
        <h1>{railMode === "workspaces" ? workspaceLabel : rootLabel}</h1>
      </div>
      {#if railMode === "library"}
        <button class="root-add-button" onclick={addFolder} disabled={isOpening} title="Add Folder">+</button>
      {/if}
    </div>

    {#if railMode === "workspaces"}
      <section class="workspace-manager" aria-label="Workspace manager">
        <div class="workspace-hero">
          <span>Active Workspace</span>
          <strong>{workspaceLabel}</strong>
          <small>{activeWorkspaceRoots.length} folders · {activeWorkspaceFiles.length} markdown files</small>
        </div>
        <div class="workspace-actions">
          <button class="btn-sidebar-footer-primary" onclick={startCreateWorkspace}>
            New Workspace
          </button>
        </div>

        {#if isCreatingWorkspace}
          <form class="workspace-create-form" onsubmit={(event) => { event.preventDefault(); createWorkspace(); }}>
            <label>
              <span>Workspace name</span>
              <input
                bind:this={newWorkspaceInput}
                bind:value={newWorkspaceName}
                placeholder="Writing, Work, Research..."
                maxlength="80"
                onkeydown={(event) => {
                  if (event.key === "Escape") {
                    event.preventDefault();
                    cancelCreateWorkspace();
                  }
                }}
              />
            </label>
            <div class="workspace-create-actions">
              <button type="submit" class="btn-sidebar-footer-primary" disabled={!newWorkspaceName.trim()}>
                Create
              </button>
              <button type="button" class="btn-sidebar-footer-ghost" onclick={cancelCreateWorkspace}>
                Cancel
              </button>
            </div>
          </form>
        {/if}

        <div class="workspace-roots-title">
          <span>Saved Workspaces</span>
          <span>Right-click for actions</span>
        </div>
        <div class="workspace-card-list">
          {#each workspaces as workspace (workspace.id)}
            {#if renamingWorkspaceId === workspace.id}
              <form class="workspace-rename-form" onsubmit={(event) => { event.preventDefault(); commitRenameWorkspace(); }}>
                <input
                  bind:this={renamingWorkspaceInput}
                  bind:value={renamingWorkspaceName}
                  aria-label={`Rename ${workspace.name}`}
                  maxlength="80"
                  onkeydown={(event) => {
                    if (event.key === "Escape") {
                      event.preventDefault();
                      cancelRenameWorkspace();
                    }
                  }}
                />
                <button type="submit" disabled={!renamingWorkspaceName.trim()}>Save</button>
                <button type="button" onclick={cancelRenameWorkspace}>Cancel</button>
              </form>
            {:else}
              <button
                class="workspace-card"
                class:active={workspace.id === activeWorkspaceId}
                title={workspace.name}
                onclick={() => switchWorkspace(workspace.id)}
                oncontextmenu={(event) => showWorkspaceMenu(event, workspace)}
              >
                <span>
                  <strong>{workspace.name}</strong>
                  <small>{workspace.rootIds.length} folders</small>
                </span>
                <span class="workspace-card-count">
                  {files.filter((file) => workspace.rootIds.includes(file.rootId)).length}
                </span>
                <span
                  class="workspace-card-menu"
                  role="button"
                  tabindex="0"
                  onclick={(event) => { event.stopPropagation(); showWorkspaceMenu(event, workspace); }}
                  onkeydown={(event) => {
                    if (event.key === "Enter" || event.key === " ") {
                      event.preventDefault();
                      const rect = event.currentTarget.getBoundingClientRect();
                      workspaceMenu = { workspace, x: rect.left, y: rect.bottom + 6 };
                    }
                  }}
                >⋯</span>
              </button>
            {/if}
          {:else}
            <div class="empty-sidebar">Create a workspace to group folders.</div>
          {/each}
        </div>
      </section>
    {:else}
      <label class="explorer-search">
        <span>Explorer filter</span>
        <input
          id="explorer-search"
          bind:value={explorerQuery}
          oninput={resetExplorerScroll}
          placeholder="Filter tree... (⇧⌘F)"
          autocomplete="off"
        />
      </label>

      <div class="status-line">
        <span>{explorerQuery.trim() ? `${explorerFilteredFiles.length}/${activeWorkspaceFiles.length}` : activeWorkspaceFiles.length} files</span>
        <span>{activeWorkspaceRoots.length > 0 ? "Watching" : "Idle"}</span>
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
        {#if activeWorkspaceRoots.length === 0}
          <div class="empty-sidebar">
            Use + to add a folder to this workspace.
          </div>
        {:else if explorerQuery.trim() && explorerFilteredFiles.length === 0}
          <div class="empty-sidebar">
            No files match "{explorerQuery}".
          </div>
        {:else}
          <div style={`height: ${topSpacer}px`} aria-hidden="true"></div>
          {#each visibleRows as row (row.type === "folder" ? `folder:${row.path}` : `file:${row.file.rootId}:${row.file.path}`)}
            {#if row.type === "folder"}
              <button
                class="file-row folder-row"
                class:root-folder-row={Boolean(row.root)}
                onclick={() => !explorerQuery.trim() && toggleFolder(row.path)}
                oncontextmenu={(event) => row.root && showRootMenu(event, row.root)}
                title={row.root?.path ?? row.path}
                style={`padding-left: ${10 + row.depth * 14}px`}
              >
                <span class="file-title">
                  <span class="chevron">{explorerCollapsedFolders.has(row.path) ? "›" : "⌄"}</span>
                  <span class="row-icon" aria-hidden="true">{row.root ? "📁" : "▸"}</span>
                  {row.name}
                </span>
                {#if row.root}
                  <span class="row-meta">{files.filter((file) => file.rootId === row.root?.id).length} files</span>
                  <span
                    class="root-row-menu"
                    role="button"
                    tabindex="0"
                    aria-label={`Folder actions for ${row.root.name}`}
                    onclick={(event) => {
                      event.stopPropagation();
                      if (!row.root) return;
                      showRootMenu(event, row.root);
                    }}
                    onkeydown={(event) => {
                      if (event.key === "Enter" || event.key === " ") {
                        event.preventDefault();
                        event.stopPropagation();
                        if (!row.root) return;
                        const rect = event.currentTarget.getBoundingClientRect();
                        rootMenu = { root: row.root, x: rect.left, y: rect.bottom + 6 };
                      }
                    }}
                  >⋯</span>
                {/if}
              </button>
            {:else}
              <button
                class:selected={row.file.rootId === selectedRootId && row.file.path === selectedPath}
                class="file-row"
                onclick={() => openNote(row.file)}
                oncontextmenu={(event) => showFileMenu(event, row.file)}
                title={row.file.path}
                style={`padding-left: ${10 + row.depth * 14}px`}
              >
                <span class="file-title">
                  <span class="row-icon" aria-hidden="true">📄</span>
                  {row.file.title}
                </span>
                <span class="row-meta">{parentFolderLabel(row.file.path)}</span>
              </button>
            {/if}
          {/each}
          <div style={`height: ${bottomSpacer}px`} aria-hidden="true"></div>
        {/if}
      </nav>
    {/if}
  </aside>

  {#if rootMenu}
    <div
      class="root-context-menu"
      style={`left: ${rootMenu.x}px; top: ${rootMenu.y}px;`}
      onclick={(event) => event.stopPropagation()}
      onkeydown={(event) => {
        if (event.key === "Escape") rootMenu = null;
      }}
      role="menu"
      tabindex="-1"
    >
      <div class="root-context-title">
        <strong>{rootMenu.root.name}</strong>
        <small>{rootMenu.root.path}</small>
      </div>
      <button onclick={() => { explorerQuery = ""; railMode = "library"; rootMenu = null; }}>
        Show in library
      </button>
      <button onclick={() => void navigator.clipboard.writeText(rootMenu?.root.path ?? "")}>
        Copy full path
      </button>
      <button onclick={() => void navigator.clipboard.writeText(rootMenu ? rootRelativePath(rootMenu.root) : "")}>
        Copy relative path
      </button>
      <button class="danger" onclick={() => rootMenu && removeFolder(rootMenu.root)} disabled={isOpening}>
        Remove from Library
      </button>
    </div>
  {/if}

  {#if fileMenu}
    <div
      class="root-context-menu"
      style={`left: ${fileMenu.x}px; top: ${fileMenu.y}px;`}
      onclick={(event) => event.stopPropagation()}
      onkeydown={(event) => {
        if (event.key === "Escape") fileMenu = null;
      }}
      role="menu"
      tabindex="-1"
    >
      <div class="root-context-title">
        <strong>{fileMenu.file.title}</strong>
        <small>{fileFullPath(fileMenu.file)}</small>
      </div>
      <button onclick={() => void navigator.clipboard.writeText(fileRelativePath(fileMenu!.file))}>
        Copy relative path
      </button>
      <button onclick={() => void navigator.clipboard.writeText(fileMenu ? fileFullPath(fileMenu.file) : "")}>
        Copy full path
      </button>
    </div>
  {/if}

  {#if workspaceMenu}
    <div
      class="root-context-menu"
      style={`left: ${workspaceMenu.x}px; top: ${workspaceMenu.y}px;`}
      onclick={(event) => event.stopPropagation()}
      onkeydown={(event) => {
        if (event.key === "Escape") workspaceMenu = null;
      }}
      role="menu"
      tabindex="-1"
    >
      <div class="root-context-title">
        <strong>{workspaceMenu.workspace.name}</strong>
        <small>{workspaceMenu.workspace.rootIds.length} folders</small>
      </div>
      <button onclick={() => switchWorkspace(workspaceMenu?.workspace.id ?? activeWorkspaceId)}>
        Switch to workspace
      </button>
      <button onclick={() => workspaceMenu && startRenameWorkspace(workspaceMenu.workspace)}>
        Rename
      </button>
      <button onclick={() => workspaceMenu && duplicateWorkspace(workspaceMenu.workspace)}>
        Duplicate
      </button>
      <button class="danger" onclick={() => workspaceMenu && deleteWorkspace(workspaceMenu.workspace)}>
        Delete workspace
      </button>
    </div>
  {/if}

  <!-- CENTER DOC READER FRAME -->
  <section class="reader-frame">
    <!-- Clean minimized topbar header -->
    <header class="reader-topbar">
      <div class="document-meta">
        <button class="btn-action icon-only" onclick={toggleSidebar} title="Toggle Left Sidebar">
          <svg style="width:16px;height:16px" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16"></path></svg>
          <kbd class="hud-kbd">⌘B</kbd>
        </button>
      </div>
      {#if openTabs.length > 0 && !focusMode}
        <div class="tab-shell">
          <button class="tab-scroll-btn" onclick={() => scrollTabs(-240)} aria-label="Scroll tabs left">‹</button>
          <nav class="tab-strip" aria-label="Open notes">
            {#each openTabs as tab, index (tab.id)}
              <button
                class="note-tab"
                class:active={tab.id === activeTabId}
                onclick={() => activateTab(tab.id)}
                title={`${tab.file.rootName} / ${tab.file.path}`}
              >
                <span class="tab-index">{index + 1}</span>
                <span class="tab-title">{tab.note.title}</span>
                <span
                  class="tab-close"
                  role="button"
                  tabindex="0"
                  aria-label={`Close ${tab.note.title}`}
                  onclick={(event) => { event.stopPropagation(); closeTab(tab.id); }}
                  onkeydown={(event) => {
                    if (event.key === "Enter" || event.key === " ") {
                      event.preventDefault();
                      event.stopPropagation();
                      closeTab(tab.id);
                    }
                  }}
                >×</span>
              </button>
            {/each}
          </nav>
          <button class="tab-scroll-btn" onclick={() => scrollTabs(240)} aria-label="Scroll tabs right">›</button>
        </div>
      {/if}
    </header>

    <div class="reader-floating-actions" aria-label="Reader controls">
      <button class="reader-floating-pill" class:active={focusMode} onclick={toggleFocusMode} title="Toggle Focus Mode">
        Focus <kbd class="hud-kbd">⌘.</kbd>
      </button>
      <button
        class="reader-floating-pill"
        class:active={!tocCollapsed && currentNote && tocItems.length >= 3}
        onclick={toggleToc}
        disabled={!currentNote || tocItems.length < 3}
        title="Toggle Table of Contents"
      >
        TOC <kbd class="hud-kbd">⌘J</kbd>
      </button>
    </div>

    <!-- Floating Top-Right Find popover overlay widget -->
    {#if findOpen}
      <div class="find-popover" transition:fade={{ duration: 150 }}>
        <input
          id="find-input"
          class="find-input"
          bind:value={findQuery}
          onkeydown={handleFindKeydown}
          placeholder="Find in current note..."
          autocomplete="off"
        />
        <span class="find-count">
          {findQuery.trim().length < 2 ? "0/0" : `${findMatches.length === 0 ? 0 : findActiveIndex + 1}/${findMatches.length}`}
        </span>
        <button class="find-btn" onclick={() => scrollToFindMatch(findActiveIndex - 1)} disabled={findMatches.length === 0}>Prev</button>
        <button class="find-btn" onclick={() => scrollToFindMatch(findActiveIndex + 1)} disabled={findMatches.length === 0}>Next</button>
        <button class="find-btn" onclick={() => { findOpen = false; findQuery = ""; clearFindHighlights(); }}>Close</button>
      </div>
    {/if}

    <!-- Scrollable reader canvas -->
    <div class="reader-scroll" class:is-scrolling={readerScrolling} onscroll={handleReaderScroll}>
      {#if isRendering}
        <div class="loading-card">
          <span class="pulse"></span>
          Rendering note...
        </div>
      {:else if currentNote}
        <article class="reader" data-reader style={readerStyle}>
          {@html currentNote.html}
        </article>
      {:else}
        <div class="hero-empty">
          <p class="eyebrow">Ready to read</p>
          <h2>Choose a Markdown file to begin.</h2>
          <p>
            Add folders from the Library panel, then pick a note from the list. You can also press <kbd>⌘K</kbd> to quickly open files, tabs, headings, settings, and workspaces.
          </p>
          <p class="hero-empty-hint">
            Tip: use the <strong>+</strong> button beside Library when you want to add a folder.
          </p>
        </div>
      {/if}
    </div>

    <!-- Focus mode toast helper -->
    <div class="focus-toast" class:show={focusToastVisible}>
      Focus Mode active: press <kbd>/</kbd> to search headings &bull; <kbd>⌘.</kbd> to exit
    </div>

    <!-- Focus mode escape hint label -->
    <div class="focus-exit-hint" class:show={focusExitHintVisible}>
      Press <kbd>⌘.</kbd> to exit focus mode · <kbd>/</kbd> to search headings
    </div>

    <!-- Floating TOC card. In focus mode it appears only when / opens search. -->
    {#if shouldShowToc}
      <aside class="toc-rail" class:focus-toc={focusMode} class:search-open={headingPaletteOpen}>
        <div class="toc-title">
          <span>On This Page</span>
          <button class="toc-close" onclick={hideToc} aria-label="Hide table of contents">×</button>
        </div>
      <input
        id="toc-search"
        class="toc-search"
        bind:value={tocQuery}
        onkeydown={handleTocSearchKeydown}
        placeholder="Search headings... (/)"
        autocomplete="off"
      />
      <nav class="toc-list">
        {#each filteredTocItems as item, index (item.id)}
          <button
            class:active={item.id === activeHeadingId}
            class:selected={headingPaletteOpen && index === headingPaletteActiveIndex}
            class={`toc-item toc-level-${item.level}`}
            onclick={() => { jumpToHeading(item.id); closeTocSearch(); }}
          >
            {item.text}
          </button>
        {:else}
          <p class="toc-empty">No heading match</p>
        {/each}
      </nav>
      </aside>
    {/if}
  </section>
</main>

<!-- SETTINGS STUDIO -->
<div class="settings-drawer" class:open={settingsOpen} transition:fade={{ duration: 150 }}>
  <div class="settings-header">
    <div>
      <p class="settings-kicker">Settings Studio</p>
      <h3>Customize reader</h3>
    </div>
    <button class="settings-close" onclick={() => settingsOpen = false}>&times;</button>
  </div>

  <div class="settings-studio">
    <nav class="settings-section-nav" aria-label="Settings sections">
      {#each [
        ["general", "General"],
        ["shortcuts", "Shortcuts"],
        ["files", "Files"],
        ["appearance", "Appearance"],
        ["toc", "TOC"],
        ["markdown", "Markdown"],
        ["advanced", "Advanced"],
      ] as section}
        <button
          class:active={settingsSection === section[0]}
          onclick={() => settingsSection = section[0] as SettingsSection}
        >
          {section[1]}
        </button>
      {/each}
    </nav>

    <div class="settings-panel-content">
      {#if settingsSection === "appearance"}
        <div class="appearance-studio">
          <div class="appearance-controls">
            <div class="settings-section">
              <div class="settings-section-title">Theme Presets</div>
              <div class="preset-browser">
                <div class="preset-list compact">
                  {#each colorPresets as preset (preset.id)}
                    <button
                      class:active={selectedPresetKey === `system:${preset.id}`}
                      onclick={() => applySystemThemePreset(preset)}
                    >
                      <span class="preset-dot" style={`background: ${preset.values.reader}`}></span>
                      <span>{preset.name}</span>
                      <small>System</small>
                    </button>
                  {/each}
                  {#each customThemePresets as preset (preset.id)}
                    <button
                      class:active={selectedPresetKey === `custom:${preset.id}`}
                      onclick={() => applyCustomThemePreset(preset)}
                    >
                      <span class="preset-dot custom-dot"></span>
                      <span>{preset.name}</span>
                      <small>Custom</small>
                    </button>
                  {/each}
                </div>
                <div class="preset-actions">
                  {#if selectedPresetKey.startsWith("system:")}
                    <button class="btn-action" onclick={() => duplicateSystemThemePreset(colorPresets.find((preset) => `system:${preset.id}` === selectedPresetKey) ?? colorPresets[0])}>
                      Duplicate system preset
                    </button>
                  {:else}
                    <button class="btn-action" onclick={updateSelectedCustomPreset}>Update custom preset</button>
                  {/if}
                  <button class="btn-action" onclick={saveCurrentAsPreset}>Save current as new</button>
                  <button class="btn-action" onclick={copySelectedPresetJson}>Copy preset JSON</button>
                </div>
                <textarea readonly value={selectedPresetJson()} class="presets-textarea compact-json" aria-label="Selected preset JSON"></textarea>
              </div>
            </div>

            <div class="settings-section">
              <div class="settings-section-title">Color Theme</div>
              <div class="theme-grid">
                {#each colorPresets as preset (preset.id)}
                  <button
                    class="theme-bubble"
                    class:active={preset.id === colorPresetId}
                    onclick={() => applySystemThemePreset(preset)}
                  >
                    <span style={`background: ${preset.values.reader}`}></span>
                    {preset.name}
                  </button>
                {/each}
              </div>
            </div>

            <div class="settings-section">
              <div class="settings-section-title">Typography Presets</div>
              <div class="typography-grid">
                {#each readerPresets as preset (preset.id)}
                  <button class="btn-action" class:active={preset.id === readerPresetId} onclick={() => applyReaderPreset(preset)}>
                    {preset.name}
                  </button>
                {/each}
              </div>
            </div>

            <div class="settings-section">
              <div class="settings-section-title">Manual Adjustments</div>
              <div class="slider-group">
                <div class="slider-item">
                  <div class="slider-item-header">
                    <span>Body Font Size</span>
                    <b>{readerFontSize}px</b>
                  </div>
                  <input type="range" min="14" max="24" bind:value={readerFontSize} oninput={markReaderCustom} />
                </div>
                <div class="slider-item">
                  <div class="slider-item-header">
                    <span>Line Height</span>
                    <b>{readerLineHeight}</b>
                  </div>
                  <input type="range" min="1.3" max="2.0" step="0.05" bind:value={readerLineHeight} oninput={markReaderCustom} />
                </div>
                <div class="slider-item">
                  <div class="slider-item-header">
                    <span>Reader Width</span>
                    <b>{readerMeasure}ch</b>
                  </div>
                  <input type="range" min="50" max="85" bind:value={readerMeasure} oninput={markReaderCustom} />
                </div>
                <div class="slider-item">
                  <div class="slider-item-header">
                    <span>Paragraph Gap</span>
                    <b>{readerParagraphSpacing}em</b>
                  </div>
                  <input type="range" min="0.7" max="1.5" step="0.02" bind:value={readerParagraphSpacing} oninput={markReaderCustom} />
                </div>
              </div>
            </div>
          </div>

          <aside class="appearance-preview-pane" aria-label="Live appearance preview">
            <div class="appearance-preview-sticky">
              <div class="appearance-preview-label">Live Preview</div>
              <article class="appearance-preview-card" style={readerStyle}>
                <p class="appearance-preview-meta">Current preset: {colorPresets.find((preset) => preset.id === colorPresetId)?.name ?? "Custom"}</p>
                <h1>Markdown Preview</h1>
                <p>
                  Every setting updates this pane live. Use it to judge the reading feel before saving a custom preset.
                </p>
                <blockquote>
                  A preset is a saved reading environment: color, type, spacing, and rhythm.
                </blockquote>
                <pre><code>const preset = saveReaderPreset("Warm Editorial");
reader.apply(preset);</code></pre>
              </article>
            </div>
          </aside>
        </div>
      {:else if settingsSection === "files"}
        <div class="settings-section">
          <div class="settings-section-title">Workspace Folders</div>
          <p class="settings-help">These folders are saved as one workspace. Opening one Markdown file from Finder now adds that file folder only if missing. It does not wipe this list.</p>
          <div class="settings-root-list">
            {#each roots as root (root.id)}
              <div class="settings-root-row">
                <span>
                  <strong>{root.name}</strong>
                  <small>{root.path}</small>
                </span>
                <button class="btn-action danger" onclick={() => removeFolder(root)} disabled={isOpening}>Remove</button>
              </div>
            {:else}
              <p class="settings-help">No folders added yet.</p>
            {/each}
          </div>
          <div class="settings-inline-actions">
            <button class="btn-sidebar-footer-primary" onclick={chooseFolder} disabled={isOpening}>Open Folder(s)</button>
            <button class="btn-action" onclick={addFolder} disabled={isOpening}>Add Folder</button>
          </div>
        </div>
      {:else if settingsSection === "shortcuts"}
        <div class="settings-section">
          <div class="settings-section-title">Keyboard</div>
          <div class="settings-shortcut-list">
            <span>Command palette <kbd>⌘K</kbd></span>
            <span>Find in document <kbd>⌘F</kbd></span>
            <span>Filter file tree <kbd>⇧⌘F</kbd></span>
            <span>Focus mode <kbd>⌘.</kbd></span>
            <span>Table of contents <kbd>⌘J</kbd></span>
            <span>Tabs <kbd>⌘[</kbd> <kbd>⌘]</kbd> <kbd>⌘W</kbd></span>
            <span>Scroll reader <kbd>J</kbd> <kbd>K</kbd> <kbd>↑</kbd> <kbd>↓</kbd></span>
          </div>
        </div>
      {:else if settingsSection === "general"}
        <div class="settings-section">
          <div class="settings-section-title">General</div>
          <p class="settings-help">Core app options will live here. Current active controls are folder workspace, keyboard help, tabs, focus mode, and reader appearance.</p>
          <button class="btn-action" onclick={() => { settingsOpen = false; startTour(); }}>Restart Tour Guide</button>
        </div>
      {:else if settingsSection === "toc"}
        <div class="settings-section">
          <div class="settings-section-title">Table of Contents</div>
          <p class="settings-help">TOC is now a floating glass panel. In focus mode, press <kbd>/</kbd> to search headings without blocking the document.</p>
        </div>
      {:else if settingsSection === "markdown"}
        <div class="settings-section">
          <div class="settings-section-title">Markdown Rendering</div>
          <p class="settings-help">Markdown is rendered to safe HTML by the Rust backend, then displayed in the reader. More block-level rendering settings can be added here.</p>
        </div>
      {:else}
        <div class="settings-section">
          <div class="settings-section-title">Advanced</div>
          <p class="settings-help">Import/export raw preset config.</p>
          <textarea bind:value={presetsJsonText} class="presets-textarea"></textarea>
          <div class="settings-inline-actions">
            <button class="btn-sidebar-footer-primary" onclick={applyPresetsConfig}>Apply JSON</button>
            <button class="btn-action" onclick={copyPresetsConfig}>Copy Config</button>
          </div>
        </div>
      {/if}
    </div>
  </div>
</div>

<!-- KEYBOARD HUD OVERLAY modal -->
{#if hudOpen}
  <div class="hud-backdrop" onclick={() => hudOpen = false}>
    <div class="hud-panel" onclick={(e) => e.stopPropagation()}>
      <h3 class="hud-title">Keyboard Shortcuts</h3>
      <div class="hud-list">
        <div class="hud-row">
          <span class="hud-desc">Open Command Palette</span>
          <div class="hud-keys"><kbd class="hud-key">⌘</kbd><kbd class="hud-key">K</kbd></div>
        </div>
        <div class="hud-row">
          <span class="hud-desc">Toggle Left Sidebar</span>
          <div class="hud-keys"><kbd class="hud-key">⌘</kbd><kbd class="hud-key">B</kbd></div>
        </div>
        <div class="hud-row">
          <span class="hud-desc">Toggle Table of Contents</span>
          <div class="hud-keys"><kbd class="hud-key">⌘</kbd><kbd class="hud-key">J</kbd></div>
        </div>
        <div class="hud-row">
          <span class="hud-desc">Toggle Focus Reading Mode</span>
          <div class="hud-keys"><kbd class="hud-key">⌘</kbd><kbd class="hud-key">.</kbd></div>
        </div>
        <div class="hud-row">
          <span class="hud-desc">Move Between Tabs</span>
          <div class="hud-keys"><kbd class="hud-key">⌘</kbd><kbd class="hud-key">[</kbd><kbd class="hud-key">⌘</kbd><kbd class="hud-key">]</kbd></div>
        </div>
        <div class="hud-row">
          <span class="hud-desc">Close Current Tab</span>
          <div class="hud-keys"><kbd class="hud-key">⌘</kbd><kbd class="hud-key">W</kbd></div>
        </div>
        <div class="hud-row">
          <span class="hud-desc">Find in Current Note</span>
          <div class="hud-keys"><kbd class="hud-key">⌘</kbd><kbd class="hud-key">F</kbd></div>
        </div>
        <div class="hud-row">
          <span class="hud-desc">Search Headings List (Focus Mode)</span>
          <div class="hud-keys"><kbd class="hud-key">/</kbd></div>
        </div>
        <div class="hud-row">
          <span class="hud-desc">Scroll Article</span>
          <div class="hud-keys"><kbd class="hud-key">↑</kbd><kbd class="hud-key">↓</kbd><kbd class="hud-key">J</kbd><kbd class="hud-key">K</kbd></div>
        </div>
        <div class="hud-row">
          <span class="hud-desc">Search Headings Popover (Any Mode)</span>
          <div class="hud-keys"><kbd class="hud-key">⌘</kbd><kbd class="hud-key">⇧</kbd><kbd class="hud-key">O</kbd></div>
        </div>
        <div class="hud-row">
          <span class="hud-desc">Open Reader Style Settings</span>
          <div class="hud-keys"><kbd class="hud-key">⌘</kbd><kbd class="hud-key">,</kbd></div>
        </div>
        <div class="hud-row" style="margin-top: 10px; padding-top: 12px; border-top: 1px solid var(--line);">
          <span class="hud-desc">Toggle Shortcut HUD help</span>
          <div class="hud-keys"><kbd class="hud-key">?</kbd></div>
        </div>
        <div class="hud-row">
          <span class="hud-desc">Close Modal Overlay</span>
          <div class="hud-keys"><kbd class="hud-key">Esc</kbd></div>
        </div>
      </div>
      
      <div style="margin-top: 16px; display: flex; flex-direction: column; gap: 8px;">
        <button class="btn-sidebar-footer-primary" onclick={() => { hudOpen = false; startTour(); }} style="width: 100%; justify-content: center; font-weight: 700; background: var(--accent-strong); color: #fff;">
          Restart Walkthrough Guide
        </button>
        <button class="btn-action" style="width: 100%; justify-content: center;" onclick={() => hudOpen = false}>Close</button>
      </div>
    </div>
  </div>
{/if}

<!-- UNIFIED COMMAND PALETTE overlay -->
{#if paletteOpen}
  <div class="palette-backdrop" onclick={() => paletteOpen = false}>
    <div
      class="palette"
      role="dialog"
      aria-modal="true"
      onkeydown={handlePaletteDialogKeydown}
      onclick={(event) => event.stopPropagation()}
    >
      <input
        id="palette-input"
        bind:value={paletteQuery}
        onkeydown={handlePaletteKeydown}
        placeholder={palettePlaceholder}
        autocomplete="off"
      />
      <div class="palette-meta">
        {selectablePaletteRows.length} result{selectablePaletteRows.length === 1 ? "" : "s"}
        · Prefixes: <kbd>&gt;</kbd> actions <kbd>/</kbd> files <kbd>@</kbd> tabs <kbd>#</kbd> headings <kbd>?</kbd> settings <kbd>:</kbd> workspaces
      </div>
      <div class="palette-results" bind:this={paletteResultsElement}>
        {#each paletteGroups as group (group.id)}
          <div class="palette-group">
            <div class="palette-group-title">
              <span>{group.label}</span>
              <small>{group.rows.length}</small>
            </div>
            {#each group.rows as row}
              <button
                class="palette-row"
                class:active={paletteRowIndex(row) === paletteActiveIndex}
                data-palette-index={paletteRowIndex(row)}
                onclick={() => runPaletteRow(row)}
              >
                {#if row.type === "command"}
                  <div class="cmd-palette-item-content">
                    <div class="palette-row-copy">
                      <span class="palette-row-type">Action</span>
                      <span class="palette-row-title">{row.command.text}</span>
                    </div>
                    {#if row.command.shortcut.length > 0}
                      <div class="cmd-badges">
                        {#each row.command.shortcut as key}
                          <kbd class="cmd-badge">{key}</kbd>
                        {/each}
                      </div>
                    {/if}
                  </div>
                {:else if row.type === "tab"}
                  <div class="palette-row-copy">
                    <span class="palette-row-type">Open tab</span>
                    <span class="palette-row-title">{row.tab.note.title}</span>
                    <small>{row.tab.file.rootName} / {row.tab.file.path}</small>
                  </div>
                {:else if row.type === "file"}
                  <div class="palette-row-copy">
                    <span class="palette-row-type">File</span>
                    <span class="palette-row-title">{row.file.title}</span>
                    <small>{row.file.rootName} / {row.file.path}</small>
                  </div>
                {:else if row.type === "heading"}
                  <div class="palette-row-copy">
                    <span class="palette-row-type">Heading</span>
                    <span class="palette-row-title">{row.heading.text}</span>
                    <small>Level {row.heading.level} in current note</small>
                  </div>
                {:else if row.type === "setting"}
                  <div class="palette-row-copy">
                    <span class="palette-row-type">Setting</span>
                    <span class="palette-row-title">{row.setting.title}</span>
                    <small>Open {row.setting.section} settings</small>
                  </div>
                {:else if row.type === "workspace"}
                  <div class="palette-row-copy">
                    <span class="palette-row-type">Workspace</span>
                    <span class="palette-row-title">{row.workspace.name}</span>
                    <small>{row.workspace.rootIds.length} folder{row.workspace.rootIds.length === 1 ? "" : "s"}</small>
                  </div>
                {/if}
              </button>
            {/each}
          </div>
        {:else}
          <p class="palette-empty">No result. Try another prefix.</p>
        {/each}
      </div>
    </div>
  </div>
{/if}

<!-- ONBOARDING TOUR OVERLAYS -->
{#if tourOpen}
  <div class="tour-backdrop open" onclick={exitTour}>
    <div class="tour-highlight" style={highlightStyle}></div>
    <div class="tour-card" style={cardStyle} onclick={(e) => e.stopPropagation()}>
      <button class="tour-close" onclick={exitTour} aria-label="Quit walkthrough">&times;</button>
      <h4>{tourSteps[tourStep]?.title}</h4>
      <p>{tourSteps[tourStep]?.text}</p>
      <div class="tour-actions">
        <span>{tourStep + 1} / {tourSteps.length}</span>
        <button class="btn-action btn-skip" onclick={exitTour}>Skip Guide</button>
        <div style="display: flex; gap: 6px;">
          {#if tourStep > 0}
            <button class="btn-action" style="padding: 4px 8px; font-size: 11px;" onclick={prevTourStep}>Prev</button>
          {/if}
          <button class="btn-action" style="padding: 4px 8px; font-size: 11px; background: var(--accent-strong); color: #fff;" onclick={nextTourStep}>
            {tourStep === tourSteps.length - 1 ? 'Finish' : 'Next'}
          </button>
        </div>
      </div>
    </div>
  </div>
{/if}

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
    font-family: Outfit, ui-sans-serif, system-ui, sans-serif;
    color: var(--text);
    background: var(--canvas);
    text-rendering: optimizeLegibility;
    -webkit-font-smoothing: antialiased;
  }

  /* Style typography details */
  @font-face {
    font-family: 'Newsreader';
    src: local('Newsreader'), local('Newsreader-Regular');
    font-style: normal;
  }
  @font-face {
    font-family: 'Outfit';
    src: local('Outfit'), local('Outfit-Regular');
    font-style: normal;
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
    grid-template-columns: 56px 300px minmax(0, 1fr);
    height: 100vh;
    transition: grid-template-columns 0.3s cubic-bezier(0.25, 0.8, 0.25, 1);
    --sidebar-width: 300px;
    --toc-width: 260px;
  }

  .app-shell.sidebar-collapsed {
    grid-template-columns: 56px 0px minmax(0, 1fr);
  }

  .app-shell.focus-mode {
    grid-template-columns: 0px 0px minmax(0, 1fr);
  }

  .rail {
    grid-column: 1;
    height: 100vh;
    min-height: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 10px;
    padding: 14px 8px;
    border-right: 1px solid var(--line);
    background: color-mix(in srgb, var(--panel-strong) 44%, transparent);
    backdrop-filter: blur(18px);
    overflow: hidden;
    transition: opacity 0.2s ease;
  }

  .focus-mode .rail {
    opacity: 0;
    pointer-events: none;
  }

  .rail-brand {
    width: 34px;
    height: 34px;
    display: grid;
    place-items: center;
    margin-bottom: 10px;
    border-radius: 12px;
    color: var(--reader);
    background: var(--text);
    font-family: Fraunces, Newsreader, serif;
    font-size: 16px;
    font-weight: 800;
  }

  .rail-button {
    width: 36px;
    height: 36px;
    display: grid;
    place-items: center;
    border: 0;
    border-radius: 13px;
    color: var(--muted);
    background: transparent;
    cursor: pointer;
    font: inherit;
    font-size: 16px;
    transition: 140ms ease;
  }

  .rail-button:hover,
  .rail-button.active {
    color: var(--accent-strong);
    background: var(--accent-soft);
    transform: translateY(-1px);
  }

  .rail-button span {
    position: relative;
    display: inline-grid;
    place-items: center;
    width: 18px;
    height: 18px;
    font-size: 15px;
    line-height: 1;
  }

  .rail-icon-library span::before {
    content: "";
    position: absolute;
    top: 1px;
    left: 1px;
    width: 11px;
    height: 5px;
    border-radius: 3px 3px 1px 1px;
    background: currentColor;
    opacity: 0.55;
  }

  .rail-icon-workspaces span::before,
  .rail-icon-workspaces span::after {
    content: "";
    position: absolute;
    width: 10px;
    height: 10px;
    border: 1.5px solid currentColor;
    border-radius: 4px;
  }

  .rail-icon-workspaces span::before {
    top: 1px;
    left: 1px;
    opacity: 0.55;
  }

  .rail-icon-workspaces span::after {
    right: 1px;
    bottom: 1px;
    background: color-mix(in srgb, var(--panel-strong) 55%, transparent);
  }

  .rail-bottom {
    margin-top: auto;
  }

  .sidebar {
    grid-column: 2;
    height: 100vh;
    min-height: 0;
    display: flex;
    flex-direction: column;
    gap: 14px;
    padding: 18px 18px 0;
    border-right: 1px solid var(--line);
    background: var(--panel);
    transition: opacity 0.25s ease;
    overflow: hidden;
  }

  .sidebar-collapsed .sidebar {
    opacity: 0;
    pointer-events: none;
  }

  .brand {
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

  .icon-button {
    padding: 8px 10px;
    color: var(--muted);
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

  .workspace-manager {
    min-height: 0;
    display: flex;
    flex: 1;
    flex-direction: column;
    gap: 12px;
  }

  .workspace-hero {
    display: grid;
    gap: 4px;
    padding: 16px;
    border: 1px solid var(--line);
    border-radius: 20px;
    background:
      radial-gradient(circle at 92% 0%, color-mix(in srgb, var(--accent) 24%, transparent), transparent 36%),
      color-mix(in srgb, var(--panel-strong) 72%, transparent);
  }

  .workspace-hero span {
    color: var(--muted);
    font-size: 11px;
    font-weight: 800;
    letter-spacing: 0.1em;
    text-transform: uppercase;
  }

  .workspace-hero strong {
    overflow: hidden;
    color: var(--text);
    font-size: 18px;
    letter-spacing: -0.03em;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .workspace-hero small {
    color: var(--muted);
    font-size: 12px;
  }

  .workspace-actions {
    display: grid;
    grid-template-columns: 1fr auto;
    gap: 8px;
  }

  .workspace-create-form {
    display: grid;
    gap: 10px;
    padding: 12px;
    border: 1px solid color-mix(in srgb, var(--accent) 42%, var(--line));
    border-radius: 18px;
    background: color-mix(in srgb, var(--accent-soft) 64%, var(--panel));
  }

  .workspace-create-form label {
    display: grid;
    gap: 7px;
  }

  .workspace-create-form label span {
    color: var(--muted);
    font-size: 10px;
    font-weight: 800;
    letter-spacing: 0.09em;
    text-transform: uppercase;
  }

  .workspace-create-form input {
    width: 100%;
    border: 1px solid var(--line);
    border-radius: 12px;
    padding: 10px 11px;
    color: var(--text);
    background: var(--panel-strong);
    font: inherit;
    font-size: 13px;
    outline: none;
  }

  .workspace-create-form input:focus {
    border-color: color-mix(in srgb, var(--accent) 72%, var(--line));
    box-shadow: 0 0 0 3px var(--accent-soft);
  }

  .workspace-create-actions {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 8px;
  }

  .workspace-rename-form {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto auto;
    gap: 6px;
    padding: 8px;
    border: 1px solid color-mix(in srgb, var(--accent) 45%, var(--line));
    border-radius: 16px;
    background: var(--panel-strong);
  }

  .workspace-rename-form input {
    min-width: 0;
    border: 1px solid var(--line);
    border-radius: 10px;
    padding: 8px 9px;
    color: var(--text);
    background: var(--panel);
    font: inherit;
    font-size: 12px;
    outline: none;
  }

  .workspace-rename-form input:focus {
    border-color: var(--accent);
    box-shadow: 0 0 0 3px var(--accent-soft);
  }

  .workspace-rename-form button {
    border: 1px solid var(--line);
    border-radius: 10px;
    padding: 8px 9px;
    color: var(--muted);
    background: var(--panel);
    font: inherit;
    font-size: 11px;
    font-weight: 800;
    cursor: pointer;
  }

  .workspace-rename-form button:hover,
  .workspace-rename-form button:focus-visible {
    color: var(--accent-strong);
    border-color: var(--accent);
  }

  .workspace-card-list {
    min-height: 0;
    display: grid;
    align-content: start;
    gap: 8px;
    overflow: auto;
  }

  .workspace-card {
    width: 100%;
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto auto;
    align-items: center;
    gap: 10px;
    padding: 12px;
    border: 1px solid var(--line);
    border-radius: 16px;
    color: var(--text);
    background: color-mix(in srgb, var(--panel-strong) 70%, transparent);
    cursor: default;
    font: inherit;
    text-align: left;
  }

  .workspace-card:hover {
    border-color: color-mix(in srgb, var(--accent) 48%, var(--line));
    background: var(--accent-soft);
  }

  .workspace-card.active {
    border-color: color-mix(in srgb, var(--accent) 72%, var(--line));
    background:
      linear-gradient(135deg, color-mix(in srgb, var(--accent) 18%, transparent), transparent 62%),
      color-mix(in srgb, var(--panel-strong) 82%, transparent);
    box-shadow: inset 3px 0 0 var(--accent);
  }

  .workspace-card span:first-child {
    min-width: 0;
    display: grid;
    gap: 3px;
  }

  .workspace-card strong {
    overflow: hidden;
    font-size: 13px;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .workspace-card small {
    overflow: hidden;
    color: var(--muted);
    font-size: 10px;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .workspace-card-count {
    min-width: 26px;
    padding: 4px 7px;
    border-radius: 999px;
    color: var(--accent-strong);
    background: var(--accent-soft);
    font-size: 11px;
    font-weight: 800;
    text-align: center;
  }

  .workspace-card-menu {
    width: 28px;
    height: 28px;
    display: grid;
    place-items: center;
    border-radius: 999px;
    color: var(--muted);
    cursor: pointer;
  }

  .workspace-card-menu:hover {
    color: var(--text);
    background: var(--panel);
  }

  .root-context-menu {
    position: fixed;
    z-index: 700;
    width: 220px;
    padding: 8px;
    border: 1px solid var(--line);
    border-radius: 16px;
    background: color-mix(in srgb, var(--panel-strong) 94%, transparent);
    box-shadow: var(--shadow);
    backdrop-filter: blur(18px);
  }

  .root-context-title {
    display: grid;
    gap: 2px;
    padding: 8px 9px 10px;
    border-bottom: 1px solid var(--line);
    margin-bottom: 6px;
  }

  .root-context-title strong,
  .root-context-title small {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .root-context-title strong {
    font-size: 12px;
  }

  .root-context-title small {
    color: var(--muted);
    font-size: 10px;
  }

  .root-context-menu button {
    width: 100%;
    display: flex;
    border: 0;
    border-radius: 10px;
    padding: 9px;
    color: var(--text);
    background: transparent;
    cursor: pointer;
    font: inherit;
    font-size: 12px;
    font-weight: 750;
    text-align: left;
  }

  .root-context-menu button:hover {
    background: var(--accent-soft);
  }

  .root-context-menu button.danger {
    color: #d15a4f;
  }

  .workspace-roots-title {
    display: flex;
    align-items: center;
    justify-content: space-between;
    color: var(--muted);
    font-size: 11px;
    font-weight: 800;
    letter-spacing: 0.08em;
    text-transform: uppercase;
  }

  .root-add-button {
    width: 24px;
    height: 24px;
    border: 1px solid var(--line);
    border-radius: 999px;
    color: var(--text);
    background: var(--panel);
    cursor: pointer;
  }

  .empty-sidebar.compact {
    padding: 10px;
    border-radius: 12px;
    font-size: 12px;
  }

  .settings-root-row span {
    min-width: 0;
    display: grid;
    gap: 2px;
  }

  .settings-root-row strong {
    overflow: hidden;
    color: var(--text);
    font-size: 12px;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .settings-root-row small {
    overflow: hidden;
    color: var(--muted);
    font-size: 10px;
    text-overflow: ellipsis;
    white-space: nowrap;
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
    padding: 10px 14px;
    border: 1px solid var(--line);
    border-radius: 13px;
    color: var(--text);
    background: var(--panel-strong);
    font: inherit;
    outline: none;
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
    flex: 1;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    padding-right: 4px;
    margin-bottom: 12px;
  }

  .file-row {
    display: flex;
    align-items: center;
    gap: 6px;
    width: 100%;
    height: 32px;
    min-height: 32px;
    padding: 4px 6px;
    border: 0;
    border-radius: 8px;
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

  .root-folder-row {
    color: var(--text);
    background: color-mix(in srgb, var(--panel-strong) 48%, transparent);
  }

  .root-row-menu {
    margin-left: auto;
    display: inline-flex;
    width: 18px;
    height: 18px;
    align-items: center;
    justify-content: center;
    border-radius: 999px;
    color: var(--muted);
    font-size: 12px;
  }

  .root-row-menu:hover,
  .root-row-menu:focus-visible {
    color: var(--text);
    background: var(--accent-soft);
    outline: none;
  }

  .chevron {
    display: inline-block;
    width: 10px;
    color: var(--faint);
  }

  .file-title,
  .row-meta {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .file-title {
    flex: 1 1 auto;
    min-width: 0;
    display: inline-flex;
    align-items: center;
    gap: 5px;
    font-size: 12px;
    font-weight: 600;
  }

  .row-icon {
    flex: 0 0 auto;
    font-size: 12px;
    line-height: 1;
  }

  .row-meta {
    flex: 0 0 auto;
    color: var(--muted);
    font-size: 10px;
  }

  .empty-sidebar {
    color: var(--muted);
    padding: 14px;
    line-height: 1.5;
    border: 1px dashed var(--line);
    border-radius: 16px;
  }

  /* Sticky sidebar footer */
  .sidebar-footer {
    border-top: 1px solid var(--line);
    padding: 16px 0;
    background: var(--panel);
    display: flex;
    flex-direction: column;
    gap: 8px;
    margin-top: auto;
  }

  .btn-sidebar-footer-primary {
    background: var(--text);
    color: var(--reader);
    border: 1px solid var(--line);
    border-radius: 8px;
    padding: 8px 12px;
    cursor: pointer;
    font-size: 12px;
    font-weight: 700;
    text-align: center;
    width: 100%;
    transition: opacity 0.15s ease;
    display: block;
  }

  .btn-sidebar-footer-primary:hover {
    opacity: 0.9;
  }

  .btn-sidebar-footer-primary:disabled {
    opacity: 0.45;
    cursor: not-allowed;
  }

  .btn-sidebar-footer-ghost {
    width: 100%;
    padding: 8px 12px;
    border: 1px solid var(--line);
    border-radius: 8px;
    color: var(--muted);
    background: var(--panel-strong);
    cursor: pointer;
    font-size: 12px;
    font-weight: 700;
    text-align: center;
  }

  .btn-sidebar-footer-ghost:hover {
    color: var(--text);
    border-color: color-mix(in srgb, var(--accent) 36%, var(--line));
  }

  .btn-sidebar-footer-secondary {
    background: var(--panel-strong);
    color: var(--muted);
    border: 1px solid var(--line);
    border-radius: 8px;
    padding: 6px 10px;
    cursor: pointer;
    font-size: 11px;
    font-weight: 600;
    display: flex;
    justify-content: space-between;
    align-items: center;
    transition: background 0.15s ease, color 0.15s ease, border-color 0.15s ease;
  }

  .btn-sidebar-footer-secondary:hover {
    background: var(--accent-soft);
    color: var(--accent-strong);
    border-color: var(--accent);
  }

  .sidebar-footer-row {
    display: flex;
    gap: 8px;
    width: 100%;
  }

  /* Reader frame */
  .reader-frame {
    grid-column: 3;
    height: 100vh;
    min-width: 0;
    min-height: 0;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    background: var(--reader);
    position: relative;
  }

  .reader-topbar {
    height: 60px;
    border-bottom: 1px solid var(--line);
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 0 24px;
    background: var(--reader);
    z-index: 10;
  }

  .focus-mode .reader-topbar {
    height: 0;
    padding-block: 0;
    border-bottom: 0;
    opacity: 0;
    pointer-events: none;
    overflow: hidden;
  }

  .document-meta {
    flex: 0 0 auto;
    display: flex;
    align-items: center;
  }

  .top-bar-actions {
    flex: 0 0 auto;
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .reader-floating-actions {
    position: absolute;
    right: 24px;
    bottom: 24px;
    z-index: 95;
    display: inline-flex;
    align-items: center;
    gap: 8px;
    padding: 6px;
    border: 1px solid color-mix(in srgb, var(--line) 70%, transparent);
    border-radius: 999px;
    background: color-mix(in srgb, var(--reader) 42%, transparent);
    backdrop-filter: blur(18px) saturate(1.25);
    box-shadow:
      0 18px 52px rgba(0, 0, 0, 0.12),
      inset 0 1px 0 color-mix(in srgb, var(--panel-strong) 62%, transparent);
    opacity: 0.18;
    transition:
      opacity 0.18s ease,
      transform 0.18s ease,
      background 0.18s ease;
  }

  .reader-floating-actions:hover,
  .reader-floating-actions:focus-within,
  .focus-mode .reader-floating-actions {
    opacity: 1;
    transform: translateY(-1px);
  }

  .reader-floating-pill {
    min-height: 32px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    padding: 7px 11px;
    border: 1px solid transparent;
    border-radius: 999px;
    color: color-mix(in srgb, var(--text) 82%, var(--muted));
    background: color-mix(in srgb, var(--panel-strong) 54%, transparent);
    font: inherit;
    font-size: 12px;
    font-weight: 800;
    letter-spacing: -0.01em;
    cursor: pointer;
    box-shadow: inset 0 1px 0 color-mix(in srgb, white 18%, transparent);
    transition:
      color 0.16s ease,
      background 0.16s ease,
      border-color 0.16s ease,
      transform 0.16s ease;
  }

  .reader-floating-pill:hover,
  .reader-floating-pill:focus-visible,
  .reader-floating-pill.active {
    color: var(--text);
    background: color-mix(in srgb, var(--accent-soft) 72%, var(--panel-strong));
    border-color: color-mix(in srgb, var(--accent) 42%, transparent);
    transform: translateY(-1px);
  }

  .reader-floating-pill:disabled {
    cursor: not-allowed;
    color: color-mix(in srgb, var(--muted) 68%, transparent);
    background: color-mix(in srgb, var(--panel) 36%, transparent);
    border-color: transparent;
    transform: none;
  }

  .reader-floating-pill kbd {
    border: 1px solid color-mix(in srgb, var(--line) 80%, transparent);
    border-radius: 6px;
    padding: 1px 5px;
    background: color-mix(in srgb, var(--reader) 58%, transparent);
    color: var(--muted);
    font-size: 10px;
    font-weight: 800;
  }

  .aa-pill {
    min-width: 42px;
    font-family: var(--reader-heading-font);
    font-size: 15px;
    letter-spacing: -0.04em;
  }

  .btn-action {
    background: var(--panel-strong);
    border: 1px solid var(--line);
    border-radius: 8px;
    padding: 8px 12px;
    color: var(--muted);
    cursor: pointer;
    font-size: 12px;
    font-weight: 600;
    display: inline-flex;
    align-items: center;
    gap: 6px;
    transition: background 0.15s ease, color 0.15s ease;
  }

  .btn-action:hover, .btn-action.active {
    background: var(--accent-soft);
    color: var(--accent-strong);
    border-color: var(--accent);
  }

  .btn-action.icon-only {
    padding: 8px;
  }

  .hud-kbd {
    font-size: 9px;
    background: var(--panel);
    border: 1px solid var(--line);
    border-radius: 4px;
    padding: 1px 3px;
    color: var(--muted);
  }

  /* Option 1 Find popover style */
  .find-popover {
    position: absolute;
    top: 70px;
    right: 24px;
    background: var(--panel-strong);
    border: 1px solid var(--accent);
    border-radius: 12px;
    padding: 10px 14px;
    display: flex;
    align-items: center;
    gap: 8px;
    box-shadow: var(--shadow);
    z-index: 120;
    width: 440px;
    max-width: calc(100% - 48px);
  }

  .find-input {
    flex: 1;
    padding: 6px 10px;
    border: 1px solid var(--line);
    border-radius: 8px;
    color: var(--text);
    background: var(--panel);
    outline: none;
    font-size: 13px;
  }

  .find-count {
    font-size: 11px;
    color: var(--muted);
    font-weight: 700;
    min-width: 40px;
    text-align: center;
  }

  .find-btn {
    background: var(--panel-strong);
    border: 1px solid var(--line);
    border-radius: 6px;
    padding: 6px 10px;
    font-size: 11px;
    color: var(--muted);
    cursor: pointer;
    font-weight: 600;
  }

  .find-btn:hover {
    background: var(--accent-soft);
    color: var(--accent-strong);
  }

  .find-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .reader-scroll {
    min-height: 0;
    height: 100%;
    overflow-y: auto;
    overflow-x: hidden;
    padding: 52px 32px 120px; /* comfy bottom spacing */
    -webkit-overflow-scrolling: touch;
    scroll-behavior: smooth;
    scroll-padding-top: 32px;
    scrollbar-width: thin;
    scrollbar-color: transparent transparent;
    transition: scrollbar-color 0.18s ease;
  }

  .reader-scroll::-webkit-scrollbar {
    width: 10px;
  }

  .reader-scroll::-webkit-scrollbar-track {
    background: transparent;
  }

  .reader-scroll::-webkit-scrollbar-thumb {
    border: 3px solid transparent;
    border-radius: 999px;
    background-clip: content-box;
    background-color: transparent;
  }

  .reader-scroll.is-scrolling {
    scrollbar-color: color-mix(in srgb, var(--faint) 55%, transparent) transparent;
  }

  .reader-scroll.is-scrolling::-webkit-scrollbar-thumb {
    background-color: color-mix(in srgb, var(--faint) 55%, transparent);
  }

  .tab-shell {
    flex: 1 1 auto;
    min-width: 120px;
    display: flex;
    align-items: center;
    gap: 6px;
    overflow: hidden;
  }

  .tab-strip {
    flex: 1 1 auto;
    min-width: 0;
    display: flex;
    gap: 6px;
    max-width: none;
    margin: 0;
    padding: 6px;
    overflow-x: auto;
    border: 1px solid var(--line);
    border-radius: 999px;
    background: color-mix(in srgb, var(--panel) 72%, transparent);
    scrollbar-width: none;
  }

  .tab-scroll-btn {
    flex: 0 0 auto;
    width: 28px;
    height: 28px;
    display: inline-grid;
    place-items: center;
    border: 1px solid var(--line);
    border-radius: 999px;
    color: var(--muted);
    background: var(--panel-strong);
    cursor: pointer;
    font-size: 18px;
    line-height: 1;
  }

  .tab-scroll-btn:hover,
  .tab-scroll-btn:focus-visible {
    color: var(--text);
    border-color: var(--accent);
    outline: none;
  }

  .tab-strip::-webkit-scrollbar {
    display: none;
  }

  .note-tab {
    flex: 0 0 auto;
    width: max-content;
    min-width: 0;
    max-width: min(280px, 42vw);
    display: inline-flex;
    align-items: center;
    gap: 7px;
    padding: 6px 8px;
    border: 1px solid transparent;
    border-radius: 999px;
    color: var(--muted);
    background: transparent;
    cursor: pointer;
    font: inherit;
    font-size: 11px;
    font-weight: 700;
  }

  .note-tab:hover,
  .note-tab.active {
    color: var(--text);
    background: var(--accent-soft);
    border-color: var(--line);
  }

  .note-tab.active {
    border-color: var(--accent);
    box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--accent) 40%, transparent);
  }

  .tab-index {
    flex: 0 0 auto;
    width: 18px;
    height: 18px;
    display: inline-grid;
    place-items: center;
    border-radius: 999px;
    color: var(--reader);
    background: var(--accent);
    font-size: 10px;
  }

  .tab-title {
    flex: 0 1 auto;
    min-width: 0;
    max-width: 190px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .tab-close {
    flex: 0 0 auto;
    width: 18px;
    height: 18px;
    display: inline-grid;
    place-items: center;
    border-radius: 999px;
    color: var(--muted);
  }

  .tab-close:hover,
  .tab-close:focus-visible {
    color: var(--text);
    background: color-mix(in srgb, var(--line) 70%, transparent);
    outline: none;
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
    padding-left: 1.1em;
    list-style-position: outside;
  }

  .reader :global(li) {
    margin: 0.12em 0;
    display: list-item;
  }

  .reader :global(li::marker) {
    color: var(--accent);
  }

  .reader :global(li input[type="checkbox"]) {
    width: 0.9em;
    height: 0.9em;
    margin: 0 0.4em 0 0;
    vertical-align: -0.12em;
    appearance: none;
    -webkit-appearance: none;
    border: 1.5px solid color-mix(in srgb, var(--muted) 55%, var(--line));
    border-radius: 0.22em;
    background: color-mix(in srgb, var(--panel) 82%, transparent);
    display: inline-grid;
    place-content: center;
  }

  .reader :global(li input[type="checkbox"]:checked) {
    border-color: var(--accent);
    background: var(--accent);
  }

  .reader :global(li input[type="checkbox"]:checked::before) {
    content: "";
    width: 0.28em;
    height: 0.5em;
    border-right: 2px solid white;
    border-bottom: 2px solid white;
    transform: translateY(-0.04em) rotate(45deg);
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

  .reader :global(.frontmatter) {
    margin: 0 0 1.2em;
    padding: 0.95em 1em;
    border: 1px solid color-mix(in srgb, var(--line) 76%, transparent);
    border-radius: 16px;
    background: color-mix(in srgb, var(--panel-strong) 72%, transparent);
  }

  .reader :global(.frontmatter-title) {
    margin: 0 0 0.75em;
    color: var(--muted);
    font-size: 0.78em;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.12em;
  }

  .reader :global(.frontmatter-list) {
    display: flex;
    flex-direction: column;
    gap: 0.7em;
    margin: 0;
  }

  .reader :global(.frontmatter-item) {
    display: flex;
    flex-direction: column;
    gap: 0.18em;
    margin: 0;
  }

  .reader :global(.frontmatter-item dt) {
    margin: 0;
    color: var(--muted);
    font-size: 0.78em;
    font-weight: 650;
    text-transform: none;
    letter-spacing: 0.02em;
  }

  .reader :global(.frontmatter-item dd) {
    margin: 0;
    color: var(--text);
    font-size: 0.96em;
    line-height: 1.45;
    word-break: break-word;
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

  .loading-card {
    text-align: center;
    margin: 60px auto;
    color: var(--muted);
    font-size: 14px;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
  }

  .pulse {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--accent);
    animation: blink 1.2s infinite;
  }

  @keyframes blink {
    0%, 100% { opacity: 0.2; }
    50% { opacity: 1; }
  }

  .hero-empty {
    text-align: center;
    max-width: 440px;
    margin: 80px auto;
    padding: 24px;
  }

  .hero-empty p {
    color: var(--muted);
    font-size: 14px;
    line-height: 1.6;
    margin: 10px 0;
  }

  .hero-empty-hint {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    padding: 9px 12px;
    border: 1px solid color-mix(in srgb, var(--line) 80%, transparent);
    border-radius: 999px;
    background: color-mix(in srgb, var(--panel) 62%, transparent);
  }

  /* Focus toast styling */
  .focus-toast {
    position: absolute;
    top: 20px;
    left: 50%;
    transform: translate(-50%, -20px);
    background: var(--panel-strong);
    border: 1.5px solid var(--accent);
    color: var(--text);
    padding: 10px 18px;
    border-radius: 20px;
    font-size: 12.5px;
    font-weight: 600;
    box-shadow: var(--shadow);
    z-index: 180;
    pointer-events: none;
    opacity: 0;
    transition: transform 0.4s cubic-bezier(0.25, 0.8, 0.25, 1), opacity 0.3s ease;
  }

  .focus-toast.show {
    transform: translate(-50%, 0);
    opacity: 0.95;
  }

  .focus-toast kbd {
    background: var(--panel);
    border: 1px solid var(--line);
    border-radius: 4px;
    padding: 1px 4px;
    font-size: 10px;
    color: var(--muted);
  }

  .focus-exit-hint {
    position: absolute;
    bottom: 24px;
    left: 50%;
    transform: translateX(-50%);
    background: var(--panel-strong);
    border: 1px solid var(--line);
    border-radius: 20px;
    padding: 6px 16px;
    font-size: 11px;
    font-weight: 600;
    color: var(--muted);
    box-shadow: var(--shadow);
    pointer-events: none;
    opacity: 0;
    transition: opacity 0.3s ease;
    z-index: 90;
  }

  .focus-exit-hint.show {
    opacity: 0.85;
  }

  .focus-exit-hint kbd {
    font-size: 10px;
    background: var(--panel);
    border: 1px solid var(--line);
    border-radius: 4px;
    padding: 1px 4px;
    color: var(--muted);
  }

  /* Floating page-map TOC */
  .toc-rail {
    position: absolute;
    top: 84px;
    right: 20px;
    width: min(var(--toc-width), calc(100% - 40px));
    max-height: calc(100vh - 132px);
    background: color-mix(in srgb, var(--reader) 18%, transparent);
    backdrop-filter: blur(20px) saturate(1.35) contrast(1.08);
    border: 1px solid color-mix(in srgb, var(--line) 58%, transparent);
    border-radius: 18px;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    transition: opacity 0.2s ease, transform 0.2s ease, background 0.2s ease;
    padding: 12px;
    box-shadow:
      0 18px 60px rgba(0, 0, 0, 0.10),
      inset 0 1px 0 color-mix(in srgb, var(--reader) 46%, transparent);
    z-index: 80;
  }

  .toc-title {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
    font-size: 11px;
    font-weight: 700;
    color: color-mix(in srgb, var(--text) 76%, var(--muted));
    text-transform: uppercase;
    letter-spacing: 0.08em;
    margin-bottom: 10px;
    text-shadow: 0 1px 10px var(--reader);
  }

  .toc-close {
    width: 22px;
    height: 22px;
    border: 1px solid var(--line);
    border-radius: 999px;
    background: color-mix(in srgb, var(--reader) 22%, transparent);
    color: color-mix(in srgb, var(--text) 76%, var(--muted));
    cursor: pointer;
    line-height: 1;
  }

  .toc-close:hover {
    color: var(--accent-strong);
    border-color: var(--accent);
  }

  .toc-search {
    width: 100%;
    padding: 8px 11px;
    border: 1px solid var(--line);
    border-radius: 999px;
    background: color-mix(in srgb, var(--reader) 38%, transparent);
    color: var(--text);
    font-size: 12px;
    outline: none;
    margin-bottom: 10px;
  }

  .toc-search:focus {
    border-color: var(--accent);
    box-shadow: 0 0 0 3px var(--accent-soft);
  }

  .toc-list {
    display: flex;
    flex-direction: column;
    gap: 4px;
    overflow-y: auto;
    padding-right: 2px;
  }

  .toc-item {
    display: block;
    padding: 6px 8px;
    border-radius: 6px;
    font-size: 12px;
    color: color-mix(in srgb, var(--text) 82%, var(--muted));
    font-weight: 650;
    text-shadow:
      0 1px 8px var(--reader),
      0 0 1px color-mix(in srgb, var(--reader) 80%, transparent);
    text-decoration: none;
    cursor: pointer;
    border-left: 2px solid transparent;
    line-height: 1.4;
    text-align: left;
    background: transparent;
    border-top: 0;
    border-right: 0;
    border-bottom: 0;
    width: 100%;
  }

  .toc-item:hover, .toc-item.active, .toc-item.selected {
    color: var(--text);
    background: color-mix(in srgb, var(--accent-soft) 68%, transparent);
  }

  .toc-item.active,
  .toc-item.selected {
    border-left-color: var(--accent);
    background: color-mix(in srgb, var(--accent-soft) 76%, transparent);
    box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--accent) 20%, transparent);
  }

  .toc-level-2 { padding-left: 18px; }
  .toc-level-3 { padding-left: 30px; }

  .toc-empty {
    font-size: 11px;
    color: var(--muted);
    margin: 10px 0;
  }

  .focus-toc {
    top: 22px;
    right: 22px;
    width: min(300px, calc(100% - 44px));
    max-height: min(420px, calc(100vh - 44px));
    background: color-mix(in srgb, var(--reader) 16%, transparent);
    box-shadow:
      0 18px 70px rgba(0, 0, 0, 0.14),
      inset 0 1px 0 color-mix(in srgb, var(--reader) 42%, transparent);
  }

  .settings-drawer {
    position: fixed;
    inset: 0;
    width: 100vw;
    height: 100vh;
    background:
      radial-gradient(circle at 82% 10%, color-mix(in srgb, var(--accent) 14%, transparent), transparent 32%),
      color-mix(in srgb, var(--panel-strong) 96%, transparent);
    backdrop-filter: blur(18px);
    z-index: 150;
    padding: clamp(18px, 3vw, 34px);
    display: flex;
    flex-direction: column;
    box-shadow: 0 30px 90px rgba(0, 0, 0, 0.24);
    opacity: 0;
    pointer-events: none;
    transform: scale(0.985);
    transition:
      opacity 0.18s ease,
      transform 0.22s cubic-bezier(0.25, 0.8, 0.25, 1);
  }

  .settings-drawer.open {
    opacity: 1;
    pointer-events: auto;
    transform: scale(1);
  }

  .settings-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 20px;
    width: min(1440px, 100%);
    margin: 0 auto 18px;
  }

  .settings-header h3 {
    margin: 0;
    font-size: 18px;
    letter-spacing: -0.015em;
  }

  .settings-kicker {
    margin: 0 0 4px;
    color: var(--muted);
    font-size: 11px;
    font-weight: 800;
    letter-spacing: 0.12em;
    text-transform: uppercase;
  }

  .settings-close {
    width: 38px;
    height: 38px;
    border: 1px solid var(--line);
    border-radius: 999px;
    cursor: pointer;
    font-size: 24px;
    color: color-mix(in srgb, var(--text) 72%, var(--muted));
    line-height: 1;
    padding: 0;
    background: color-mix(in srgb, var(--panel) 72%, transparent);
  }

  .settings-close:hover {
    color: var(--accent-strong);
    border-color: var(--accent);
    background: var(--accent-soft);
  }

  .settings-studio {
    min-height: 0;
    display: grid;
    grid-template-columns: minmax(154px, 190px) minmax(0, 1fr);
    gap: clamp(14px, 2vw, 24px);
    flex: 1;
    width: min(1440px, 100%);
    margin: 0 auto;
  }

  .settings-section-nav {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 8px;
    border: 1px solid var(--line);
    border-radius: 18px;
    background: color-mix(in srgb, var(--panel) 74%, transparent);
    align-self: start;
    max-height: 100%;
    overflow: auto;
  }

  .settings-section-nav button {
    border: 0;
    border-radius: 12px;
    padding: 9px 10px;
    color: var(--muted);
    background: transparent;
    cursor: pointer;
    font: inherit;
    font-size: 12px;
    font-weight: 750;
    text-align: left;
  }

  .settings-section-nav button:hover,
  .settings-section-nav button.active {
    color: var(--text);
    background: var(--accent-soft);
  }

  .settings-panel-content {
    display: flex;
    flex-direction: column;
    gap: 14px;
    overflow-y: auto;
    flex: 1;
    min-width: 0;
    padding-right: 4px;
  }

  .settings-section {
    padding: 14px;
    border: 1px solid var(--line);
    border-radius: 18px;
    background: color-mix(in srgb, var(--panel) 72%, transparent);
  }

  .appearance-studio {
    display: grid;
    grid-template-columns: minmax(360px, 0.95fr) minmax(320px, 0.75fr);
    gap: clamp(14px, 2vw, 24px);
    align-items: start;
  }

  .appearance-controls {
    display: flex;
    flex-direction: column;
    gap: 14px;
    min-width: 0;
  }

  .appearance-preview-pane {
    min-width: 0;
  }

  .appearance-preview-sticky {
    position: sticky;
    top: 0;
  }

  .appearance-preview-label {
    margin-bottom: 8px;
    color: var(--muted);
    font-size: 11px;
    font-weight: 800;
    letter-spacing: 0.12em;
    text-transform: uppercase;
  }

  .appearance-preview-card {
    max-width: min(100%, var(--reader-measure));
    padding: 24px;
    border: 1px solid color-mix(in srgb, var(--accent) 28%, var(--line));
    border-radius: 28px;
    background:
      radial-gradient(circle at 20% 0%, color-mix(in srgb, var(--accent) 16%, transparent), transparent 34%),
      color-mix(in srgb, var(--reader) 92%, transparent);
    box-shadow: 0 24px 60px color-mix(in srgb, var(--shadow) 34%, transparent);
    color: var(--reader-text);
    font-family: var(--reader-body-font);
    font-size: var(--reader-font-size);
    font-weight: var(--reader-weight);
    line-height: var(--reader-line-height);
  }

  .appearance-preview-meta {
    margin: 0 0 10px;
    color: var(--reader-muted);
    font-size: 0.72em;
    font-weight: 800;
    letter-spacing: 0.1em;
    text-transform: uppercase;
  }

  .appearance-preview-card h1 {
    margin: 0 0 0.5em;
    color: var(--reader-text);
    font-family: var(--reader-heading-font);
    font-size: calc(var(--reader-font-size) * var(--reader-h1-scale));
    line-height: 1.05;
    letter-spacing: -0.04em;
  }

  .appearance-preview-card p {
    margin: 0 0 var(--reader-paragraph-spacing);
  }

  .appearance-preview-card blockquote {
    margin: 0 0 var(--reader-paragraph-spacing);
    padding: 0.85em 1em;
    border-left: 4px solid var(--accent);
    border-radius: 0 18px 18px 0;
    background: color-mix(in srgb, var(--accent-soft) 62%, transparent);
    color: var(--reader-text);
  }

  .appearance-preview-card pre {
    margin: 0;
    padding: 1em;
    overflow: auto;
    border: 1px solid color-mix(in srgb, var(--accent) 18%, var(--line));
    border-radius: 18px;
    background: var(--code-bg);
    color: var(--reader-text);
    font-size: var(--reader-code-scale);
    line-height: 1.55;
  }

  .appearance-preview-card code {
    font-family: "SFMono-Regular", "JetBrains Mono", ui-monospace, monospace;
  }

  .settings-section-title {
    font-size: 11px;
    font-weight: 700;
    color: var(--muted);
    text-transform: uppercase;
    letter-spacing: 0.08em;
    margin-bottom: 8px;
  }

  .theme-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(118px, 1fr));
    gap: 8px;
  }

  .theme-bubble {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 10px;
    border: 1px solid var(--line);
    border-radius: 12px;
    font-size: 12px;
    cursor: pointer;
    background: var(--panel-strong);
    font-weight: 600;
    text-align: left;
  }

  .theme-bubble:hover, .theme-bubble.active {
    border-color: var(--accent);
    background: var(--accent-soft);
  }

  .theme-bubble span {
    width: 14px;
    height: 14px;
    border-radius: 50%;
    border: 1px solid var(--line);
    display: inline-block;
  }

  .slider-group {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .slider-item-header {
    display: flex;
    justify-content: space-between;
    font-size: 11px;
    color: var(--muted);
    margin-bottom: 4px;
  }

  .slider-item input {
    width: 100%;
    accent-color: var(--accent);
  }

  @media (max-width: 980px) {
    .appearance-studio {
      grid-template-columns: 1fr;
    }

    .appearance-preview-sticky {
      position: static;
    }
  }

  .typography-grid,
  .settings-inline-actions,
  .preset-actions {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }

  .preset-browser {
    display: grid;
    gap: 10px;
  }

  .preset-list.compact {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
    gap: 8px;
    max-height: 150px;
    overflow: auto;
  }

  .preset-list button {
    display: grid;
    grid-template-columns: auto minmax(0, 1fr) auto;
    align-items: center;
    gap: 8px;
    padding: 9px 10px;
    border: 1px solid var(--line);
    border-radius: 13px;
    color: var(--text);
    background: var(--panel-strong);
    cursor: pointer;
    font: inherit;
    font-size: 12px;
    font-weight: 750;
    text-align: left;
  }

  .preset-list button.active {
    border-color: var(--accent);
    background: var(--accent-soft);
  }

  .preset-list button span:not(.preset-dot) {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .preset-list button small {
    color: var(--muted);
    font-size: 10px;
    font-weight: 700;
  }

  .preset-dot {
    width: 16px;
    height: 16px;
    border: 1px solid var(--line);
    border-radius: 999px;
  }

  .custom-dot {
    background: linear-gradient(135deg, var(--accent), var(--reader));
  }

  .settings-help {
    margin: 0 0 12px;
    color: var(--muted);
    font-size: 13px;
    line-height: 1.55;
  }

  .settings-root-list,
  .settings-shortcut-list {
    display: grid;
    gap: 8px;
  }

  .settings-root-row,
  .settings-shortcut-list span {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 10px;
    border: 1px solid var(--line);
    border-radius: 13px;
    background: var(--panel-strong);
    color: var(--text);
    font-size: 12px;
  }

  .settings-root-row {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto;
  }

  .btn-action.danger {
    color: #d15a4f;
  }

  .presets-textarea {
    width: 100%;
    flex: 1;
    min-height: 200px;
    font-family: monospace;
    font-size: 11px;
    padding: 10px;
    border-radius: 8px;
    border: 1px solid var(--line);
    background: var(--panel);
    color: var(--text);
    outline: none;
    resize: none;
  }

  .compact-json {
    min-height: 120px;
    max-height: 160px;
    flex: 0 0 auto;
  }

  @media (max-width: 760px) {
    .settings-drawer {
      padding: 14px;
    }

    .settings-studio {
      grid-template-columns: 1fr;
      overflow: hidden;
    }

    .settings-section-nav {
      flex-direction: row;
      overflow-x: auto;
      max-height: none;
    }

    .settings-section-nav button {
      white-space: nowrap;
    }
  }

  /* Shortcuts HUD Overlay */
  .hud-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0,0,0,0.2);
    backdrop-filter: blur(4px);
    display: grid;
    place-items: center;
    z-index: 500;
  }

  .hud-panel {
    background: var(--panel-strong);
    border: 1px solid var(--line);
    border-radius: 16px;
    width: 440px;
    max-width: 90%;
    padding: 24px;
    box-shadow: var(--shadow);
  }

  .hud-title {
    margin: 0 0 16px;
    font-size: 18px;
    letter-spacing: -0.015em;
  }

  .hud-list {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .hud-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 13px;
  }

  .hud-desc {
    color: var(--text);
  }

  .hud-keys {
    display: flex;
    gap: 4px;
  }

  .hud-key {
    background: var(--panel);
    border: 1px solid var(--line);
    border-radius: 4px;
    padding: 2px 6px;
    font-size: 10px;
    font-weight: 700;
    color: var(--muted);
  }

  /* Command palette styles */
  .palette-backdrop {
    position: fixed;
    inset: 0;
    display: grid;
    place-items: start center;
    padding-top: 14vh;
    background: rgba(0, 0, 0, 0.26);
    backdrop-filter: blur(10px);
    z-index: 400;
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

  .palette-meta kbd {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 18px;
    margin: 0 2px;
    padding: 1px 5px;
    border: 1px solid var(--line);
    border-radius: 6px;
    background: var(--panel);
    color: var(--text);
    font-size: 10px;
  }

  .palette-results {
    max-height: 420px;
    overflow: auto;
    padding: 10px;
  }

  .palette-group + .palette-group {
    margin-top: 14px;
  }

  .palette-group-title {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 7px 10px 6px;
    color: color-mix(in srgb, var(--text) 72%, var(--accent));
    font-size: 11px;
    font-weight: 800;
    letter-spacing: 0.16em;
    text-transform: uppercase;
  }

  .palette-group-title small {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 22px;
    height: 18px;
    padding: 0 7px;
    border: 1px solid color-mix(in srgb, var(--accent) 18%, var(--line));
    border-radius: 999px;
    background: color-mix(in srgb, var(--accent-soft) 55%, transparent);
    color: var(--muted);
    font-size: 10px;
    letter-spacing: 0;
  }

  .palette-row {
    display: flex;
    flex-direction: column;
    width: 100%;
    padding: 11px 12px;
    border: 1px solid transparent;
    border-radius: 15px;
    color: var(--text);
    background: transparent;
    text-align: left;
    cursor: pointer;
    transition: background 0.08s linear, border-color 0.08s linear;
  }

  .palette-row:hover,
  .palette-row.active {
    border-color: color-mix(in srgb, var(--accent) 22%, var(--line));
    background: linear-gradient(
      135deg,
      color-mix(in srgb, var(--accent-soft) 78%, transparent),
      color-mix(in srgb, var(--panel) 72%, transparent)
    );
  }

  .palette-row.active {
    box-shadow: inset 3px 0 0 var(--accent);
  }

  .palette-row-copy {
    display: grid;
    grid-template-columns: max-content minmax(0, 1fr);
    align-items: baseline;
    column-gap: 10px;
    row-gap: 2px;
    width: 100%;
    min-width: 0;
  }

  .palette-row-copy small {
    grid-column: 2;
    overflow: hidden;
    max-width: 100%;
    color: var(--muted);
    font-size: 11px;
    line-height: 1.35;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .palette-row-type {
    display: inline-flex;
    align-items: center;
    min-height: 20px;
    padding: 0 8px;
    border: 1px solid color-mix(in srgb, var(--accent) 16%, var(--line));
    border-radius: 999px;
    background: color-mix(in srgb, var(--panel) 86%, var(--accent-soft));
    color: color-mix(in srgb, var(--muted) 82%, var(--text));
    font-size: 10px;
    font-weight: 800;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    white-space: nowrap;
  }

  .palette-row-title {
    overflow: hidden;
    min-width: 0;
    color: var(--text);
    font-size: 14px;
    font-weight: 750;
    line-height: 1.35;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .cmd-palette-item-content {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 12px;
    width: 100%;
    min-width: 0;
  }

  .cmd-badges {
    display: flex;
    flex: 0 0 auto;
    gap: 4px;
  }

  .cmd-badge {
    background: var(--panel);
    border: 1px solid var(--line);
    border-radius: 4px;
    padding: 1px 4px;
    font-size: 9px;
    color: var(--muted);
  }

  .palette-empty {
    padding: 16px;
    text-align: center;
    color: var(--muted);
    font-size: 13px;
  }

  /* Onboarding Walkthrough styles */
  .tour-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.4);
    z-index: 999;
    display: none;
    pointer-events: auto;
  }
  .tour-backdrop.open {
    display: block;
  }
  .tour-close {
    position: absolute;
    top: 10px;
    right: 12px;
    background: none;
    border: none;
    font-size: 14px;
    font-weight: 700;
    color: var(--muted);
    cursor: pointer;
    padding: 4px;
    line-height: 1;
    transition: color 0.15s ease;
  }
  .tour-close:hover {
    color: var(--accent-strong);
  }
  .btn-skip {
    background: transparent !important;
    border: 1px solid transparent !important;
    color: var(--muted) !important;
    font-size: 11px !important;
    padding: 4px 8px !important;
  }
  .btn-skip:hover {
    color: var(--accent-strong) !important;
    border-color: var(--line) !important;
  }
  .tour-card {
    position: absolute;
    background: var(--panel-strong);
    border: 1.5px solid var(--accent);
    border-radius: 12px;
    padding: 16px 20px;
    width: 290px;
    box-shadow: var(--shadow);
    z-index: 1000;
    display: flex;
    flex-direction: column;
    gap: 12px;
    transition: top 0.3s cubic-bezier(0.25, 0.8, 0.25, 1), left 0.3s cubic-bezier(0.25, 0.8, 0.25, 1);
  }
  .tour-card h4 {
    margin: 0;
    font-size: 14px;
    font-weight: 800;
    color: var(--accent-strong);
    letter-spacing: -0.015em;
  }
  .tour-card p {
    margin: 0;
    font-size: 12.5px;
    line-height: 1.45;
    color: var(--text);
  }
  .tour-actions {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-top: 4px;
  }
  .tour-actions span {
    font-size: 11px;
    color: var(--muted);
    font-weight: 700;
  }
  .tour-highlight {
    position: absolute;
    border: 3px solid var(--accent-strong);
    border-radius: 10px;
    box-shadow: 0 0 0 9999px rgba(0, 0, 0, 0.45);
    z-index: 998;
    pointer-events: none;
    transition: all 0.3s cubic-bezier(0.25, 0.8, 0.25, 1);
  }

  @media (max-width: 820px) {
    .app-shell {
      grid-template-columns: 48px minmax(0, 1fr);
    }

    .sidebar {
      display: none;
    }

    .rail {
      grid-column: 1;
      padding-inline: 6px;
    }

    .reader-frame {
      grid-column: 2;
    }

    .reader-scroll {
      padding: 32px 18px 64px;
    }

    .toc-rail {
      display: none;
    }
  }
</style>
