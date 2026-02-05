import { cn } from "@/lib/utils";
import {
  FileText,
  FileImage,
  FileVideo,
  FileAudio,
  FileArchive,
  FileCode,
  FileSpreadsheet,
  Presentation,
  File,
} from "lucide-react";
import { Badge } from "@/components/ui/badge";
import type { ObjectInfo } from "@/lib/lfs";

interface ObjectRowProps {
  object: ObjectInfo;
  isSelected: boolean;
  isAlternate: boolean;
  onClick: (e: React.MouseEvent) => void;
  onDoubleClick: () => void;
}

function getFileIcon(ext?: string | null) {
  if (!ext) return File;
  switch (ext.toLowerCase()) {
    case "pdf":
    case "doc":
    case "docx":
    case "txt":
    case "md":
      return FileText;
    case "jpg":
    case "jpeg":
    case "png":
    case "gif":
    case "webp":
      return FileImage;
    case "mp4":
    case "mov":
    case "avi":
      return FileVideo;
    case "mp3":
    case "wav":
    case "flac":
      return FileAudio;
    case "zip":
    case "rar":
    case "7z":
      return FileArchive;
    case "py":
    case "js":
    case "ts":
    case "rs":
      return FileCode;
    case "xls":
    case "xlsx":
    case "csv":
      return FileSpreadsheet;
    case "ppt":
    case "pptx":
      return Presentation;
    default:
      return File;
  }
}

function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(1))} ${sizes[i]}`;
}

function formatDate(timestamp: number): string {
  const date = new Date(timestamp);
  const now = new Date();
  const diffDays = Math.floor((now.getTime() - date.getTime()) / (1000 * 60 * 60 * 24));
  
  if (diffDays === 0) return "Today";
  if (diffDays === 1) return "Yesterday";
  if (diffDays < 7) return `${diffDays} days ago`;
  
  return date.toLocaleDateString(undefined, {
    month: "short",
    day: "numeric",
    year: date.getFullYear() !== now.getFullYear() ? "numeric" : undefined,
  });
}

function getTrustColor(trust?: number | null): string {
  if (trust === null || trust === undefined) return "text-muted-foreground";
  if (trust >= 90) return "text-green-500";
  if (trust >= 70) return "text-yellow-500";
  if (trust >= 50) return "text-orange-500";
  return "text-red-500";
}

export const ObjectRow = ({
  object,
  isSelected,
  isAlternate,
  onClick,
  onDoubleClick,
}: ObjectRowProps) => {
  const Icon = getFileIcon(object.extension);

  return (
    <div
      className={cn(
        "flex items-center h-9 px-4 cursor-pointer",
        "transition-colors duration-100",
        "hover:bg-muted/60",
        isAlternate && "bg-muted/20",
        isSelected && "bg-primary/10 hover:bg-primary/15"
      )}
      onClick={onClick}
      onDoubleClick={onDoubleClick}
    >
      {/* Name */}
      <div className="flex-1 min-w-0 flex items-center gap-2">
        <Icon className="w-4 h-4 flex-shrink-0 text-muted-foreground" />
        <span className="text-sm truncate">{object.name}</span>
      </div>

      {/* Type */}
      <div className="w-20 text-xs text-muted-foreground uppercase">
        {object.extension || "—"}
      </div>

      {/* Size */}
      <div className="w-24 text-xs text-muted-foreground tabular-nums">
        {formatBytes(object.sizeBytes)}
      </div>

      {/* Modified */}
      <div className="w-32 text-xs text-muted-foreground">
        {formatDate(object.modifiedAt)}
      </div>

      {/* Tags */}
      <div className="w-24 flex items-center gap-1 overflow-hidden">
        {object.tags.slice(0, 2).map((tag) => (
          <Badge
            key={`${tag.key}:${tag.value}`}
            variant="secondary"
            className="text-[10px] px-1 py-0 h-4"
          >
            {tag.value}
          </Badge>
        ))}
        {object.tags.length > 2 && (
          <span className="text-[10px] text-muted-foreground">
            +{object.tags.length - 2}
          </span>
        )}
      </div>

      {/* Trust */}
      <div className={cn("w-16 text-right text-xs tabular-nums", getTrustColor(object.trustLevel))}>
        {object.trustLevel !== null && object.trustLevel !== undefined
          ? `${object.trustLevel}%`
          : "—"}
      </div>
    </div>
  );
};
