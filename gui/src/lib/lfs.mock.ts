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
