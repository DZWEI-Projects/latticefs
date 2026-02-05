export interface FileNode {
  id: string;
  name: string;
  type: "file" | "folder";
  extension?: string;
  size?: number;
  createdAt: Date;
  modifiedAt: Date;
  accessedAt: Date;
  tags: string[];
  views: string[];
  connections: string[];
  metadata?: {
    source?: string;
    downloadedFrom?: string;
    project?: string;
  };
}

export interface ViewNode {
  id: string;
  name: string;
  icon: string;
  description: string;
  files: string[];
  color: "primary" | "secondary" | "warning" | "muted";
}

// Mock file system data
export const mockFiles: FileNode[] = [
  {
    id: "file-1",
    name: "Projektplan_Phoenix.pdf",
    type: "file",
    extension: "pdf",
    size: 2456000,
    createdAt: new Date("2024-01-15"),
    modifiedAt: new Date("2024-01-20"),
    accessedAt: new Date("2024-01-22"),
    tags: ["projekt:phoenix", "dokument", "wichtig"],
    views: ["neueste", "projekte", "nach-typ"],
    connections: ["file-2", "file-5"],
    metadata: {
      project: "Phoenix",
    },
  },
  {
    id: "file-2",
    name: "Rechnung_2024_001.pdf",
    type: "file",
    extension: "pdf",
    size: 145000,
    createdAt: new Date("2024-01-18"),
    modifiedAt: new Date("2024-01-18"),
    accessedAt: new Date("2024-01-19"),
    tags: ["rechnung", "2024", "finanzen"],
    views: ["neueste", "nach-typ"],
    connections: ["file-3"],
  },
  {
    id: "file-3",
    name: "setup_installer.exe",
    type: "file",
    extension: "exe",
    size: 52400000,
    createdAt: new Date("2024-01-21"),
    modifiedAt: new Date("2024-01-21"),
    accessedAt: new Date("2024-01-21"),
    tags: ["inbox:downloads", "quarantäne", "unbekannt"],
    views: ["downloads", "quarantäne"],
    connections: [],
    metadata: {
      downloadedFrom: "external-source.com",
      source: "Browser-Download",
    },
  },
  {
    id: "file-4",
    name: "Urlaub_Mallorca_2023.jpg",
    type: "file",
    extension: "jpg",
    size: 4200000,
    createdAt: new Date("2023-08-15"),
    modifiedAt: new Date("2023-08-15"),
    accessedAt: new Date("2024-01-10"),
    tags: ["foto", "urlaub", "2023"],
    views: ["nach-typ"],
    connections: ["file-6", "file-7"],
  },
  {
    id: "file-5",
    name: "Meeting_Notes_Phoenix.md",
    type: "file",
    extension: "md",
    size: 15000,
    createdAt: new Date("2024-01-19"),
    modifiedAt: new Date("2024-01-22"),
    accessedAt: new Date("2024-01-22"),
    tags: ["projekt:phoenix", "notizen", "meeting"],
    views: ["neueste", "projekte"],
    connections: ["file-1"],
    metadata: {
      project: "Phoenix",
    },
  },
  {
    id: "file-6",
    name: "Urlaub_Mallorca_2023_2.jpg",
    type: "file",
    extension: "jpg",
    size: 3800000,
    createdAt: new Date("2023-08-15"),
    modifiedAt: new Date("2023-08-15"),
    accessedAt: new Date("2024-01-10"),
    tags: ["foto", "urlaub", "2023"],
    views: ["nach-typ"],
    connections: ["file-4", "file-7"],
  },
  {
    id: "file-7",
    name: "Urlaub_Mallorca_2023_3.jpg",
    type: "file",
    extension: "jpg",
    size: 5100000,
    createdAt: new Date("2023-08-15"),
    modifiedAt: new Date("2023-08-15"),
    accessedAt: new Date("2024-01-10"),
    tags: ["foto", "urlaub", "2023"],
    views: ["nach-typ"],
    connections: ["file-4", "file-6"],
  },
  {
    id: "file-8",
    name: "Budget_2024.xlsx",
    type: "file",
    extension: "xlsx",
    size: 89000,
    createdAt: new Date("2024-01-02"),
    modifiedAt: new Date("2024-01-20"),
    accessedAt: new Date("2024-01-21"),
    tags: ["finanzen", "2024", "planung"],
    views: ["neueste", "nach-typ"],
    connections: ["file-2"],
  },
  {
    id: "file-9",
    name: "Präsentation_Q1.pptx",
    type: "file",
    extension: "pptx",
    size: 12500000,
    createdAt: new Date("2024-01-17"),
    modifiedAt: new Date("2024-01-22"),
    accessedAt: new Date("2024-01-22"),
    tags: ["präsentation", "q1", "arbeit"],
    views: ["neueste", "nach-typ", "projekte"],
    connections: ["file-1", "file-5"],
    metadata: {
      project: "Phoenix",
    },
  },
  {
    id: "file-10",
    name: "unknown_archive.zip",
    type: "file",
    extension: "zip",
    size: 156000000,
    createdAt: new Date("2024-01-22"),
    modifiedAt: new Date("2024-01-22"),
    accessedAt: new Date("2024-01-22"),
    tags: ["inbox:downloads", "quarantäne", "archiv"],
    views: ["downloads", "quarantäne"],
    connections: [],
    metadata: {
      downloadedFrom: "file-share.net",
      source: "E-Mail-Anhang",
    },
  },
  {
    id: "file-11",
    name: "Lebenslauf_aktuell.docx",
    type: "file",
    extension: "docx",
    size: 245000,
    createdAt: new Date("2023-11-10"),
    modifiedAt: new Date("2024-01-15"),
    accessedAt: new Date("2024-01-18"),
    tags: ["persönlich", "dokument", "wichtig"],
    views: ["nach-typ"],
    connections: [],
  },
  {
    id: "file-12",
    name: "code_snippet.py",
    type: "file",
    extension: "py",
    size: 4500,
    createdAt: new Date("2024-01-20"),
    modifiedAt: new Date("2024-01-21"),
    accessedAt: new Date("2024-01-22"),
    tags: ["code", "python", "projekt:phoenix"],
    views: ["neueste", "projekte", "nach-typ"],
    connections: ["file-1", "file-5"],
    metadata: {
      project: "Phoenix",
    },
  },
];

