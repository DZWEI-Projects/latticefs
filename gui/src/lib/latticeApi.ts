export type FolderOption = {
  id: string;
  name: string;
  path: string;
  exists: boolean;
  defaultSelected: boolean;
  isDemo: boolean;
};

export type OnboardingSettings = {
  quarantineDownloads: boolean;
  versioning: boolean;
  executeWarning: boolean;
};

export type OnboardingView = {
  id: string;
  name: string;
  icon: "Clock" | "Folder" | "Grid" | "Download" | "Shield";
  description: string;
  color: "primary" | "secondary" | "warning" | "muted";
};

export type OnboardingFile = {
  id: string;
  name: string;
  extension: string | null;
  sizeBytes: number;
  createdAt: string;
  tags: string[];
  views: string[];
};

export type OnboardingProject = {
  id: string;
  name: string;
  color: string;
};

export type OnboardingData = {
  views: OnboardingView[];
  files: OnboardingFile[];
  projects: OnboardingProject[];
  highlightedFileId: string | null;
};

export type ImportSummary = {
  folderId: string;
  files: number;
  objects: number;
  bytes: number;
  errors: string[];
};

export type ImportResponse = {
  summaries: ImportSummary[];
  totalFiles: number;
  totalObjects: number;
  totalBytes: number;
  errors: string[];
};

const API_BASE = import.meta.env.VITE_LFS_GUI_API ?? "http://localhost:8788";

const toFolderOption = (option: {
  id: string;
  name: string;
  path: string;
  exists: boolean;
  default_selected: boolean;
  is_demo: boolean;
}): FolderOption => ({
  id: option.id,
  name: option.name,
  path: option.path,
  exists: option.exists,
  defaultSelected: option.default_selected,
  isDemo: option.is_demo,
});

const toSettings = (settings: {
  quarantine_downloads: boolean;
  versioning: boolean;
  execute_warning: boolean;
}): OnboardingSettings => ({
  quarantineDownloads: settings.quarantine_downloads,
  versioning: settings.versioning,
  executeWarning: settings.execute_warning,
});

const toImportSummary = (summary: {
  folder_id: string;
  files: number;
  objects: number;
  bytes: number;
  errors: string[];
}): ImportSummary => ({
  folderId: summary.folder_id,
  files: summary.files,
  objects: summary.objects,
  bytes: summary.bytes,
  errors: summary.errors,
});

const toImportResponse = (data: {
  summaries: {
    folder_id: string;
    files: number;
    objects: number;
    bytes: number;
    errors: string[];
  }[];
  total_files: number;
  total_objects: number;
  total_bytes: number;
  errors: string[];
}): ImportResponse => ({
  summaries: data.summaries.map(toImportSummary),
  totalFiles: data.total_files,
  totalObjects: data.total_objects,
  totalBytes: data.total_bytes,
  errors: data.errors,
});

const toOnboardingView = (view: {
  id: string;
  name: string;
  icon: "Clock" | "Folder" | "Grid" | "Download" | "Shield";
  description: string;
  color: "primary" | "secondary" | "warning" | "muted";
}): OnboardingView => view;

const toOnboardingFile = (file: {
  id: string;
  name: string;
  extension: string | null;
  size_bytes: number;
  created_at: string;
  tags: string[];
  views: string[];
}): OnboardingFile => ({
  id: file.id,
  name: file.name,
  extension: file.extension,
  sizeBytes: file.size_bytes,
  createdAt: file.created_at,
  tags: file.tags,
  views: file.views,
});

const toOnboardingProject = (project: {
  id: string;
  name: string;
  color: string;
}): OnboardingProject => project;

export const initRepo = async (): Promise<{ repoPath: string; version: string }> => {
  const response = await fetch(`${API_BASE}/api/onboarding/init`, { method: "POST" });
  if (!response.ok) {
    throw new Error("Repo konnte nicht initialisiert werden.");
  }
  const data = (await response.json()) as { repo_path: string; version: string };
  return { repoPath: data.repo_path, version: data.version };
};

export const fetchFolderOptions = async (): Promise<FolderOption[]> => {
  const response = await fetch(`${API_BASE}/api/onboarding/folders`);
  if (!response.ok) {
    throw new Error("Ordner konnten nicht geladen werden.");
  }
  const data = (await response.json()) as {
    id: string;
    name: string;
    path: string;
    exists: boolean;
    default_selected: boolean;
    is_demo: boolean;
  }[];
  return data.map(toFolderOption);
};

