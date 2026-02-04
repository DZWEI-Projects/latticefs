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
  size = "lg",
}: AnimatedButtonProps) => {
  const [isPressed, setIsPressed] = useState(false);

  const sizeClasses = {
    sm: "px-4 py-2 text-sm",
    md: "px-6 py-3 text-base",
    lg: "px-8 py-4 text-lg",
  };

  const variantClasses = {
    primary: cn(
      "bg-primary text-primary-foreground",
      "hover:bg-primary/90 hover:glow-primary",
      "active:scale-[0.98]"
    ),
    secondary: cn(
      "bg-secondary/20 text-secondary-foreground border border-secondary/30",
      "hover:bg-secondary/30 hover:border-secondary/50",
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
        "relative group flex items-center justify-center gap-3",
        "rounded-xl font-medium tracking-tight",
        "transition-all duration-300 ease-out-expo",
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
            "w-5 h-5 transition-transform duration-300 ease-out-expo",
            "group-hover:translate-x-1"
          )}
        />
      )}
      
      {/* Glow effect on hover */}
      <div 
        className={cn(
          "absolute inset-0 rounded-xl opacity-0 transition-opacity duration-300",
          "bg-gradient-to-r from-primary/20 via-primary/10 to-primary/20",
          "group-hover:opacity-100",
          "pointer-events-none"
        )}
      />
    </button>
  );
};
