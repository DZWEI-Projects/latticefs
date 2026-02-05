import { cn } from "@/lib/utils";

interface LogoProps {
  className?: string;
  animate?: boolean;
}

export const Logo = ({ className, animate = true }: LogoProps) => {
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
      
      {/* Logo SVG - Neural network inspired design */}
      <svg
        viewBox="0 0 80 80"
        className="w-16 h-16 relative z-10"
        fill="none"
        xmlns="http://www.w3.org/2000/svg"
      >
        {/* Central node */}
        <circle
          cx="40"
          cy="40"
          r="8"
          fill="hsl(var(--primary))"
          className={animate ? "animate-pulse-glow" : ""}
        />
        
        {/* Orbiting nodes */}
        <g className={animate ? "animate-spin-slow" : ""} style={{ transformOrigin: "40px 40px" }}>
          <circle cx="40" cy="16" r="4" fill="hsl(var(--secondary))" />
          <circle cx="60" cy="52" r="4" fill="hsl(var(--secondary))" />
          <circle cx="20" cy="52" r="4" fill="hsl(var(--secondary))" />
        </g>
        
        {/* Connection lines */}
        <g stroke="hsl(var(--primary) / 0.4)" strokeWidth="1.5">
          <line x1="40" y1="32" x2="40" y2="20" className={animate ? "animate-connection-pulse" : ""} />
          <line x1="46" y1="45" x2="56" y2="50" className={animate ? "animate-connection-pulse" : ""} style={{ animationDelay: "0.3s" }} />
          <line x1="34" y1="45" x2="24" y2="50" className={animate ? "animate-connection-pulse" : ""} style={{ animationDelay: "0.6s" }} />
        </g>
        
        {/* Outer ring */}
        <circle
          cx="40"
          cy="40"
          r="32"
          stroke="hsl(var(--primary) / 0.2)"
          strokeWidth="1"
          fill="none"
          strokeDasharray="4 4"
          className={animate ? "animate-spin-slow" : ""}
          style={{ transformOrigin: "40px 40px", animationDirection: "reverse", animationDuration: "30s" }}
        />
      </svg>
      
      {/* Text */}
      <span 
        className={cn(
          "absolute -bottom-6 text-lg font-semibold tracking-tight text-foreground",
          animate && "opacity-0 animate-fade-up"
        )}
        style={{ animationDelay: animate ? "400ms" : "0ms", animationFillMode: "forwards" }}
      >
        NeuralFS
      </span>
    </div>
  );
};
