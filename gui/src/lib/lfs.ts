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

export const isTauriApp = () =>
  typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

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

export const importPaths = async (
  targets: ImportTarget[]
): Promise<ImportSummary> => {
  if (isTauriApp()) return invoke<ImportSummary>("import_paths", { targets });
  return (await getMocks()).mockImportPaths(targets);
};

export const createSampleFiles = async (): Promise<SampleFilesResult> => {
  if (isTauriApp()) return invoke<SampleFilesResult>("create_sample_files");
  return (await getMocks()).mockCreateSampleFiles();
};

export const onImportProgress = async (
  handler: (progress: ImportProgress) => void
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