export const seedDemoFiles = async (): Promise<{ demoRoot: string; folders: FolderOption[] }> => {
  const response = await fetch(`${API_BASE}/api/onboarding/seed-files`, { method: "POST" });
  if (!response.ok) {
    throw new Error("Demo-Dateien konnten nicht erstellt werden.");
  }
  const data = (await response.json()) as {
    demo_root: string;
    folders: {
      id: string;
      name: string;
      path: string;
      exists: boolean;
      default_selected: boolean;
      is_demo: boolean;
    }[];
  };
  return { demoRoot: data.demo_root, folders: data.folders.map(toFolderOption) };
};

export const importFolders = async (folderIds: string[]): Promise<ImportResponse> => {
  const response = await fetch(`${API_BASE}/api/onboarding/import`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ folder_ids: folderIds }),
  });
  if (!response.ok) {
    throw new Error("Import fehlgeschlagen.");
  }
  const data = (await response.json()) as {
    summaries: {
      folder_id: string;
      files: number;
      objects: number;
      bytes: number;
      errors: string[];
    }[];
    total_files: number;
    total_objects: number;
    total_bytes: number;
    errors: string[];
  };
  return toImportResponse(data);
};

export const fetchOnboardingData = async (): Promise<OnboardingData> => {
  const response = await fetch(`${API_BASE}/api/onboarding/data`);
  if (!response.ok) {
    throw new Error("Onboarding-Daten konnten nicht geladen werden.");
  }
  const data = (await response.json()) as {
    views: {
      id: string;
      name: string;
      icon: "Clock" | "Folder" | "Grid" | "Download" | "Shield";
      description: string;
      color: "primary" | "secondary" | "warning" | "muted";
    }[];
    files: {
      id: string;
      name: string;
      extension: string | null;
      size_bytes: number;
      created_at: string;
      tags: string[];
      views: string[];
    }[];
    projects: { id: string; name: string; color: string }[];
    highlighted_file_id: string | null;
  };
  return {
    views: data.views.map(toOnboardingView),
    files: data.files.map(toOnboardingFile),
    projects: data.projects.map(toOnboardingProject),
    highlightedFileId: data.highlighted_file_id,
  };
};

export const fetchSettings = async (): Promise<OnboardingSettings> => {
  const response = await fetch(`${API_BASE}/api/onboarding/settings`);
  if (!response.ok) {
    throw new Error("Einstellungen konnten nicht geladen werden.");
  }
  const data = (await response.json()) as {
    settings: { quarantine_downloads: boolean; versioning: boolean; execute_warning: boolean };
  };
  return toSettings(data.settings);
};

export const updateSettings = async (settings: OnboardingSettings): Promise<OnboardingSettings> => {
  const response = await fetch(`${API_BASE}/api/onboarding/settings`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      quarantine_downloads: settings.quarantineDownloads,
      versioning: settings.versioning,
      execute_warning: settings.executeWarning,
    }),
  });
  if (!response.ok) {
    throw new Error("Einstellungen konnten nicht gespeichert werden.");
  }
  const data = (await response.json()) as {
    settings: { quarantine_downloads: boolean; versioning: boolean; execute_warning: boolean };
  };
  return toSettings(data.settings);
};

export const assignProject = async (
  objectId: string,
  projectId: string,
): Promise<OnboardingData> => {
  const response = await fetch(`${API_BASE}/api/onboarding/assign-project`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ object_id: objectId, project_id: projectId }),
  });
  if (!response.ok) {
    throw new Error("Projekt konnte nicht zugewiesen werden.");
  }
  const data = (await response.json()) as {
    views: {
      id: string;
      name: string;
      icon: "Clock" | "Folder" | "Grid" | "Download" | "Shield";
      description: string;
      color: "primary" | "secondary" | "warning" | "muted";
    }[];
    files: {
      id: string;
      name: string;
      extension: string | null;
      size_bytes: number;
      created_at: string;
      tags: string[];
      views: string[];
    }[];
    projects: { id: string; name: string; color: string }[];
    highlighted_file_id: string | null;
  };
  return {
    views: data.views.map(toOnboardingView),
    files: data.files.map(toOnboardingFile),
    projects: data.projects.map(toOnboardingProject),
    highlightedFileId: data.highlighted_file_id,
  };
};
