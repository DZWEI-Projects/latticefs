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
  UpdateViewArgs,
  TagInfo,
  ObjectVersion,
  VersionDiffResult,
  VersionState,
} from "./lfs";

const delay = (ms: number) => new Promise((r) => setTimeout(r, ms));

// Event handlers for simulated progress
let progressHandlers: ((p: ImportProgress) => void)[] = [];

export const mockGetRepoInfo = async (): Promise<RepoInfo> => {
  await delay(150);
  return {
    root: "~/NeuralFS",
    configPath: "~/.config/NeuralFS/config.toml",
  };
};

export const mockInitRepo = async (): Promise<RepoInfo> => {
  await delay(400);
  return {
    root: "~/NeuralFS",
    configPath: "~/.config/NeuralFS/config.toml",
  };
};

export const mockCheckInitialized = async (): Promise<boolean> => {
  await delay(100);
  return false;
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
    root: "~/NeuralFS-Beispiele",
    files: [
      "~/NeuralFS-Beispiele/Dokumente/Willkommen.md",
      "~/NeuralFS-Beispiele/Projekte/Phoenix/plan.txt",
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
    name: "Neueste",
    description: "Objekte, die in den letzten 7 Tagen aktualisiert wurden",
    query: "updated within 7d SORT updated DESC LIMIT 100",
    viewType: "builtin",
    icon: "Clock",
    objectCount: 12,
  },
  {
    id: "projects",
    name: "Projekte",
    description: "Objekte mit der Eigenschaft Projekt",
    query: "tag:projekt SORT updated DESC",
    viewType: "builtin",
    icon: "Folder",
    objectCount: 4,
  },
  {
    id: "drafts",
    name: "Entwürfe",
    description: "Objekte im Entwurfsstatus",
    query: "state:draft SORT updated DESC",
    viewType: "builtin",
    icon: "FileEdit",
    objectCount: 3,
  },
  {
    id: "pending-review",
    name: "Ausstehende Prüfung",
    description: "Objekte, die auf Prüfung warten",
    query: "state:review SORT updated DESC",
    viewType: "builtin",
    icon: "Eye",
    objectCount: 1,
  },
  {
    id: "approved",
    name: "Freigegeben",
    description: "Freigegebene Objekte",
    query: "state:approved SORT updated DESC",
    viewType: "builtin",
    icon: "CheckCircle",
    objectCount: 8,
  },
  {
    id: "all-objects",
    name: "Alle Objekte",
    description: "Alle Objekte im Repository",
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
      { key: "projekt", value: "phoenix" },
      { key: "typ", value: "dokument" },
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
      { key: "typ", value: "rechnung" },
      { key: "jahr", value: "2024" },
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
    tags: [{ key: "quelle", value: "downloads" }],
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
      { key: "typ", value: "foto" },
      { key: "ort", value: "mallorca" },
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
      { key: "projekt", value: "raccoon" },
      { key: "typ", value: "notizen" },
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
      { key: "typ", value: "tabelle" },
      { key: "jahr", value: "2024" },
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
      { key: "projekt", value: "raccoon" },
      { key: "typ", value: "präsentation" },
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
      { key: "projekt", value: "raccoon" },
      { key: "typ", value: "code" },
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
    tags: [{ key: "typ", value: "dokument" }],
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
    tags: [{ key: "quelle", value: "downloads" }],
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
      { key: "projekt", value: "phoenix" },
      { key: "typ", value: "design" },
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
      { key: "projekt", value: "phoenix" },
      { key: "typ", value: "dokumentation" },
    ],
    views: ["recent", "projects", "approved"],
    trustLevel: 100,
  },
];

const mockVersionText: Record<string, Record<string, string>> = {
  "obj-005": {
    "ver-005-1": `# Raccoon Notes\n\nInitial project notes.\n`,
    "ver-005-2": `# Raccoon Notes\n\nInitial project notes.\n\n## Scope\n- MVP timeline\n`,
    "ver-005-3": `# Raccoon Notes\n\nInitial project notes.\n\n## Scope\n- MVP timeline\n- Release checklist\n`,
  },
  "obj-008": {
    "ver-008-1": `def greet(name: str) -> str:\n    return f\"Hello, {name}!\"\n`,
    "ver-008-2": `def greet(name: str) -> str:\n    return f\"Hi, {name}!\"\n\nprint(greet(\"Lattice\"))\n`,
  },
  "obj-012": {
    "ver-012-1": `# API Documentation\n\nReleased API surface. No further edits allowed.\n`,
  },
};

