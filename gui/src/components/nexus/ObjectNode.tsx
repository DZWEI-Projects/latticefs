import { cn } from "@/lib/utils";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import type { ObjectInfo } from "@/lib/lfs";

interface ObjectNodeProps {
  object: ObjectInfo;
  position: { x: number; y: number };
  isSelected: boolean;
  isHovered: boolean;
  isConnected: boolean;
  onHover: () => void;
  onLeave: () => void;
  onClick: (e: React.MouseEvent) => void;
  onDoubleClick: () => void;
}

function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(1))} ${sizes[i]}`;
}

function getExtensionColor(ext?: string | null): string {
  if (!ext) return "bg-muted";
  switch (ext.toLowerCase()) {
    case "pdf":
      return "bg-red-500/20 border-red-500/40";
    case "doc":
    case "docx":
      return "bg-blue-500/20 border-blue-500/40";
    case "xls":
    case "xlsx":
      return "bg-green-500/20 border-green-500/40";
    case "ppt":
    case "pptx":
      return "bg-orange-500/20 border-orange-500/40";
    case "jpg":
    case "jpeg":
    case "png":
    case "gif":
      return "bg-purple-500/20 border-purple-500/40";
    case "mp4":
    case "mov":
    case "avi":
      return "bg-pink-500/20 border-pink-500/40";
    case "mp3":
    case "wav":
    case "flac":
      return "bg-cyan-500/20 border-cyan-500/40";
    case "zip":
    case "rar":
    case "7z":
      return "bg-yellow-500/20 border-yellow-500/40";
    case "exe":
    case "dmg":
      return "bg-red-600/20 border-red-600/40";
    case "py":
    case "js":
    case "ts":
    case "rs":
      return "bg-emerald-500/20 border-emerald-500/40";
    case "md":
    case "txt":
      return "bg-slate-500/20 border-slate-500/40";
    default:
      return "bg-muted border-muted-foreground/20";
  }
}

export const ObjectNode = ({
  object,
  position,
  isSelected,
  isHovered,
  isConnected,
  onHover,
  onLeave,
  onClick,
  onDoubleClick,
}: ObjectNodeProps) => {
  const isHighlighted = isSelected || isHovered || isConnected;

  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <div
          className={cn(
            "absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2",
            "transition-all duration-200 ease-out cursor-pointer z-20"
          )}
          style={{
            transform: `translate(calc(-50% + ${position.x}px), calc(-50% + ${position.y}px)) scale(${isHighlighted ? 1.15 : 1})`,
          }}
          onMouseEnter={onHover}
          onMouseLeave={onLeave}
          onClick={onClick}
          onDoubleClick={onDoubleClick}
        >
          <div
            className={cn(
              "w-9 h-9 rounded-lg border flex items-center justify-center",
              "text-[10px] font-medium uppercase tracking-tight",
              "transition-all duration-200",
              getExtensionColor(object.extension),
              isSelected && "ring-2 ring-primary ring-offset-2 ring-offset-background",
              isHovered && "shadow-lg",
              isConnected && !isHovered && "opacity-90"
            )}
          >
            {object.extension?.slice(0, 3) || "?"}
          </div>
          
          {/* Name label on hover */}
          {isHighlighted && (
            <span className="absolute -bottom-5 left-1/2 -translate-x-1/2 text-[10px] text-foreground whitespace-nowrap max-w-[100px] truncate">
              {object.name}
            </span>
          )}
        </div>
      </TooltipTrigger>
      <TooltipContent side="right" className="max-w-[240px]">
        <div className="space-y-1">
          <p className="font-medium truncate">{object.name}</p>
          <p className="text-xs text-muted-foreground">
            {formatBytes(object.sizeBytes)}
          </p>
          {object.tags.length > 0 && (
            <div className="flex flex-wrap gap-1 pt-1">
              {object.tags.slice(0, 3).map((tag) => (
                <span
                  key={`${tag.key}:${tag.value}`}
                  className="text-[10px] px-1.5 py-0.5 rounded bg-muted"
                >
                  {tag.key}:{tag.value}
                </span>
              ))}
            </div>
          )}
          {object.views.length > 0 && (
            <p className="text-[10px] text-muted-foreground pt-1">
              In {object.views.length} {object.views.length === 1 ? "view" : "views"}
            </p>
          )}
        </div>
      </TooltipContent>
    </Tooltip>
  );
};
