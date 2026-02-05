import type { Config } from "tailwindcss";

export default {
  darkMode: ["class"],
  content: ["./pages/**/*.{ts,tsx}", "./components/**/*.{ts,tsx}", "./app/**/*.{ts,tsx}", "./src/**/*.{ts,tsx}"],
  prefix: "",
  theme: {
    container: {
      center: true,
      padding: "2rem",
      screens: {
        "2xl": "1400px",
      },
    },
    extend: {
      colors: {
        border: "hsl(var(--border))",
        input: "hsl(var(--input))",
        ring: "hsl(var(--ring))",
        background: "hsl(var(--background))",
        "background-deep": "hsl(var(--background-deep))",
        foreground: "hsl(var(--foreground))",
        primary: {
          DEFAULT: "hsl(var(--primary))",
          foreground: "hsl(var(--primary-foreground))",
          glow: "hsl(var(--primary-glow))",
        },
        secondary: {
          DEFAULT: "hsl(var(--secondary))",
          foreground: "hsl(var(--secondary-foreground))",
        },
        destructive: {
          DEFAULT: "hsl(var(--destructive))",
          foreground: "hsl(var(--destructive-foreground))",
        },
        muted: {
          DEFAULT: "hsl(var(--muted))",
          foreground: "hsl(var(--muted-foreground))",
        },
        accent: {
          DEFAULT: "hsl(var(--accent))",
          foreground: "hsl(var(--accent-foreground))",
        },
        warning: {
          DEFAULT: "hsl(var(--warning))",
          foreground: "hsl(var(--warning-foreground))",
        },
        popover: {
          DEFAULT: "hsl(var(--popover))",
          foreground: "hsl(var(--popover-foreground))",
        },
        card: {
          DEFAULT: "hsl(var(--card))",
          foreground: "hsl(var(--card-foreground))",
        },
        glass: {
          bg: "hsl(var(--glass-bg))",
          border: "hsl(var(--glass-border))",
        },
        node: {
          primary: "hsl(var(--node-primary))",
          secondary: "hsl(var(--node-secondary))",
          file: "hsl(var(--node-file))",
        },
        connection: {
          line: "hsl(var(--connection-line))",
        },
        sidebar: {
          DEFAULT: "hsl(var(--sidebar-background))",
          foreground: "hsl(var(--sidebar-foreground))",
          primary: "hsl(var(--sidebar-primary))",
          "primary-foreground": "hsl(var(--sidebar-primary-foreground))",
          accent: "hsl(var(--sidebar-accent))",
          "accent-foreground": "hsl(var(--sidebar-accent-foreground))",
          border: "hsl(var(--sidebar-border))",
          ring: "hsl(var(--sidebar-ring))",
        },
      },
      borderRadius: {
        lg: "var(--radius)",
        md: "calc(var(--radius) - 2px)",
        sm: "calc(var(--radius) - 4px)",
        xl: "calc(var(--radius) + 4px)",
        "2xl": "calc(var(--radius) + 8px)",
      },
      fontFamily: {
        sans: [
          "SF Pro Text",
          "SF Pro Display",
          "-apple-system",
          "BlinkMacSystemFont",
          "Inter",
          "system-ui",
          "sans-serif",
        ],
        display: [
          "SF Pro Display",
          "SF Pro Text",
          "-apple-system",
          "BlinkMacSystemFont",
          "Inter",
          "system-ui",
          "sans-serif",
        ],
      },
      letterSpacing: {
        tight: "-0.02em",
        tighter: "-0.04em",
      },
      keyframes: {
        "accordion-down": {
          from: { height: "0" },
          to: {
            height:
              "var(--radix-accordion-content-height, var(--radix-collapsible-content-height))",
          },
        },
        "accordion-up": {
          from: {
            height:
              "var(--radix-accordion-content-height, var(--radix-collapsible-content-height))",
          },
          to: { height: "0" },
        },
        "fade-up": {
          from: { opacity: "0", transform: "translateY(30px)" },
          to: { opacity: "1", transform: "translateY(0)" },
        },
        "fade-in": {
          from: { opacity: "0" },
          to: { opacity: "1" },
        },
        "fade-out": {
          from: { opacity: "1" },
          to: { opacity: "0" },
        },
        "scale-in": {
          from: { opacity: "0", transform: "scale(0.9)" },
          to: { opacity: "1", transform: "scale(1)" },
        },
        "scale-out": {
          from: { opacity: "1", transform: "scale(1)" },
          to: { opacity: "0", transform: "scale(0.9)" },
        },
        "slide-in-right": {
          from: { opacity: "0", transform: "translateX(100px)" },
          to: { opacity: "1", transform: "translateX(0)" },
        },
        "slide-in-left": {
          from: { opacity: "0", transform: "translateX(-100px)" },
          to: { opacity: "1", transform: "translateX(0)" },
        },
        "slide-in-up": {
          from: { opacity: "0", transform: "translateY(20px)" },
          to: { opacity: "1", transform: "translateY(0)" },
        },
        float: {
          "0%, 100%": { transform: "translateY(0px)" },
          "50%": { transform: "translateY(-10px)" },
        },
        "pulse-glow": {
          "0%, 100%": { opacity: "0.5", transform: "scale(1)" },
          "50%": { opacity: "1", transform: "scale(1.05)" },
        },
        "draw-line": {
          from: { strokeDashoffset: "1000" },
          to: { strokeDashoffset: "0" },
        },
        shimmer: {
          "0%": { backgroundPosition: "-200% 0" },
          "100%": { backgroundPosition: "200% 0" },
        },
        "spin-slow": {
          from: { transform: "rotate(0deg)" },
          to: { transform: "rotate(360deg)" },
        },
        "particle-float": {
          "0%, 100%": { transform: "translate(0, 0) scale(1)", opacity: "0.3" },
          "25%": { transform: "translate(10px, -20px) scale(1.1)", opacity: "0.6" },
          "50%": { transform: "translate(-5px, -40px) scale(0.9)", opacity: "0.4" },
          "75%": { transform: "translate(15px, -20px) scale(1.05)", opacity: "0.5" },
        },
        "connection-pulse": {
          "0%, 100%": { strokeOpacity: "0.3" },
          "50%": { strokeOpacity: "0.8" },
        },
        "node-pulse": {
          "0%, 100%": { 
            transform: "scale(1)",
            boxShadow: "0 0 0 0 hsl(var(--primary) / 0.4)",
          },
          "50%": { 
            transform: "scale(1.02)",
            boxShadow: "0 0 20px 10px hsl(var(--primary) / 0)",
          },
        },
        celebration: {
          "0%": { transform: "scale(0) rotate(0deg)", opacity: "1" },
          "50%": { transform: "scale(1.2) rotate(180deg)", opacity: "0.8" },
          "100%": { transform: "scale(0) rotate(360deg)", opacity: "0" },
        },
        "lattice-expand": {
          from: { 
            opacity: "0",
            transform: "scale(0)",
          },
          to: { 
            opacity: "1",
            transform: "scale(1)",
          },
        },
      },
      animation: {
        "accordion-down": "accordion-down 0.2s ease-out",
        "accordion-up": "accordion-up 0.2s ease-out",
        "fade-up": "fade-up 0.8s cubic-bezier(0.16, 1, 0.3, 1) forwards",
        "fade-in": "fade-in 0.5s ease-out forwards",
        "fade-out": "fade-out 0.3s ease-out forwards",
        "scale-in": "scale-in 0.5s cubic-bezier(0.16, 1, 0.3, 1) forwards",
        "scale-out": "scale-out 0.3s ease-out forwards",
        "slide-in-right": "slide-in-right 0.6s cubic-bezier(0.16, 1, 0.3, 1) forwards",
        "slide-in-left": "slide-in-left 0.6s cubic-bezier(0.16, 1, 0.3, 1) forwards",
        "slide-in-up": "slide-in-up 0.5s cubic-bezier(0.16, 1, 0.3, 1) forwards",
        float: "float 6s ease-in-out infinite",
        "pulse-glow": "pulse-glow 2s ease-in-out infinite",
        "draw-line": "draw-line 1.5s ease-out forwards",
        shimmer: "shimmer 2s linear infinite",
        "spin-slow": "spin-slow 20s linear infinite",
        "particle-float": "particle-float 8s ease-in-out infinite",
        "connection-pulse": "connection-pulse 2s ease-in-out infinite",
        "node-pulse": "node-pulse 2s ease-in-out infinite",
        celebration: "celebration 0.8s ease-out forwards",
        "lattice-expand": "lattice-expand 1s cubic-bezier(0.16, 1, 0.3, 1) forwards",
      },
      transitionTimingFunction: {
        "out-expo": "cubic-bezier(0.16, 1, 0.3, 1)",
        "out-back": "cubic-bezier(0.34, 1.56, 0.64, 1)",
        "in-out-circ": "cubic-bezier(0.85, 0, 0.15, 1)",
      },
      backdropBlur: {
        xs: "2px",
      },
    },
  },
  plugins: [require("tailwindcss-animate")],
} satisfies Config;