const mockVersionData: Record<string, ObjectVersion[]> = {
  "obj-005": [
    {
      id: "ver-005-1",
      index: 1,
      createdAt: Date.now() - 86400000 * 6,
      sizeBytes: 12000,
      state: "approved",
      parentVersion: null,
      message: "Initial notes",
      isCurrent: false,
    },
    {
      id: "ver-005-2",
      index: 2,
      createdAt: Date.now() - 86400000 * 4,
      sizeBytes: 13000,
      state: "review",
      parentVersion: "ver-005-1",
      message: "Add scope section",
      isCurrent: false,
    },
    {
      id: "ver-005-3",
      index: 3,
      createdAt: Date.now() - 86400000 * 1,
      sizeBytes: 15000,
      state: "draft",
      parentVersion: "ver-005-2",
      message: "Checklist update",
      isCurrent: true,
    },
  ],
  "obj-008": [
    {
      id: "ver-008-1",
      index: 1,
      createdAt: Date.now() - 86400000 * 3,
      sizeBytes: 4200,
      state: "discarded",
      parentVersion: null,
      message: "First snippet",
      isCurrent: false,
    },
    {
      id: "ver-008-2",
      index: 2,
      createdAt: Date.now() - 86400000 * 1,
      sizeBytes: 4500,
      state: "draft",
      parentVersion: "ver-008-1",
      message: "Improve greeting",
      isCurrent: true,
    },
  ],
  "obj-012": [
    {
      id: "ver-012-1",
      index: 1,
      createdAt: Date.now() - 86400000 * 2,
      sizeBytes: 32000,
      state: "sealed",
      parentVersion: null,
      message: "Release candidate",
      isCurrent: true,
    },
  ],
};

const createVersionId = () => `ver-${Math.random().toString(36).slice(2, 10)}`;

const ensureVersionData = (objectId: string): ObjectVersion[] => {
  if (!mockVersionData[objectId]) {
    mockVersionData[objectId] = [
      {
        id: createVersionId(),
        index: 1,
        createdAt: Date.now() - 86400000,
        sizeBytes: 0,
        state: "draft",
        parentVersion: null,
        message: "Initial version",
        isCurrent: true,
      },
    ];
  }
  return mockVersionData[objectId];
};

mockObjectsData.forEach((obj) => {
  const versions = mockVersionData[obj.id];
  obj.versionCount = versions ? versions.length : 1;
});

export const mockListViews = async (): Promise<ViewInfo[]> => {
  await delay(100);
  return mockViewsData;
};

export const mockGetViewObjects = async (viewId: string): Promise<ObjectInfo[]> => {
  await delay(150);
  if (viewId === "all-objects" || viewId === "all") {
    return mockObjectsData;
  }
  return mockObjectsData.filter((obj) => obj.views.includes(viewId));
};

export const mockEvaluateQuery = async (_query: string): Promise<ObjectInfo[]> => {
  await delay(200);
  // For mock purposes, just return all objects
  return mockObjectsData;
};

// --- Object Operations Mocks ---

const findObject = (objectId: string) =>
  mockObjectsData.find((obj) => obj.id === objectId) || null;

export const mockAddObjectTag = async (
  objectId: string,
  tag: TagInfo,
): Promise<ObjectInfo | null> => {
  await delay(120);
  const object = findObject(objectId);
  if (!object) return null;
  const exists = object.tags.some(
    (existing) => existing.key === tag.key && existing.value === tag.value
  );
  if (!exists) {
    object.tags.push(tag);
  }
  return object;
};

export const mockRemoveObjectTag = async (
  objectId: string,
  tag: TagInfo,
): Promise<ObjectInfo | null> => {
  await delay(120);
  const object = findObject(objectId);
  if (!object) return null;
  object.tags = object.tags.filter(
    (existing) => !(existing.key === tag.key && existing.value === tag.value)
  );
  return object;
};

export const mockSetObjectTrustLevel = async (
  objectId: string,
  trustLevel: number | null,
): Promise<ObjectInfo | null> => {
  await delay(120);
  const object = findObject(objectId);
  if (!object) return null;
  object.trustLevel = trustLevel;
  return object;
};

export const mockOpenObject = async (_objectId: string): Promise<void> => {
  await delay(150);
};

// --- Version Mocks ---

const applyAutoAdvance = (state: VersionState): VersionState => {
  if (state === "review") return "approved";
  if (state === "draft") return "discarded";
  return state;
};

