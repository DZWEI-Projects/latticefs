import { cn } from "@/lib/utils";
import { ReactNode, useState } from "react";
import { ArrowRight } from "lucide-react";

interface AnimatedButtonProps {
  children: ReactNode;
  onClick?: () => void;
  className?: string;
  disabled?: boolean;
  showArrow?: boolean;
  variant?: "primary" | "secondary" | "ghost";
  size?: "sm" | "md" | "lg";
}

export const AnimatedButton = ({
  children,
  onClick,
  className,
  disabled = false,
  showArrow = true,
  variant = "primary",
  size = "md",
}: AnimatedButtonProps) => {
  const [isPressed, setIsPressed] = useState(false);

  const sizeClasses = {
    sm: "px-3.5 py-1.5 text-xs",
    md: "px-5 py-2.5 text-sm",
    lg: "px-6 py-3 text-base",
  };

  const variantClasses = {
    primary: cn(
      "bg-primary text-primary-foreground border border-primary/30",
      "hover:bg-primary/90 hover:border-primary/40",
      "shadow-[inset_0_1px_0_hsl(var(--primary)/0.35)]",
      "active:scale-[0.98] active:shadow-none"
    ),
    secondary: cn(
      "bg-secondary/15 text-secondary-foreground border border-secondary/25",
      "hover:bg-secondary/25 hover:border-secondary/40",
      "active:scale-[0.98]"
    ),
    ghost: cn(
      "bg-transparent text-muted-foreground",
      "hover:text-foreground hover:bg-muted/20",
      "active:scale-[0.98]"
    ),
  };

  return (
    <button
      onClick={onClick}
      disabled={disabled}
      onMouseDown={() => setIsPressed(true)}
      onMouseUp={() => setIsPressed(false)}
      onMouseLeave={() => setIsPressed(false)}
      className={cn(
        "relative group flex items-center justify-center gap-2",
        "rounded-lg font-medium tracking-tight",
        "transition-all duration-200 ease-out-expo",
        "disabled:opacity-50 disabled:cursor-not-allowed",
        "focus:outline-none focus:ring-2 focus:ring-primary/50 focus:ring-offset-2 focus:ring-offset-background",
        sizeClasses[size],
        variantClasses[variant],
        isPressed && "scale-[0.98]",
        className
      )}
    >
      <span className="relative z-10">{children}</span>
      {showArrow && (
        <ArrowRight 
          className={cn(
            "w-4 h-4 transition-transform duration-300 ease-out-expo",
            "group-hover:translate-x-1"
          )}
        />
      )}
      
      {/* Glow effect on hover */}
      <div 
        className={cn(
          "absolute inset-0 rounded-lg opacity-0 transition-opacity duration-300",
          "bg-gradient-to-r from-primary/20 via-primary/10 to-primary/20",
          "group-hover:opacity-100",
          "pointer-events-none"
        )}
      />
    </button>
  );
};
