import { cn } from "@/lib/utils";

interface LogoProps {
  className?: string;
  animate?: boolean;
  showWordmark?: boolean;
}

export const Logo = ({ className, animate = true, showWordmark = true }: LogoProps) => {
  return (
    <div 
      className={cn(
        "relative flex items-center justify-center",
        animate && "opacity-0 animate-scale-in",
        className
      )}
      style={{ animationDelay: animate ? "200ms" : "0ms", animationFillMode: "forwards" }}
    >
      {/* Logo glow effect */}
      <div 
        className={cn(
          "absolute inset-0 rounded-full blur-xl",
          animate && "animate-pulse-glow"
        )}
        style={{
          background: "radial-gradient(circle, hsl(var(--primary) / 0.3) 0%, transparent 70%)",
        }}
      />
      
      <img
        src="/neural.svg"
        alt="NeuralFS Logo"
        className={cn(
          "w-16 h-16 relative z-10 object-contain drop-shadow-[0_0_32px_hsl(var(--primary)/0.35)]",
          animate && "animate-pulse-glow"
        )}
      />
      
      {/* Text */}
      {showWordmark && (
        <span 
          className={cn(
            "absolute -bottom-6 text-lg font-semibold tracking-tight text-foreground",
            animate && "opacity-0 animate-fade-up"
          )}
          style={{ animationDelay: animate ? "400ms" : "0ms", animationFillMode: "forwards" }}
        >
          NeuralFS
        </span>
      )}
    </div>
  );
};
