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
  ChildPolicy,
  TagInfo,
  VersionInfo,
  VersionState,
  DiffResult,
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
    parentId: null,
    icon: "Clock",
    objectCount: 12,
  },
  {
    id: "projects",
    name: "Projekte",
    description: "Objekte mit der Eigenschaft Projekt",
    query: "tag:projekt SORT updated DESC",
    viewType: "builtin",
    parentId: null,
    icon: "Folder",
    objectCount: 4,
  },
  {
    id: "drafts",
    name: "Entwürfe",
    description: "Objekte im Entwurfsstatus",
    query: "state:draft SORT updated DESC",
    viewType: "builtin",
    parentId: null,
    icon: "FileEdit",
    objectCount: 3,
  },
  {
    id: "pending-review",
    name: "Ausstehende Prüfung",
    description: "Objekte, die auf Prüfung warten",
    query: "state:review SORT updated DESC",
    viewType: "builtin",
    parentId: null,
    icon: "Eye",
    objectCount: 1,
  },
  {
    id: "approved",
    name: "Final",
    description: "Finale Objekte",
    query: "state:approved SORT updated DESC",
    viewType: "builtin",
    parentId: null,
    icon: "CheckCircle",
    objectCount: 8,
  },
  {
    id: "all-objects",
    name: "Alle Objekte",
    description: "Alle Objekte im Repository",
    query: "trust >= 0",
    viewType: "builtin",
    parentId: null,
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
    versionCount: 3,
    currentVersionState: "approved",
    isSealed: false,
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
    versionCount: 1,
    currentVersionState: "draft",
    isSealed: false,
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
    versionCount: 1,
    currentVersionState: "draft",
    isSealed: false,
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
    versionCount: 1,
    currentVersionState: "sealed",
    isSealed: true,
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
    versionCount: 2,
    currentVersionState: "draft",
    isSealed: false,
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
    versionCount: 1,
    currentVersionState: "review",
    isSealed: false,
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
    versionCount: 1,
    currentVersionState: "draft",
    isSealed: false,
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
    versionCount: 4,
    currentVersionState: "draft",
    isSealed: false,
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
    versionCount: 2,
    currentVersionState: "approved",
    isSealed: false,
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
    versionCount: 1,
    currentVersionState: "draft",
    isSealed: false,
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
    versionCount: 2,
    currentVersionState: "review",
    isSealed: false,
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
    versionCount: 3,
    currentVersionState: "approved",
    isSealed: false,
  },
];

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
    parentId: args.parentId || null,
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
    parentId: args.parentId ?? null,
  };
  mockViewsData[index] = updated;
  return updated;
};

