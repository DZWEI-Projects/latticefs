// Mock implementations for browser development mode.
// This file can be safely deleted to remove browser support.
// When deleted, also simplify lfs.ts to remove the getMocks() calls.

import type {
  RepoInfo,
  ImportTarget,
  ImportSummary,
  ImportProgress,
  SampleFilesResult,
  OnboardingGraphData,
  ViewInfo,
  ObjectInfo,
  CreateViewArgs,
} from "./lfs";

const delay = (ms: number) => new Promise((r) => setTimeout(r, ms));

// Event handlers for simulated progress
let progressHandlers: ((p: ImportProgress) => void)[] = [];

export const mockGetRepoInfo = async (): Promise<RepoInfo> => {
  await delay(150);
  return {
    root: "~/LatticeFS",
    configPath: "~/.config/latticefs/config.toml",
  };
};

export const mockInitRepo = async (): Promise<RepoInfo> => {
  await delay(400);
  return {
    root: "~/LatticeFS",
    configPath: "~/.config/latticefs/config.toml",
  };
};

export const mockImportPaths = async (
  targets: ImportTarget[]
): Promise<ImportSummary> => {
  const total = targets.length * 3; // Simulate 3 files per target
  for (let i = 0; i < total; i++) {
    await delay(100);
    progressHandlers.forEach((h) =>
      h({ current: i + 1, total, path: `/mock/file_${i}.txt` })
    );
  }
  return { imported: total, failed: 0, errors: [] };
};

export const mockCreateSampleFiles = async (): Promise<SampleFilesResult> => {
  await delay(300);
  return {
    root: "~/LatticeFS Samples",
    files: [
      "~/LatticeFS Samples/Documents/Welcome.md",
      "~/LatticeFS Samples/Projects/Phoenix/plan.txt",
    ],
  };
};

export const mockGetOnboardingGraph = async (): Promise<OnboardingGraphData> => {
  await delay(120);
  return { files: [] };
};

export const mockOnImportProgress = async (
  handler: (p: ImportProgress) => void
): Promise<() => void> => {
  progressHandlers.push(handler);
  return () => {
    progressHandlers = progressHandlers.filter((h) => h !== handler);
  };
};

// Path mocks
export const mockDocumentDir = async () => "~/Documents";
export const mockDownloadDir = async () => "~/Downloads";
export const mockHomeDir = async () => "~";
export const mockPictureDir = async () => "~/Pictures";
export const mockJoin = async (...paths: string[]) => paths.join("/");

// --- Nexus / Hub View Mocks ---

const mockViewsData: ViewInfo[] = [
  {
    id: "recent",
    name: "Recent",
    description: "Objects updated within the last 7 days",
    query: "updated within 7d SORT updated DESC LIMIT 100",
    viewType: "builtin",
    icon: "Clock",
    objectCount: 12,
  },
  {
    id: "projects",
    name: "Projects",
    description: "Objects tagged as projects",
    query: "tag:project SORT updated DESC",
    viewType: "builtin",
    icon: "Folder",
    objectCount: 4,
  },
  {
    id: "drafts",
    name: "Drafts",
    description: "Objects in draft state",
    query: "state:draft SORT updated DESC",
    viewType: "builtin",
    icon: "FileEdit",
    objectCount: 3,
  },
  {
    id: "pending-review",
    name: "Pending Review",
    description: "Objects pending review",
    query: "state:review SORT updated DESC",
    viewType: "builtin",
    icon: "Eye",
    objectCount: 1,
  },
  {
    id: "approved",
    name: "Approved",
    description: "Approved objects",
    query: "state:approved SORT updated DESC",
    viewType: "builtin",
    icon: "CheckCircle",
    objectCount: 8,
  },
  {
    id: "all-objects",
    name: "All Objects",
    description: "All objects in the repository",
    query: "trust >= 0",
    viewType: "builtin",
    icon: "Grid",
    objectCount: 24,
  },
];

