use notify::{recommended_watcher, Event, RecommendedWatcher, RecursiveMode, Watcher};
use pulldown_cmark::{html, Options, Parser};
use serde::Serialize;
use serde_yaml::Value as YamlValue;
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicU64, Ordering},
        Mutex,
    },
    thread,
    time::{Duration, UNIX_EPOCH},
};
use tauri::{AppHandle, Emitter, Manager, State};
use walkdir::WalkDir;

#[derive(Default)]
struct AppState {
    vault: Mutex<VaultState>,
    watcher: Mutex<Option<RecommendedWatcher>>,
    render_cache: Mutex<HashMap<String, CachedRendered>>,
    watcher_generation: AtomicU64,
}

#[derive(Default)]
struct VaultState {
    roots: Vec<VaultRoot>,
    files: Vec<FileEntry>,
}

#[derive(Clone)]
struct VaultRoot {
    id: String,
    path: PathBuf,
    name: String,
}

#[derive(Clone)]
struct CachedRendered {
    modified: i64,
    size: u64,
    note: RenderedNote,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct FileEntry {
    root_id: String,
    root_name: String,
    path: String,
    title: String,
    modified: i64,
    size: u64,
    #[serde(skip)]
    search_key: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct VaultSnapshot {
    roots: Vec<RootEntry>,
    files: Vec<FileEntry>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct RootEntry {
    id: String,
    path: String,
    name: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct RenderedNote {
    path: String,
    title: String,
    html: String,
    modified: i64,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct OpenedMarkdown {
    snapshot: VaultSnapshot,
    note: RenderedNote,
    root_id: String,
    path: String,
}

fn is_markdown(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| matches!(ext.to_ascii_lowercase().as_str(), "md" | "markdown" | "mdx"))
        .unwrap_or(false)
}

fn rel_path(root: &Path, path: &Path) -> Option<String> {
    path.strip_prefix(root)
        .ok()
        .and_then(|rel| rel.to_str())
        .map(|rel| rel.replace(std::path::MAIN_SEPARATOR, "/"))
}

fn title_from_path(path: &str) -> String {
    Path::new(path)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .map(|stem| stem.replace(['-', '_'], " "))
        .filter(|title| !title.trim().is_empty())
        .unwrap_or_else(|| "Untitled".to_string())
}

fn modified_secs(path: &Path) -> i64 {
    fs::metadata(path)
        .ok()
        .and_then(|metadata| metadata.modified().ok())
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0)
}

fn file_size(path: &Path) -> u64 {
    fs::metadata(path).map(|metadata| metadata.len()).unwrap_or(0)
}

fn root_name(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.trim().is_empty())
        .unwrap_or("Folder")
        .to_string()
}

fn root_id(path: &Path) -> String {
    path.to_string_lossy().to_string()
}

fn scan_markdown_files(root: &VaultRoot) -> Vec<FileEntry> {
    let mut files = Vec::new();
    for entry in WalkDir::new(&root.path)
        .max_depth(64)
        .into_iter()
        .filter_entry(|entry| {
            let name = entry.file_name().to_string_lossy();
            !matches!(
                name.as_ref(),
                ".git" | "node_modules" | ".svelte-kit" | "target" | ".tauri" | ".scratch"
            )
        })
        .flatten()
    {
        let path = entry.path();
        if !path.is_file() || !is_markdown(path) {
            continue;
        }
        let Some(relative_path) = rel_path(&root.path, path) else {
            continue;
        };
        let size = entry.metadata().map(|metadata| metadata.len()).unwrap_or(0);
        let title = title_from_path(&relative_path);
        let search_key = format!("{} {}", title.to_lowercase(), relative_path.to_lowercase());
        files.push(FileEntry {
            root_id: root.id.clone(),
            root_name: root.name.clone(),
            title,
            path: relative_path,
            modified: modified_secs(path),
            size,
            search_key,
        });
    }
    files.sort_by(|a, b| {
        a.root_name
            .to_lowercase()
            .cmp(&b.root_name.to_lowercase())
            .then_with(|| a.path.to_lowercase().cmp(&b.path.to_lowercase()))
    });
    files
}

fn file_entry(root: &Path, path: &Path) -> Option<FileEntry> {
    if !path.is_file() || !is_markdown(path) {
        return None;
    }
    let relative_path = rel_path(root, path)?;
    let size = fs::metadata(path).map(|metadata| metadata.len()).unwrap_or(0);
    let title = title_from_path(&relative_path);
    let search_key = format!("{} {}", title.to_lowercase(), relative_path.to_lowercase());
    let root_name = root_name(root);
    Some(FileEntry {
        root_id: root_id(root),
        root_name,
        title,
        path: relative_path,
        modified: modified_secs(path),
        size,
        search_key,
    })
}

fn snapshot(state: &VaultState) -> VaultSnapshot {
    VaultSnapshot {
        roots: state
            .roots
            .iter()
            .map(|root| RootEntry {
                id: root.id.clone(),
                path: root.path.to_string_lossy().into_owned(),
                name: root.name.clone(),
            })
            .collect(),
        files: state.files.clone(),
    }
}

fn split_frontmatter(markdown: &str) -> (Option<String>, &str) {
    let markdown = markdown.strip_prefix('\u{feff}').unwrap_or(markdown);
    let Some(rest) = markdown.strip_prefix("---\n").or_else(|| markdown.strip_prefix("---\r\n")) else {
        return (None, markdown);
    };
    if let Some(end) = rest.find("\n---\n") {
        let yaml = &rest[..end];
        let body = &rest[end + 5..];
        return (Some(yaml.to_string()), body.trim_start_matches(['\n', '\r']));
    }
    if let Some(end) = rest.find("\r\n---\r\n") {
        let yaml = &rest[..end];
        let body = &rest[end + 7..];
        return (Some(yaml.to_string()), body.trim_start_matches(['\n', '\r']));
    }
    (None, markdown)
}

fn render_frontmatter(frontmatter: &YamlValue) -> String {
    let Some(map) = frontmatter.as_mapping() else {
        return String::new();
    };
    if map.is_empty() {
        return String::new();
    }
    let mut html = String::from("<section class=\"frontmatter\"><div class=\"frontmatter-title\">Document details</div><dl class=\"frontmatter-list\">");
    for (key, value) in map {
        let key = key.as_str().unwrap_or("field");
        let value = match value {
            YamlValue::String(text) => text.clone(),
            YamlValue::Number(num) => num.to_string(),
            YamlValue::Bool(flag) => flag.to_string(),
            YamlValue::Sequence(items) => items
                .iter()
                .map(|item| match item {
                    YamlValue::String(text) => text.clone(),
                    YamlValue::Number(num) => num.to_string(),
                    YamlValue::Bool(flag) => flag.to_string(),
                    other => other.as_str().unwrap_or("").to_string(),
                })
                .collect::<Vec<_>>()
                .join(", "),
            other => other.as_str().unwrap_or("").to_string(),
        };
        if value.is_empty() {
            continue;
        }
        html.push_str("<div class=\"frontmatter-item\"><dt>");
        html.push_str(&html_escape::encode_text(key));
        html.push_str("</dt><dd>");
        html.push_str(&html_escape::encode_text(&value));
        html.push_str("</dd></div>");
    }
    html.push_str("</dl></section>");
    html
}

fn render_markdown(markdown: &str) -> String {
    let (frontmatter, body_markdown) = split_frontmatter(markdown);
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_HEADING_ATTRIBUTES);

    let parser = Parser::new_ext(body_markdown, options);
    let mut body = String::new();
    html::push_html(&mut body, parser);
    let frontmatter_html = frontmatter
        .and_then(|yaml| serde_yaml::from_str::<YamlValue>(&yaml).ok())
        .map(|value| render_frontmatter(&value))
        .unwrap_or_default();
    let combined = if frontmatter_html.is_empty() {
        body
    } else {
        format!("{frontmatter_html}{body}")
    };
    ammonia::Builder::default()
        .add_tags(["input"])
        .add_tags(["section", "dl", "dt", "dd"])
        .add_generic_attributes(["class", "checked", "disabled"])
        .clean(&combined)
        .to_string()
}

fn note_title(markdown: &str, note_path: &str) -> String {
    let (frontmatter, body_markdown) = split_frontmatter(markdown);
    if let Some(yaml) = frontmatter
        .and_then(|yaml| serde_yaml::from_str::<YamlValue>(&yaml).ok())
        .and_then(|value| value.as_mapping().cloned())
    {
        if let Some(title) = yaml
            .get(&YamlValue::String("title".to_string()))
            .and_then(|value| value.as_str())
            .map(str::trim)
            .filter(|title| !title.is_empty())
        {
            return title.to_string();
        }
    }
    body_markdown
        .lines()
        .find_map(|line| line.trim().strip_prefix("# ").map(str::trim))
        .filter(|title| !title.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| title_from_path(note_path))
}

fn abs_note_path(root: &Path, relative_path: &str) -> Result<PathBuf, String> {
    if relative_path.contains('\\') {
        return Err("Invalid path".to_string());
    }
    let joined = root.join(relative_path);
    let canonical_root = root.canonicalize().map_err(|e| e.to_string())?;
    let canonical_file = joined.canonicalize().map_err(|e| e.to_string())?;
    if !canonical_file.starts_with(&canonical_root) || !is_markdown(&canonical_file) {
        return Err("Path is outside vault or not markdown".to_string());
    }
    Ok(canonical_file)
}

fn sort_files(files: &mut [FileEntry]) {
    files.sort_by(|a, b| {
        a.root_name
            .to_lowercase()
            .cmp(&b.root_name.to_lowercase())
            .then_with(|| a.path.to_lowercase().cmp(&b.path.to_lowercase()))
    });
}

fn rebuild_watcher(app: AppHandle, roots: &[VaultRoot]) -> Result<(), String> {
    let app_for_watcher = app.clone();
    let mut watcher = recommended_watcher(move |result: notify::Result<notify::Event>| {
        if let Ok(event) = result {
            if event.paths.iter().any(|path| is_markdown(path)) {
                schedule_refresh(app_for_watcher.clone(), event);
            }
        }
    })
    .map_err(|e| e.to_string())?;
    for root in roots {
        watcher
            .watch(&root.path, RecursiveMode::Recursive)
            .map_err(|e| e.to_string())?;
    }
    let state = app.state::<AppState>();
    *state.watcher.lock().expect("watcher mutex") = Some(watcher);
    Ok(())
}

fn refresh_vault(app: &AppHandle, state: &AppState) {
    let maybe_snapshot = {
        let mut vault = state.vault.lock().expect("vault mutex");
        if vault.roots.is_empty() {
            return;
        }
        vault.files = vault.roots.iter().flat_map(scan_markdown_files).collect();
        sort_files(&mut vault.files);
        snapshot(&vault)
    };
    let _ = app.emit("vault-updated", maybe_snapshot);
}

fn apply_watch_event(app: &AppHandle, state: &AppState, event: Event) -> bool {
    let mut changed = false;
    {
        let mut vault = state.vault.lock().expect("vault mutex");
        if vault.roots.is_empty() {
            return false;
        }
        for path in event.paths {
            if path.is_dir() {
                return false;
            }
            if !is_markdown(&path) {
                continue;
            }
            let Some(root) = vault.roots.iter().find(|root| path.starts_with(&root.path)).cloned() else {
                continue;
            };
            let Some(relative_path) = rel_path(&root.path, &path) else {
                continue;
            };
            if let Some(mut entry) = file_entry(&root.path, &path) {
                entry.root_id = root.id.clone();
                entry.root_name = root.name.clone();
                if let Some(existing) = vault
                    .files
                    .iter_mut()
                    .find(|file| file.root_id == root.id && file.path == relative_path)
                {
                    *existing = entry;
                } else {
                    vault.files.push(entry);
                }
            } else {
                vault.files.retain(|file| !(file.root_id == root.id && file.path == relative_path));
            }
            state
                .render_cache
                .lock()
                .expect("render cache mutex")
                .remove(&format!("{}:{}", root.id, relative_path));
            changed = true;
        }
        if changed {
            sort_files(&mut vault.files);
            let _ = app.emit("vault-updated", snapshot(&vault));
        }
    }
    changed
}

fn schedule_refresh(app: AppHandle, event: Event) {
    let state = app.state::<AppState>();
    let generation = state.watcher_generation.fetch_add(1, Ordering::SeqCst) + 1;
    tauri::async_runtime::spawn_blocking(move || {
        thread::sleep(Duration::from_millis(220));
        let state = app.state::<AppState>();
        if state.watcher_generation.load(Ordering::SeqCst) == generation {
            if !apply_watch_event(&app, &state, event) {
                refresh_vault(&app, &state);
            }
        }
    });
}

#[tauri::command]
async fn open_vaults(app: AppHandle, state: State<'_, AppState>, paths: Vec<String>) -> Result<VaultSnapshot, String> {
    let roots = paths
        .into_iter()
        .map(PathBuf::from)
        .filter(|path| path.is_dir())
        .map(|path| VaultRoot {
            id: root_id(&path),
            name: root_name(&path),
            path,
        })
        .collect::<Vec<_>>();
    if roots.is_empty() {
        return Err("Select at least one folder".to_string());
    }

    let scan_roots = roots.clone();
    let mut files = tauri::async_runtime::spawn_blocking(move || {
        scan_roots.iter().flat_map(scan_markdown_files).collect::<Vec<_>>()
    })
        .await
        .map_err(|e| e.to_string())?;
    sort_files(&mut files);
    {
        let mut vault = state.vault.lock().expect("vault mutex");
        vault.roots = roots.clone();
        vault.files = files;
    }
    state.render_cache.lock().expect("render cache mutex").clear();

    rebuild_watcher(app, &roots)?;

    let vault = state.vault.lock().expect("vault mutex");
    Ok(snapshot(&vault))
}

#[tauri::command]
async fn remove_vault(app: AppHandle, state: State<'_, AppState>, root_id: String) -> Result<VaultSnapshot, String> {
    let roots = {
        let vault = state.vault.lock().expect("vault mutex");
        vault
            .roots
            .iter()
            .filter(|root| root.id != root_id)
            .map(|root| root.path.to_string_lossy().into_owned())
            .collect::<Vec<_>>()
    };
    if roots.is_empty() {
        {
            let mut vault = state.vault.lock().expect("vault mutex");
            vault.roots.clear();
            vault.files.clear();
        }
        state.render_cache.lock().expect("render cache mutex").clear();
        *state.watcher.lock().expect("watcher mutex") = None;
        let vault = state.vault.lock().expect("vault mutex");
        return Ok(snapshot(&vault));
    }
    open_vaults(app, state, roots).await
}

#[tauri::command]
async fn add_vault(app: AppHandle, state: State<'_, AppState>, path: String) -> Result<VaultSnapshot, String> {
    let mut paths = {
        let vault = state.vault.lock().expect("vault mutex");
        vault
            .roots
            .iter()
            .map(|root| root.path.to_string_lossy().into_owned())
            .collect::<Vec<_>>()
    };
    if !paths.iter().any(|existing| existing == &path) {
        paths.push(path);
    }
    open_vaults(app, state, paths).await
}

#[tauri::command]
fn get_vault(state: State<AppState>) -> VaultSnapshot {
    let vault = state.vault.lock().expect("vault mutex");
    snapshot(&vault)
}

#[tauri::command]
fn search_files(state: State<AppState>, query: String, limit: usize) -> Vec<FileEntry> {
    let needle = query.trim().to_lowercase();
    let limit = limit.clamp(1, 80);
    let vault = state.vault.lock().expect("vault mutex");
    if needle.is_empty() {
        return vault.files.iter().take(limit).cloned().collect();
    }
    vault
        .files
        .iter()
        .filter(|file| file.search_key.contains(&needle))
        .take(limit)
        .cloned()
        .collect()
}

#[tauri::command]
async fn render_note(state: State<'_, AppState>, root_id: String, path: String) -> Result<RenderedNote, String> {
    let root = {
        let vault = state.vault.lock().expect("vault mutex");
        vault
            .roots
            .iter()
            .find(|root| root.id == root_id)
            .map(|root| root.path.clone())
            .ok_or("Open this note's folder first")?
    };
    let file_path = abs_note_path(&root, &path)?;
    let modified = modified_secs(&file_path);
    let size = file_size(&file_path);
    let cache_key = format!("{}:{}", root_id, path);
    if let Some(cached) = state.render_cache.lock().expect("render cache mutex").get(&cache_key) {
        if cached.modified == modified && cached.size == size {
            return Ok(cached.note.clone());
        }
    }
    let note = {
        let file_path = file_path.clone();
        let note_path = path.clone();
        tauri::async_runtime::spawn_blocking(move || -> Result<RenderedNote, String> {
            let markdown = fs::read_to_string(&file_path).map_err(|e| e.to_string())?;
            let title = note_title(&markdown, &note_path);
            Ok(RenderedNote {
                path: note_path,
                title,
                html: render_markdown(&markdown),
                modified,
            })
        })
        .await
        .map_err(|e| e.to_string())??
    };
    {
        let mut cache = state.render_cache.lock().expect("render cache mutex");
        if cache.len() >= 64 {
            if let Some(oldest_key) = cache.keys().next().cloned() {
                cache.remove(&oldest_key);
            }
        }
        cache.insert(
            cache_key,
            CachedRendered {
            modified,
            size,
            note: note.clone(),
            },
        );
    }
    Ok(note)
}

#[tauri::command]
async fn open_markdown_file(app: AppHandle, state: State<'_, AppState>, path: String) -> Result<OpenedMarkdown, String> {
    let file_path = PathBuf::from(path);
    let canonical_file = file_path.canonicalize().map_err(|e| e.to_string())?;
    if !canonical_file.is_file() || !is_markdown(&canonical_file) {
        return Err("Selected file is not a Markdown document".to_string());
    }
    let root_path = canonical_file
        .parent()
        .ok_or("Markdown file has no parent folder")?
        .to_path_buf();
    let root_id = root_id(&root_path);
    let root = VaultRoot {
        id: root_id.clone(),
        name: root_name(&root_path),
        path: root_path.clone(),
    };
    let relative_path = rel_path(&root_path, &canonical_file).ok_or("Could not resolve Markdown file path")?;
    let roots = {
        let vault = state.vault.lock().expect("vault mutex");
        let mut roots = vault.roots.clone();
        if !roots.iter().any(|existing| existing.id == root_id) {
            roots.push(root.clone());
        }
        roots
    };
    let scan_roots = roots.clone();
    let mut files = tauri::async_runtime::spawn_blocking(move || {
        scan_roots.iter().flat_map(scan_markdown_files).collect::<Vec<_>>()
    })
    .await
    .map_err(|e| e.to_string())?;
    sort_files(&mut files);
    {
        let mut vault = state.vault.lock().expect("vault mutex");
        vault.roots = roots.clone();
        vault.files = files;
    }
    state.render_cache.lock().expect("render cache mutex").clear();
    rebuild_watcher(app, &roots)?;

    let snapshot = {
        let vault = state.vault.lock().expect("vault mutex");
        snapshot(&vault)
    };
    let note = render_note(state, root.id.clone(), relative_path.clone()).await?;
    Ok(OpenedMarkdown {
        snapshot,
        note,
        root_id: root.id,
        path: relative_path,
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            open_vaults,
            add_vault,
            remove_vault,
            get_vault,
            search_files,
            render_note,
            open_markdown_file
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, event| {
            if let tauri::RunEvent::Opened { urls } = event {
                let paths = urls
                    .into_iter()
                    .filter_map(|url| url.to_file_path().ok())
                    .filter(|path| is_markdown(path))
                    .map(|path| path.to_string_lossy().into_owned())
                    .collect::<Vec<_>>();
                if !paths.is_empty() {
                    let _ = app.emit("opened-markdown-files", paths);
                }
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn scanner_finds_markdown_and_skips_other_files() {
        let dir = tempfile::tempdir().expect("tempdir");
        let md_path = dir.path().join("notes").join("hello-world.md");
        fs::create_dir_all(md_path.parent().expect("parent")).expect("mkdir");
        fs::write(&md_path, "# Hello").expect("write md");
        fs::write(dir.path().join("ignore.txt"), "No").expect("write txt");

        let root = VaultRoot {
            id: root_id(dir.path()),
            name: root_name(dir.path()),
            path: dir.path().to_path_buf(),
        };
        let files = scan_markdown_files(&root);

        assert_eq!(files.len(), 1);
        assert_eq!(files[0].root_id, root.id);
        assert_eq!(files[0].path, "notes/hello-world.md");
        assert_eq!(files[0].title, "hello world");
    }

    #[test]
    fn scanner_includes_mdx_files() {
        let dir = tempfile::tempdir().expect("tempdir");
        let mdx_path = dir.path().join("notes").join("component-note.mdx");
        fs::create_dir_all(mdx_path.parent().expect("parent")).expect("mkdir");
        fs::write(&mdx_path, "# Component").expect("write mdx");

        let root = VaultRoot {
            id: root_id(dir.path()),
            name: root_name(dir.path()),
            path: dir.path().to_path_buf(),
        };
        let files = scan_markdown_files(&root);

        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, "notes/component-note.mdx");
    }

    #[test]
    fn renderer_sanitizes_raw_script_html() {
        let html = render_markdown("# Safe\n\n<script>alert('x')</script>\n\n**bold**");

        assert!(html.contains("<h1>Safe</h1>"));
        assert!(html.contains("<strong>bold</strong>"));
        assert!(!html.contains("<script>"));
        assert!(!html.contains("alert"));
    }

    #[test]
    fn renderer_turns_frontmatter_into_metadata_block() {
        let markdown = "---\ntitle: CalmPage\nstatus: draft\ntags:\n  - markdown\n  - reader\n---\n# Ignore this heading\n\nBody text.";
        let html = render_markdown(markdown);

        assert!(html.contains("Document details"));
        assert!(html.contains("CalmPage"));
        assert!(html.contains("status"));
        assert!(html.contains("draft"));
        assert!(html.contains("markdown, reader"));
        assert!(!html.contains("Ignore this heading"));
    }

    #[test]
    fn file_entry_precomputes_search_key() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("Project_Plan.md");
        let mut file = fs::File::create(&path).expect("create");
        writeln!(file, "# Project").expect("write");

        let entry = file_entry(dir.path(), &path).expect("entry");

        assert!(entry.search_key.contains("project plan"));
        assert!(entry.search_key.contains("project_plan.md"));
    }
}
