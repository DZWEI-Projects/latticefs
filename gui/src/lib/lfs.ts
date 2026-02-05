import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

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

export const getRepoInfo = async (): Promise<RepoInfo> => {
  return invoke<RepoInfo>("get_repo_info");
};

export const initRepo = async (): Promise<RepoInfo> => {
  return invoke<RepoInfo>("init_repo");
};

export const importPaths = async (
  targets: ImportTarget[]
): Promise<ImportSummary> => {
  return invoke<ImportSummary>("import_paths", { targets });
};

export const createSampleFiles = async (): Promise<SampleFilesResult> => {
  return invoke<SampleFilesResult>("create_sample_files");
};

export const onImportProgress = async (
  handler: (progress: ImportProgress) => void
): Promise<UnlistenFn> => {
  return listen<ImportProgress>("import_progress", (event) => {
    handler(event.payload);
  });
};