const mockObjectsData: ObjectInfo[] = [
  {
    id: "obj-001",
    name: "Projektplan_Phoenix.pdf",
    extension: "pdf",
    objectType: "blob",
    sizeBytes: 2456000,
    createdAt: Date.now() - 86400000 * 5,
    modifiedAt: Date.now() - 86400000 * 2,
    tags: [
      { key: "project", value: "phoenix" },
      { key: "type", value: "document" },
    ],
    views: ["recent", "projects"],
    trustLevel: 100,
  },
  {
    id: "obj-002",
    name: "Rechnung_2024_001.pdf",
    extension: "pdf",
    objectType: "blob",
    sizeBytes: 145000,
    createdAt: Date.now() - 86400000 * 3,
    modifiedAt: Date.now() - 86400000 * 3,
    tags: [
      { key: "type", value: "invoice" },
      { key: "year", value: "2024" },
    ],
    views: ["recent"],
    trustLevel: 100,
  },
  {
    id: "obj-003",
    name: "setup_installer.exe",
    extension: "exe",
    objectType: "blob",
    sizeBytes: 52400000,
    createdAt: Date.now() - 86400000 * 1,
    modifiedAt: Date.now() - 86400000 * 1,
    tags: [{ key: "source", value: "downloads" }],
    views: ["recent"],
    trustLevel: 45,
  },
  {
    id: "obj-004",
    name: "Urlaub_Mallorca_2023.jpg",
    extension: "jpg",
    objectType: "blob",
    sizeBytes: 4200000,
    createdAt: Date.now() - 86400000 * 180,
    modifiedAt: Date.now() - 86400000 * 180,
    tags: [
      { key: "type", value: "photo" },
      { key: "location", value: "mallorca" },
    ],
    views: [],
    trustLevel: 100,
  },
  {
    id: "obj-005",
    name: "Raccoon_Notes.md",
    extension: "md",
    objectType: "blob",
    sizeBytes: 15000,
    createdAt: Date.now() - 86400000 * 4,
    modifiedAt: Date.now() - 86400000 * 1,
    tags: [
      { key: "project", value: "raccoon" },
      { key: "type", value: "notes" },
    ],
    views: ["recent", "projects", "drafts"],
    trustLevel: 100,
  },
  {
    id: "obj-006",
    name: "Raccoon_Budget_2024.xlsx",
    extension: "xlsx",
    objectType: "blob",
    sizeBytes: 89000,
    createdAt: Date.now() - 86400000 * 20,
    modifiedAt: Date.now() - 86400000 * 2,
    tags: [
      { key: "type", value: "spreadsheet" },
      { key: "year", value: "2024" },
    ],
    views: ["recent"],
    trustLevel: 100,
  },
  {
    id: "obj-007",
    name: "Raccoon_Presentation_Q1.pptx",
    extension: "pptx",
    objectType: "blob",
    sizeBytes: 12500000,
    createdAt: Date.now() - 86400000 * 6,
    modifiedAt: Date.now() - 86400000 * 1,
    tags: [
      { key: "project", value: "raccoon" },
      { key: "type", value: "presentation" },
    ],
    views: ["recent", "projects"],
    trustLevel: 100,
  },
  {
    id: "obj-008",
    name: "Raccoon_Code_Snippet.py",
    extension: "py",
    objectType: "blob",
    sizeBytes: 4500,
    createdAt: Date.now() - 86400000 * 2,
    modifiedAt: Date.now() - 86400000 * 1,
    tags: [
      { key: "project", value: "raccoon" },
      { key: "type", value: "code" },
    ],
    views: ["recent", "projects", "drafts"],
    trustLevel: 100,
  },
  {
    id: "obj-009",
    name: "Lebenslauf_aktuell.docx",
    extension: "docx",
    objectType: "blob",
    sizeBytes: 245000,
    createdAt: Date.now() - 86400000 * 60,
    modifiedAt: Date.now() - 86400000 * 7,
    tags: [{ key: "type", value: "document" }],
    views: [],
    trustLevel: 100,
  },
  {
    id: "obj-010",
    name: "unknown_archive.zip",
    extension: "zip",
    objectType: "blob",
    sizeBytes: 156000000,
    createdAt: Date.now() - 86400000 * 1,
    modifiedAt: Date.now() - 86400000 * 1,
    tags: [{ key: "source", value: "downloads" }],
    views: ["recent"],
    trustLevel: 30,
  },
  {
    id: "obj-011",
    name: "Design_Mockup_v2.fig",
    extension: "fig",
    objectType: "blob",
    sizeBytes: 8900000,
    createdAt: Date.now() - 86400000 * 3,
    modifiedAt: Date.now() - 86400000 * 1,
    tags: [
      { key: "project", value: "phoenix" },
      { key: "type", value: "design" },
    ],
    views: ["recent", "projects", "pending-review"],
    trustLevel: 100,
  },
  {
    id: "obj-012",
    name: "API_Documentation.md",
    extension: "md",
    objectType: "blob",
    sizeBytes: 32000,
    createdAt: Date.now() - 86400000 * 10,
    modifiedAt: Date.now() - 86400000 * 2,
    tags: [
      { key: "project", value: "phoenix" },
      { key: "type", value: "documentation" },
    ],
    views: ["recent", "projects", "approved"],
    trustLevel: 100,
  },
];

export const mockListViews = async (): Promise<ViewInfo[]> => {
  await delay(100);
  return mockViewsData;
};

export const mockGetViewObjects = async (viewName: string): Promise<ObjectInfo[]> => {
  await delay(150);
  if (viewName === "all-objects" || viewName === "all") {
    return mockObjectsData;
  }
  return mockObjectsData.filter((obj) => obj.views.includes(viewName));
};

export const mockEvaluateQuery = async (_query: string): Promise<ObjectInfo[]> => {
  await delay(200);
  // For mock purposes, just return all objects
  return mockObjectsData;
};

// --- View Management Mocks ---

let dynamicViewCounter = 1;

export const mockCreateView = async (args: CreateViewArgs): Promise<ViewInfo> => {
  await delay(200);
  const newView: ViewInfo = {
    id: `dynamic-${dynamicViewCounter++}`,
    name: args.name,
    description: args.description || "",
    query: args.query,
    viewType: "dynamic",
    icon: null,
    objectCount: Math.floor(Math.random() * 10),
  };
  mockViewsData.push(newView);
  return newView;
};

export const mockDeleteView = async (name: string): Promise<void> => {
  await delay(150);
  const index = mockViewsData.findIndex((v) => v.name === name);
  if (index !== -1) {
    mockViewsData.splice(index, 1);
  }
};

// --- File Picker Mocks ---

export const mockPickFiles = async (): Promise<string[] | null> => {
  await delay(100);
  // In browser mode, return mock paths
  return ["~/Documents/example.pdf", "~/Downloads/photo.jpg"];
};

export const mockPickFolders = async (): Promise<string[] | null> => {
  await delay(100);
  // In browser mode, return mock paths
  return ["~/Documents/Projects"];
};