export const mockDeleteView = async (
  id: string,
  childPolicy?: ChildPolicy,
): Promise<void> => {
  await delay(150);
  const index = mockViewsData.findIndex((v) => v.id === id);
  if (index === -1) return;

  const children = mockViewsData.filter((v) => v.parentId === id && v.viewType === "dynamic");
  if (children.length > 0 && !childPolicy) {
    throw new Error("Diese Perspektive hat Unteransichten.");
  }

  if (childPolicy === "cascade") {
    const queue = [id];
    const toDelete = new Set<string>();
    while (queue.length > 0) {
      const current = queue.shift()!;
      toDelete.add(current);
      for (const child of mockViewsData) {
        if (child.parentId === current && child.viewType === "dynamic") {
          queue.push(child.id);
        }
      }
    }
    for (let i = mockViewsData.length - 1; i >= 0; i--) {
      if (toDelete.has(mockViewsData[i].id)) {
        mockViewsData.splice(i, 1);
      }
    }
    return;
  }

  if (childPolicy === "detach") {
    for (const child of mockViewsData) {
      if (child.parentId === id && child.viewType === "dynamic") {
        child.parentId = null;
      }
    }
  }

  mockViewsData.splice(index, 1);
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

// --- Version Operations Mocks ---

const mockVersionsStore: Record<string, VersionInfo[]> = {
  "obj-001": [
    { id: "ver-001a", number: 1, parentVersion: null, state: "discarded", sizeBytes: 2200000, createdAt: Date.now() - 86400000 * 5, commitMessage: "Erster Entwurf", isCurrent: false },
    { id: "ver-001b", number: 2, parentVersion: "ver-001a", state: "discarded", sizeBytes: 2350000, createdAt: Date.now() - 86400000 * 3, commitMessage: "Abschnitt Zeitplan ergänzt", isCurrent: false },
    { id: "ver-001c", number: 3, parentVersion: "ver-001b", state: "approved", sizeBytes: 2456000, createdAt: Date.now() - 86400000 * 2, commitMessage: "Finale Version", isCurrent: true },
  ],
  "obj-005": [
    { id: "ver-005a", number: 1, parentVersion: null, state: "discarded", sizeBytes: 12000, createdAt: Date.now() - 86400000 * 4, commitMessage: null, isCurrent: false },
    { id: "ver-005b", number: 2, parentVersion: "ver-005a", state: "draft", sizeBytes: 15000, createdAt: Date.now() - 86400000 * 1, commitMessage: "Notizen aktualisiert", isCurrent: true },
  ],
  "obj-008": [
    { id: "ver-008a", number: 1, parentVersion: null, state: "discarded", sizeBytes: 3000, createdAt: Date.now() - 86400000 * 4, commitMessage: null, isCurrent: false },
    { id: "ver-008b", number: 2, parentVersion: "ver-008a", state: "discarded", sizeBytes: 3500, createdAt: Date.now() - 86400000 * 3, commitMessage: "Funktion hinzugefügt", isCurrent: false },
    { id: "ver-008c", number: 3, parentVersion: "ver-008b", state: "discarded", sizeBytes: 4200, createdAt: Date.now() - 86400000 * 2, commitMessage: "Bug behoben", isCurrent: false },
    { id: "ver-008d", number: 4, parentVersion: "ver-008c", state: "draft", sizeBytes: 4500, createdAt: Date.now() - 86400000 * 1, commitMessage: "Tests hinzugefügt", isCurrent: true },
  ],
  "obj-012": [
    { id: "ver-012a", number: 1, parentVersion: null, state: "discarded", sizeBytes: 20000, createdAt: Date.now() - 86400000 * 10, commitMessage: "API v1 Doku", isCurrent: false },
    { id: "ver-012b", number: 2, parentVersion: "ver-012a", state: "discarded", sizeBytes: 28000, createdAt: Date.now() - 86400000 * 5, commitMessage: "Endpunkte ergänzt", isCurrent: false },
    { id: "ver-012c", number: 3, parentVersion: "ver-012b", state: "approved", sizeBytes: 32000, createdAt: Date.now() - 86400000 * 2, commitMessage: "Authentifizierung dokumentiert", isCurrent: true },
  ],
};

export const mockGetObjectVersions = async (objectId: string): Promise<VersionInfo[]> => {
  await delay(120);
  if (mockVersionsStore[objectId]) {
    return mockVersionsStore[objectId];
  }
  const obj = findObject(objectId);
  if (!obj) return [];
  return [{
    id: `ver-${objectId}-1`,
    number: 1,
    parentVersion: null,
    state: obj.currentVersionState,
    sizeBytes: obj.sizeBytes,
    createdAt: obj.createdAt,
    commitMessage: null,
    isCurrent: true,
  }];
};

export const mockGetVersionContent = async (
  _objectId: string,
  _versionId: string,
): Promise<string | null> => {
  await delay(150);
  return "# Beispielinhalt\n\nDies ist der Inhalt der ausgewählten Version.\n\n## Abschnitt 1\n\nLorem ipsum dolor sit amet.\n";
};

export const mockSetVersionState = async (
  objectId: string,
  versionId: string,
  state: VersionState,
): Promise<VersionInfo> => {
  await delay(120);
  const versions = mockVersionsStore[objectId];
  if (versions) {
    const v = versions.find((ver) => ver.id === versionId);
    if (v) {
      v.state = state;
      return v;
    }
  }
  return {
    id: versionId,
    number: 1,
    parentVersion: null,
    state,
    sizeBytes: 0,
    createdAt: Date.now(),
    commitMessage: null,
    isCurrent: true,
  };
};

export const mockReviseObject = async (
  objectId: string,
  _content: string,
  _message?: string,
): Promise<ObjectInfo> => {
  await delay(200);
  const obj = findObject(objectId);
  if (!obj) throw new Error("Objekt nicht gefunden");
  obj.versionCount += 1;
  obj.modifiedAt = Date.now();
  return obj;
};

export const mockDiffVersions = async (
  _objectId: string,
  _versionIdA: string,
  _versionIdB: string,
): Promise<DiffResult> => {
  await delay(200);
  return {
    isBinary: false,
    unifiedDiff: `--- v1
+++ v2
@@ -1,5 +1,7 @@
 # Beispielinhalt

-Dies ist der alte Inhalt.
+Dies ist der neue Inhalt.
+
+## Neuer Abschnitt

 ## Abschnitt 1
`,
    leftSize: 120,
    rightSize: 155,
    identical: false,
  };
};

export const mockCheckoutObjectVersion = async (
  objectId: string,
  _versionId: string,
): Promise<ObjectInfo> => {
  await delay(150);
  const obj = findObject(objectId);
  if (!obj) throw new Error("Objekt nicht gefunden");
  return obj;
};

export const mockExportObjectVersion = async (
  _objectId: string,
  _versionId?: string,
): Promise<void> => {
  await delay(200);
};
