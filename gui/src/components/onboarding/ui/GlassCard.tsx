import { cn } from "@/lib/utils";
import { ReactNode } from "react";

interface GlassCardProps {
  children: ReactNode;
  className?: string;
  hover?: boolean;
  glow?: boolean;
  delay?: number;
  onClick?: () => void;
  tabIndex?: number;
}

export const GlassCard = ({
  children,
  className,
  hover = true,
  glow = false,
  delay = 0,
  onClick,
  tabIndex = 0,
}: GlassCardProps) => {
  return (
    <div
      role="button"
      tabIndex={tabIndex}
      className={cn(
        "glass rounded-xl p-4 opacity-0 animate-fade-up",
        hover && "transition-all duration-400 ease-out-expo hover:scale-[1.01] hover:border-primary/25",
        glow && "glow-primary",
        className
      )}
      style={{ animationDelay: `${delay}ms`, animationFillMode: "forwards" }}
      onClick={onClick}
    >
      {children}
    </div>
  );
};
