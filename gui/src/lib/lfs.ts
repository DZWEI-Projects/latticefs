import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import * as tauriPath from "@tauri-apps/api/path";

// Types
export interface RepoInfo {
  root: string;
  configPath: string;
}

export interface ImportTarget {
  path: string;
  tags: string[];
}

export interface ImportProgress {
  current: number;
  total: number;
  path: string;
}

export interface ImportSummary {
  imported: number;
  failed: number;
  errors: string[];
}

export interface SampleFilesResult {
  root: string;
  files: string[];
}

export interface OnboardingFile {
  id: string;
  name: string;
  extension?: string | null;
  views: string[];
}

export interface OnboardingGraphData {
  files: OnboardingFile[];
}

// --- Nexus / Hub View Types ---

export interface TagInfo {
  key: string;
  value: string;
}

export interface ViewInfo {
  id: string;
  name: string;
  description: string;
  query: string;
  viewType: "builtin" | "dynamic";
  icon?: string | null;
  objectCount: number;
}

export interface ObjectInfo {
  id: string;
  name: string;
  extension?: string | null;
  objectType: "blob" | "tree" | "commit";
  sizeBytes: number;
  createdAt: number;
  modifiedAt: number;
  tags: TagInfo[];
  views: string[];
  trustLevel?: number | null;
  versionCount: number;
  currentVersionState: string;
  isSealed: boolean;
}

export interface VersionInfo {
  id: string;
  number: number;
  parentVersion?: string | null;
  state: string;
  sizeBytes: number;
  createdAt: number;
  commitMessage?: string | null;
  isCurrent: boolean;
}

export interface DiffResult {
  isBinary: boolean;
  unifiedDiff?: string | null;
  leftSize: number;
  rightSize: number;
  identical: boolean;
}

export type VersionState =
  | "draft"
  | "review"
  | "approved"
  | "discarded"
  | "sealed"
  | "archived";

export const VERSION_STATE_TRANSITIONS: Record<VersionState, VersionState[]> = {
  draft: ["review", "discarded", "sealed", "archived"],
  review: ["draft", "approved", "discarded", "sealed", "archived"],
  approved: ["sealed", "archived"],
  discarded: ["archived"],
  sealed: [],
  archived: [],
};

export const VERSION_STATE_LABELS: Record<VersionState, string> = {
  draft: "Entwurf",
  review: "In Prüfung",
  approved: "Freigegeben",
  discarded: "Verworfen",
  sealed: "Versiegelt",
  archived: "Archiviert",
};

/** File extensions that can be edited as text */
export const TEXT_EDITABLE_EXTENSIONS = new Set([
  "txt", "md", "markdown", "json", "xml", "yaml", "yml", "toml",
  "csv", "tsv", "html", "htm", "css", "js", "ts", "jsx", "tsx",
  "py", "rb", "rs", "go", "java", "c", "cpp", "h", "hpp",
  "sh", "bash", "zsh", "fish", "bat", "ps1", "sql", "ini", "cfg",
  "conf", "log", "env", "gitignore", "dockerfile", "makefile",
]);

export const isTauriApp = () => {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
};

// Lazy-load mocks only when needed (avoids bundling in production Tauri build)
const getMocks = () => import("./lfs.mock");

// --- Repository Operations ---

export const getRepoInfo = async (): Promise<RepoInfo> => {
  if (isTauriApp()) return invoke<RepoInfo>("get_repo_info");
  return (await getMocks()).mockGetRepoInfo();
};

export const initRepo = async (): Promise<RepoInfo> => {
  if (isTauriApp()) return invoke<RepoInfo>("init_repo");
  return (await getMocks()).mockInitRepo();
};

export const checkInitialized = async (): Promise<boolean> => {
  if (isTauriApp()) return invoke<boolean>("check_initialized");
  return (await getMocks()).mockCheckInitialized();
};

export const importPaths = async (
  targets: ImportTarget[],
): Promise<ImportSummary> => {
  if (isTauriApp()) return invoke<ImportSummary>("import_paths", { targets });
  return (await getMocks()).mockImportPaths(targets);
};

export const createSampleFiles = async (): Promise<SampleFilesResult> => {
  if (isTauriApp()) return invoke<SampleFilesResult>("create_sample_files");
  return (await getMocks()).mockCreateSampleFiles();
};

export const getOnboardingGraph = async (): Promise<OnboardingGraphData> => {
  if (isTauriApp()) return invoke<OnboardingGraphData>("get_onboarding_graph");
  return (await getMocks()).mockGetOnboardingGraph();
};

