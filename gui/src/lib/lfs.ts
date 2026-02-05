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
  parentId?: string | null;
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
}

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
        parent_id: string | null;
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
      parentId: v.parent_id,
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
  parentId?: string | null;
}

export interface UpdateViewArgs {
  id: string;
  name: string;
  query: string;
  description?: string;
  parentId?: string | null;
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
      parent_id: string | null;
    }>("create_view", { args });
    return {
      id: view.id,
      name: view.name,
      description: view.description,
      query: view.query,
      viewType: view.view_type as "builtin" | "dynamic",
      icon: view.icon,
      objectCount: view.object_count,
      parentId: view.parent_id,
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
      parent_id: string | null;
    }>("update_view", { args });
    return {
      id: view.id,
      name: view.name,
      description: view.description,
      query: view.query,
      viewType: view.view_type as "builtin" | "dynamic",
      icon: view.icon,
      objectCount: view.object_count,
      parentId: view.parent_id,
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
