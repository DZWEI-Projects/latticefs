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
import type { ObjectInfo } from "@/lib/lfs";

interface ObjectCardProps {
  object: ObjectInfo;
  isSelected: boolean;
  onClick: (e: React.MouseEvent) => void;
  onDoubleClick: () => void;
  onContextMenu?: (e: React.MouseEvent) => void;
}

function getFileIcon(ext?: string | null) {
  if (!ext) return File;
  switch (ext.toLowerCase()) {
    case "pdf":
    case "doc":
    case "docx":
    case "txt":
    case "md":
    case "rtf":
      return FileText;
    case "jpg":
    case "jpeg":
    case "png":
    case "gif":
    case "webp":
    case "svg":
    case "bmp":
      return FileImage;
    case "mp4":
    case "mov":
    case "avi":
    case "mkv":
    case "webm":
      return FileVideo;
    case "mp3":
    case "wav":
    case "flac":
    case "aac":
    case "ogg":
      return FileAudio;
    case "zip":
    case "rar":
    case "7z":
    case "tar":
    case "gz":
      return FileArchive;
    case "py":
    case "js":
    case "ts":
    case "tsx":
    case "jsx":
    case "rs":
    case "go":
    case "java":
    case "cpp":
    case "c":
    case "h":
      return FileCode;
    case "xls":
    case "xlsx":
    case "csv":
      return FileSpreadsheet;
    case "ppt":
    case "pptx":
    case "key":
      return Presentation;
    default:
      return File;
  }
}

function getIconColor(ext?: string | null): string {
  if (!ext) return "text-muted-foreground";
  switch (ext.toLowerCase()) {
    case "pdf":
      return "text-red-500";
    case "doc":
    case "docx":
      return "text-blue-500";
    case "xls":
    case "xlsx":
      return "text-green-500";
    case "ppt":
    case "pptx":
      return "text-orange-500";
    case "jpg":
    case "jpeg":
    case "png":
    case "gif":
      return "text-purple-500";
    case "mp4":
    case "mov":
      return "text-pink-500";
    case "mp3":
    case "wav":
      return "text-cyan-500";
    case "zip":
    case "rar":
      return "text-yellow-500";
    case "py":
    case "js":
    case "ts":
    case "rs":
      return "text-emerald-500";
    default:
      return "text-muted-foreground";
  }
}

export const ObjectCard = ({
  object,
  isSelected,
  onClick,
  onDoubleClick,
  onContextMenu,
}: ObjectCardProps) => {
  const Icon = getFileIcon(object.extension);
  const iconColor = getIconColor(object.extension);

  return (
    <div
      className={cn(
        "flex flex-col items-center gap-1.5 p-3 rounded-lg cursor-pointer",
        "transition-all duration-150",
        "hover:bg-muted/60",
        isSelected && "bg-primary/10 ring-1 ring-primary/50"
      )}
      onClick={onClick}
      onDoubleClick={onDoubleClick}
      onContextMenu={onContextMenu}
    >
      <div
        className={cn(
          "w-12 h-12 rounded-lg flex items-center justify-center",
          "bg-muted/50"
        )}
      >
        <Icon className={cn("w-7 h-7", iconColor)} />
      </div>
      
      <span className="text-xs text-center truncate w-full px-1" title={object.name}>
        {object.name}
      </span>
      
      {object.extension && (
        <span className="text-[10px] text-muted-foreground uppercase">
          {object.extension}
        </span>
      )}
    </div>
  );
};