export const onImportProgress = async (
  handler: (progress: ImportProgress) => void,
): Promise<UnlistenFn> => {
  if (isTauriApp()) {
    return listen<ImportProgress>("import_progress", (e) => handler(e.payload));
  }
  return (await getMocks()).mockOnImportProgress(handler);
};

// --- Path Operations ---

export const documentDir = async (): Promise<string> => {
  if (isTauriApp()) return tauriPath.documentDir();
  return (await getMocks()).mockDocumentDir();
};

export const downloadDir = async (): Promise<string> => {
  if (isTauriApp()) return tauriPath.downloadDir();
  return (await getMocks()).mockDownloadDir();
};

export const homeDir = async (): Promise<string> => {
  if (isTauriApp()) return tauriPath.homeDir();
  return (await getMocks()).mockHomeDir();
};

export const pictureDir = async (): Promise<string> => {
  if (isTauriApp()) return tauriPath.pictureDir();
  return (await getMocks()).mockPictureDir();
};

export const joinPath = async (...paths: string[]): Promise<string> => {
  if (isTauriApp()) return tauriPath.join(...paths);
  return (await getMocks()).mockJoin(...paths);
};

// --- Nexus / Hub View Operations ---

export const listViews = async (): Promise<ViewInfo[]> => {
  if (isTauriApp()) {
    const views = await invoke<
      Array<{
        id: string;
        name: string;
        description: string;
        query: string;
        view_type: string;
        icon: string | null;
        object_count: number;
      }>
    >("list_views");
    // Transform snake_case to camelCase
    return views.map((v) => ({
      id: v.id,
      name: v.name,
      description: v.description,
      query: v.query,
      viewType: v.view_type as "builtin" | "dynamic",
      icon: v.icon,
      objectCount: v.object_count,
    }));
  }
  return (await getMocks()).mockListViews();
};

export const getViewObjects = async (viewId: string): Promise<ObjectInfo[]> => {
  if (isTauriApp()) {
    const objects = await invoke<
      Array<{
        id: string;
        name: string;
        extension: string | null;
        object_type: string;
        size_bytes: number;
        created_at: number;
        modified_at: number;
        tags: Array<{ key: string; value: string }>;
        views: string[];
        trust_level: number | null;
        version_count: number;
        current_version_state: string;
        is_sealed: boolean;
      }>
    >("get_view_objects", { viewId });
    // Transform snake_case to camelCase
    return objects.map((o) => ({
      id: o.id,
      name: o.name,
      extension: o.extension,
      objectType: o.object_type as "blob" | "tree" | "commit",
      sizeBytes: o.size_bytes,
      createdAt: o.created_at,
      modifiedAt: o.modified_at,
      tags: o.tags,
      views: o.views,
      trustLevel: o.trust_level,
      versionCount: o.version_count,
      currentVersionState: o.current_version_state,
      isSealed: o.is_sealed,
    }));
  }
  return (await getMocks()).mockGetViewObjects(viewId);
};

export const evaluateQuery = async (query: string): Promise<ObjectInfo[]> => {
  if (isTauriApp()) {
    const objects = await invoke<
      Array<{
        id: string;
        name: string;
        extension: string | null;
        object_type: string;
        size_bytes: number;
        created_at: number;
        modified_at: number;
        tags: Array<{ key: string; value: string }>;
        views: string[];
        trust_level: number | null;
        version_count: number;
        current_version_state: string;
        is_sealed: boolean;
      }>
    >("evaluate_query", { query });
    // Transform snake_case to camelCase
    return objects.map((o) => ({
      id: o.id,
      name: o.name,
      extension: o.extension,
      objectType: o.object_type as "blob" | "tree" | "commit",
      sizeBytes: o.size_bytes,
      createdAt: o.created_at,
      modifiedAt: o.modified_at,
      tags: o.tags,
      views: o.views,
      trustLevel: o.trust_level,
      versionCount: o.version_count,
      currentVersionState: o.current_version_state,
      isSealed: o.is_sealed,
    }));
  }
  return (await getMocks()).mockEvaluateQuery(query);
};

// --- Object Operations ---

export const addObjectTag = async (
  objectId: string,
  tag: TagInfo,
): Promise<ObjectInfo | null> => {
  if (isTauriApp()) {
    return invoke<ObjectInfo>("add_object_tag", { objectId, tag });
  }
  return (await getMocks()).mockAddObjectTag(objectId, tag);
};