export const mockListObjectVersions = async (
  objectId: string,
): Promise<ObjectVersion[]> => {
  await delay(120);
  return ensureVersionData(objectId);
};

export const mockDiffObjectVersions = async (
  objectId: string,
  leftVersionId: string,
  rightVersionId: string,
): Promise<VersionDiffResult> => {
  await delay(120);
  const leftText = mockVersionText[objectId]?.[leftVersionId] ?? "";
  const rightText = mockVersionText[objectId]?.[rightVersionId] ?? "";
  if (leftText === rightText) {
    return {
      kind: "none",
      diff: "No differences",
      leftSize: leftText.length,
      rightSize: rightText.length,
      firstDiff: null,
    };
  }

  const leftLines = leftText.split("\n");
  const rightLines = rightText.split("\n");
  const diffLines: string[] = ["--- left", "+++ right"];
  const max = Math.max(leftLines.length, rightLines.length);
  for (let i = 0; i < max; i += 1) {
    const l = leftLines[i];
    const r = rightLines[i];
    if (l !== undefined && l !== r) diffLines.push(`-${l}`);
    if (r !== undefined && l !== r) diffLines.push(`+${r}`);
    if (l !== undefined && r !== undefined && l === r) diffLines.push(` ${l}`);
  }
  return {
    kind: "text",
    diff: diffLines.join("\n"),
    leftSize: leftText.length,
    rightSize: rightText.length,
    firstDiff: null,
  };
};

export const mockGetObjectVersionText = async (
  objectId: string,
  versionId: string,
): Promise<string> => {
  await delay(120);
  const content = mockVersionText[objectId]?.[versionId];
  if (content === undefined) {
    throw new Error("Binary content");
  }
  return content;
};

export const mockReviseObjectFromText = async (
  objectId: string,
  content: string,
  message?: string,
): Promise<void> => {
  await delay(150);
  const versions = ensureVersionData(objectId);
  const current = versions.find((version) => version.isCurrent) ?? versions.at(-1);
  if (current?.state === "sealed") {
    throw new Error("Object is sealed");
  }
  if (current) {
    current.state = applyAutoAdvance(current.state);
    current.isCurrent = false;
  }
  const newId = createVersionId();
  const newVersion: ObjectVersion = {
    id: newId,
    index: versions.length + 1,
    createdAt: Date.now(),
    sizeBytes: content.length,
    state: "draft",
    parentVersion: current?.id ?? null,
    message: message ?? null,
    isCurrent: true,
  };
  versions.push(newVersion);
  mockVersionText[objectId] = {
    ...(mockVersionText[objectId] ?? {}),
    [newId]: content,
  };
  const object = findObject(objectId);
  if (object) {
    object.modifiedAt = Date.now();
    object.versionCount = versions.length;
  }
};

export const mockReviseObjectFromFile = async (
  objectId: string,
  path: string,
  message?: string,
): Promise<void> => {
  await delay(150);
  await mockReviseObjectFromText(
    objectId,
    `Imported content from ${path}`,
    message,
  );
};

export const mockSetObjectVersionState = async (
  objectId: string,
  versionId: string,
  state: VersionState,
): Promise<void> => {
  await delay(120);
  const versions = ensureVersionData(objectId);
  const target = versions.find((version) => version.id === versionId);
  if (!target) {
    throw new Error("Version not found");
  }
  target.state = state;
};

export const mockCheckoutObjectVersion = async (
  objectId: string,
  versionId: string,
): Promise<void> => {
  await delay(120);
  const versions = ensureVersionData(objectId);
  versions.forEach((version) => {
    version.isCurrent = version.id === versionId;
  });
};

export const mockExportObjectVersion = async (
  _objectId: string,
  _versionId: string,
  _outputPath: string,
  _mode: "tree" | "archive",
): Promise<void> => {
  await delay(150);
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

export const mockUpdateView = async (args: UpdateViewArgs): Promise<ViewInfo> => {
  await delay(180);
  const index = mockViewsData.findIndex((v) => v.id === args.id);
  if (index === -1) {
    throw new Error("Perspektive nicht gefunden");
  }
  const updated: ViewInfo = {
    ...mockViewsData[index],
    name: args.name,
    description: args.description || "",
    query: args.query,
  };
  mockViewsData[index] = updated;
  return updated;
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

export const mockPickExportPath = async (
  suggestedName?: string,
): Promise<string | null> => {
  await delay(100);
  return suggestedName ? `~/Exports/${suggestedName}` : "~/Exports/export.bin";
};