export const mockViews: ViewNode[] = [
  {
    id: "neueste",
    name: "Neueste",
    icon: "Clock",
    description: "Dinge, die du kürzlich angesehen hast.",
    files: ["file-1", "file-2", "file-5", "file-8", "file-9", "file-12"],
    color: "primary",
  },
  {
    id: "projekte",
    name: "Projekte",
    icon: "Folder",
    description: "Dateien, die zu Projekten gehören.",
    files: ["file-1", "file-5", "file-9", "file-12"],
    color: "secondary",
  },
  {
    id: "nach-typ",
    name: "Nach Typ",
    icon: "Grid",
    description: "Dokumente, Bilder, Code — automatisch organisiert.",
    files: ["file-1", "file-2", "file-4", "file-6", "file-7", "file-8", "file-9", "file-11", "file-12"],
    color: "muted",
  },
  {
    id: "downloads",
    name: "Downloads",
    icon: "Download",
    description: "Externe Inhalte. Sicherer hier.",
    files: ["file-3", "file-10"],
    color: "warning",
  },
  {
    id: "quarantäne",
    name: "Quarantäne",
    icon: "Shield",
    description: "Unbekannte oder potenziell unsichere Dateien.",
    files: ["file-3", "file-10"],
    color: "warning",
  },
];

export const mockProjects = [
  { id: "phoenix", name: "Phoenix", color: "#8b5cf6" },
  { id: "aurora", name: "Aurora", color: "#4f8fff" },
  { id: "nebula", name: "Nebula", color: "#f59e0b" },
];

// Helper functions
export const getFileById = (id: string): FileNode | undefined => {
  return mockFiles.find((f) => f.id === id);
};

export const getViewById = (id: string): ViewNode | undefined => {
  return mockViews.find((v) => v.id === id);
};

export const getFilesInView = (viewId: string): FileNode[] => {
  const view = getViewById(viewId);
  if (!view) return [];
  return view.files.map((fId) => getFileById(fId)).filter(Boolean) as FileNode[];
};

export const getFileConnections = (fileId: string): FileNode[] => {
  const file = getFileById(fileId);
  if (!file) return [];
  return file.connections.map((cId) => getFileById(cId)).filter(Boolean) as FileNode[];
};