export const removeObjectTag = async (
  objectId: string,
  tag: TagInfo,
): Promise<ObjectInfo | null> => {
  if (isTauriApp()) {
    return invoke<ObjectInfo>("remove_object_tag", { objectId, tag });
  }
  return (await getMocks()).mockRemoveObjectTag(objectId, tag);
};

export const setObjectTrustLevel = async (
  objectId: string,
  trustLevel: number | null,
): Promise<ObjectInfo | null> => {
  if (isTauriApp()) {
    return invoke<ObjectInfo>("set_object_trust_level", {
      objectId,
      trustLevel,
    });
  }
  return (await getMocks()).mockSetObjectTrustLevel(objectId, trustLevel);
};

export const openObject = async (objectId: string): Promise<void> => {
  if (isTauriApp()) {
    await invoke("open_object", { objectId });
    return;
  }
  return (await getMocks()).mockOpenObject(objectId);
};

// --- View Management ---

export interface CreateViewArgs {
  name: string;
  query: string;
  description?: string;
}

export interface UpdateViewArgs {
  id: string;
  name: string;
  query: string;
  description?: string;
}

export const createView = async (args: CreateViewArgs): Promise<ViewInfo> => {
  if (isTauriApp()) {
    const view = await invoke<{
      id: string;
      name: string;
      description: string;
      query: string;
      view_type: string;
      icon: string | null;
      object_count: number;
    }>("create_view", { args });
    return {
      id: view.id,
      name: view.name,
      description: view.description,
      query: view.query,
      viewType: view.view_type as "builtin" | "dynamic",
      icon: view.icon,
      objectCount: view.object_count,
    };
  }
  return (await getMocks()).mockCreateView(args);
};

export const updateView = async (args: UpdateViewArgs): Promise<ViewInfo> => {
  if (isTauriApp()) {
    const view = await invoke<{
      id: string;
      name: string;
      description: string;
      query: string;
      view_type: string;
      icon: string | null;
      object_count: number;
    }>("update_view", { args });
    return {
      id: view.id,
      name: view.name,
      description: view.description,
      query: view.query,
      viewType: view.view_type as "builtin" | "dynamic",
      icon: view.icon,
      objectCount: view.object_count,
    };
  }
  return (await getMocks()).mockUpdateView(args);
};

export const deleteView = async (name: string): Promise<void> => {
  if (isTauriApp()) {
    await invoke("delete_view", { name });
    return;
  }
  return (await getMocks()).mockDeleteView(name);
};

// --- File Picker ---

export const pickFiles = async (): Promise<string[] | null> => {
  if (isTauriApp()) {
    const { open } = await import("@tauri-apps/plugin-dialog");
    const selected = await open({
      multiple: true,
      directory: false,
      title: "Dateien zum Import auswählen",
    });
    if (!selected) return null;
    return Array.isArray(selected) ? selected : [selected];
  }
  return (await getMocks()).mockPickFiles();
};

export const pickFolders = async (): Promise<string[] | null> => {
  if (isTauriApp()) {
    const { open } = await import("@tauri-apps/plugin-dialog");
    const selected = await open({
      multiple: true,
      directory: true,
      title: "Ordner zum Import auswählen",
    });
    if (!selected) return null;
    return Array.isArray(selected) ? selected : [selected];
  }
  return (await getMocks()).mockPickFolders();
};

// --- Version Operations ---

export const getObjectVersions = async (
  objectId: string,
): Promise<VersionInfo[]> => {
  if (isTauriApp()) {
    const versions = await invoke<
      Array<{
        id: string;
        number: number;
        parent_version: string | null;
        state: string;
        size_bytes: number;
        created_at: number;
        commit_message: string | null;
        is_current: boolean;
      }>
    >("list_object_versions", { objectId });
    return versions.map((v) => ({
      id: v.id,
      number: v.number,
      parentVersion: v.parent_version,
      state: v.state,
      sizeBytes: v.size_bytes,
      createdAt: v.created_at,
      commitMessage: v.commit_message,
      isCurrent: v.is_current,
    }));
  }
  return (await getMocks()).mockGetObjectVersions(objectId);
};

export const getVersionContent = async (
  objectId: string,
  versionId: string,
): Promise<string | null> => {
  if (isTauriApp()) {
    return invoke<string | null>("get_object_version_text", {
      objectId,
      versionId,
    });
  }
  return (await getMocks()).mockGetVersionContent(objectId, versionId);
};

