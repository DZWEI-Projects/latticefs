import type { OnboardingData, OnboardingFile, OnboardingProject, OnboardingView } from "@/lib/latticeApi";

export type ViewNode = OnboardingView;

export type FileNode = {
  id: string;
  name: string;
  extension?: string;
  size: number;
  createdAt: Date;
  tags: string[];
  views: string[];
};

export type ProjectNode = OnboardingProject;

export type OnboardingState = {
  views: ViewNode[];
  files: FileNode[];
  projects: ProjectNode[];
  highlightedFileId: string | null;
};

export const mapOnboardingData = (data: OnboardingData): OnboardingState => ({
  views: data.views,
  files: data.files.map((file) => ({
    id: file.id,
    name: file.name,
    extension: file.extension ?? undefined,
    size: file.sizeBytes,
    createdAt: new Date(file.createdAt),
    tags: file.tags,
    views: file.views,
  })),
  projects: data.projects,
  highlightedFileId: data.highlightedFileId,
});

export const getFilesInView = (files: FileNode[], viewId: string): FileNode[] =>
  files.filter((file) => file.views.includes(viewId));

export const getFileById = (files: FileNode[], fileId: string): FileNode | undefined =>
  files.find((file) => file.id === fileId);