export const setVersionState = async (
  objectId: string,
  versionId: string,
  state: VersionState,
): Promise<VersionInfo> => {
  if (isTauriApp()) {
    const v = await invoke<{
      id: string;
      number: number;
      parent_version: string | null;
      state: string;
      size_bytes: number;
      created_at: number;
      commit_message: string | null;
      is_current: boolean;
    }>("set_version_state", { objectId, versionId, state });
    return {
      id: v.id,
      number: v.number,
      parentVersion: v.parent_version,
      state: v.state,
      sizeBytes: v.size_bytes,
      createdAt: v.created_at,
      commitMessage: v.commit_message,
      isCurrent: v.is_current,
    };
  }
  return (await getMocks()).mockSetVersionState(objectId, versionId, state);
};

export const reviseObject = async (
  objectId: string,
  content: string,
  message?: string,
): Promise<ObjectInfo> => {
  if (isTauriApp()) {
    const o = await invoke<{
      id: string;
      name: string;
      extension: string | null;
      object_type: string;
      size_bytes: number;
      created_at: number;
      modified_at: number;
      tags: Array<{ key: string; value: string }>;
      views: string[];
      trust_level: number | null;
      version_count: number;
      current_version_state: string;
      is_sealed: boolean;
    }>("revise_object_from_text", { objectId, content, message: message ?? null });
    return {
      id: o.id,
      name: o.name,
      extension: o.extension,
      objectType: o.object_type as "blob" | "tree" | "commit",
      sizeBytes: o.size_bytes,
      createdAt: o.created_at,
      modifiedAt: o.modified_at,
      tags: o.tags,
      views: o.views,
      trustLevel: o.trust_level,
      versionCount: o.version_count,
      currentVersionState: o.current_version_state,
      isSealed: o.is_sealed,
    };
  }
  return (await getMocks()).mockReviseObject(objectId, content, message);
};

export const diffVersions = async (
  objectId: string,
  versionIdA: string,
  versionIdB: string,
): Promise<DiffResult> => {
  if (isTauriApp()) {
    const d = await invoke<{
      is_binary: boolean;
      unified_diff: string | null;
      left_size: number;
      right_size: number;
      identical: boolean;
    }>("diff_object_versions", {
      objectId,
      leftVersionId: versionIdA,
      rightVersionId: versionIdB,
    });
    return {
      isBinary: d.is_binary,
      unifiedDiff: d.unified_diff,
      leftSize: d.left_size,
      rightSize: d.right_size,
      identical: d.identical,
    };
  }
  return (await getMocks()).mockDiffVersions(objectId, versionIdA, versionIdB);
};

export const checkoutObjectVersion = async (
  objectId: string,
  versionId: string,
): Promise<ObjectInfo> => {
  if (isTauriApp()) {
    const o = await invoke<{
      id: string;
      name: string;
      extension: string | null;
      object_type: string;
      size_bytes: number;
      created_at: number;
      modified_at: number;
      tags: Array<{ key: string; value: string }>;
      views: string[];
      trust_level: number | null;
      version_count: number;
      current_version_state: string;
      is_sealed: boolean;
    }>("checkout_object_version", { objectId, versionId });
    return {
      id: o.id,
      name: o.name,
      extension: o.extension,
      objectType: o.object_type as "blob" | "tree" | "commit",
      sizeBytes: o.size_bytes,
      createdAt: o.created_at,
      modifiedAt: o.modified_at,
      tags: o.tags,
      views: o.views,
      trustLevel: o.trust_level,
      versionCount: o.version_count,
      currentVersionState: o.current_version_state,
      isSealed: o.is_sealed,
    };
  }
  return (await getMocks()).mockCheckoutObjectVersion(objectId, versionId);
};

export const exportObjectVersion = async (
  objectId: string,
  versionId?: string,
  outputPath?: string,
): Promise<void> => {
  if (isTauriApp()) {
    if (!outputPath) {
      const { save } = await import("@tauri-apps/plugin-dialog");
      const selected = await save({
        title: "Version exportieren",
      });
      if (!selected) return;
      outputPath = selected;
    }
    await invoke("export_object_version", {
      objectId,
      versionId: versionId ?? null,
      outputPath,
    });
    return;
  }
  return (await getMocks()).mockExportObjectVersion(objectId, versionId);
};

/** Check whether an extension indicates a text-editable file */
export function isTextEditable(extension?: string | null): boolean {
  if (!extension) return false;
  return TEXT_EDITABLE_EXTENSIONS.has(extension.toLowerCase());
}
