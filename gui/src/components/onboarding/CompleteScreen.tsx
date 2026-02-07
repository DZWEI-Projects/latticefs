import { motion } from "motion/react";

// const MARK_ASPECT_RATIO = 475.183 / 346.557;
const MARK_ASPECT_RATIO = 1;

interface CompleteScreenProps {
  isExiting: boolean;
}

export const CompleteScreen = ({ isExiting }: CompleteScreenProps) => {
  return (
    <motion.div 
      className="relative flex items-center justify-center min-h-screen overflow-hidden"
      initial={{ opacity: 0 }}
      animate={isExiting ? { opacity: 0 } : { opacity: 1 }}
      transition={{ 
        duration: 2.2,
        delay: isExiting ? 0 : 0.35,
        ease: [0.16, 1, 0.3, 1] 
      }}
    >
      <motion.div
        className="absolute inset-0 pointer-events-none"
        initial={{ opacity: 0, scale: 0.9 }}
        animate={isExiting ? { opacity: 0, scale: 1.08 } : { opacity: 1, scale: 1 }}
        transition={{ duration: 2.0, ease: [0.16, 1, 0.3, 1] }}
        style={{
          background:
            "radial-gradient(circle at center, hsl(var(--primary) / 0.2) 0%, hsl(var(--background) / 0) 65%)",
        }}
      />
      <motion.div
        className="absolute left-1/2 top-[18%] h-64 w-64 -translate-x-1/2 rounded-full pointer-events-none"
        initial={{ opacity: 0, scale: 0.75, filter: "blur(70px)" }}
        animate={
          isExiting
            ? { opacity: 0, scale: 1.08, filter: "blur(95px)" }
            : { opacity: 0.6, scale: 1, filter: "blur(88px)" }
        }
        transition={{ duration: isExiting ? 0.95 : 2.8, ease: [0.2, 1, 0.22, 1] }}
        style={{
          background:
            "radial-gradient(circle at center, hsl(var(--primary) / 0.4) 0%, hsl(var(--primary) / 0.08) 50%, transparent 80%)",
        }}
      />
      <motion.div 
        className="text-center relative z-10 grid justify-items-center"
        initial="hidden"
        animate={isExiting ? "exit" : "visible"}
        variants={{
          visible: {
            transition: {
              staggerChildren: 1.00,
              delayChildren: 0.9,
            },
          },
          exit: {
            transition: {
              staggerChildren: 0.18,
              staggerDirection: -1,
            },
          },
        }}
      >
        <motion.div
          className="relative w-[8.5rem] md:w-[10rem] col-start-1 row-start-1 rounded-full pointer-events-none"
          style={{ aspectRatio: MARK_ASPECT_RATIO }}
          initial={{ opacity: 0, scale: 0.82, filter: "blur(36px)" }}
          animate={isExiting ? { opacity: 0, scale: 1.1, filter: "blur(40px)" } : { opacity: 0.66, scale: 1, filter: "blur(30px)" }}
          transition={{ duration: isExiting ? 0.95 : 2.1, ease: [0.2, 1, 0.22, 1] }}
        >
          <div
            className="absolute inset-[12%_10%_15%_10%] rounded-full"
            style={{
              background:
                "radial-gradient(circle at center, hsl(var(--primary) / 0.42) 0%, hsl(var(--primary) / 0.12) 48%, transparent 84%)",
            }}
          />
        </motion.div>
        <motion.div
          className="relative w-[8.5rem] md:w-[10rem] col-start-1 row-start-1"
          style={{ aspectRatio: MARK_ASPECT_RATIO }}
          variants={{
            hidden: {
              opacity: 0,
              y: 14,
              scale: 0.98,
              filter: "blur(8px)",
            },
            visible: {
              opacity: 1,
              y: 0,
              scale: 1,
              filter: "blur(0px)",
              transition: {
                duration: 0.6,
                ease: [0.22, 1, 0.24, 1],
              },
            },
            exit: {
              opacity: 0,
              y: -10,
              scale: 1.04,
              filter: "blur(10px)",
              transition: {
                duration: 0.6,
                ease: [0.7, 0, 0.84, 0],
              },
            },
          }}
        >
          <motion.svg
            className="absolute overflow-visible"
            style={{
              left: "27.07%",
              top: "0%",
              width: "72.93%",
              height: "100%",
            }}
            viewBox="0 0 100 100"
            fill="none"
          >
            <motion.circle
              cx="50"
              cy="50"
              r="46"
              stroke="white"
              strokeWidth="3.6"
              strokeLinecap="round"
              className="drop-shadow-[0_0_18px_hsl(var(--primary)/0.28)]"
              style={{ rotate: -90, transformOrigin: "50% 50%" }}
              variants={{
                hidden: {
                  pathLength: 0,
                  opacity: 0.25,
                },
                visible: {
                  pathLength: 1,
                  opacity: 1,
                  transition: {
                    pathLength: {
                      duration: 2.6,
                      ease: [0.2, 1, 0.22, 1],
                    },
                    opacity: {
                      duration: 1.2,
                      ease: [0.2, 1, 0.22, 1],
                    },
                  },
                },
                exit: {
                  pathLength: 1,
                  opacity: 0,
                  transition: {
                    duration: 0.5,
                    ease: [0.7, 0, 0.84, 0],
                  },
                },
              }}
            />
          </motion.svg>
        </motion.div>

        <motion.div
          className="relative w-[8.5rem] md:w-[10rem] col-start-1 row-start-1"
          style={{ aspectRatio: MARK_ASPECT_RATIO }}
          variants={{
            hidden: {
              opacity: 0,
              x: 30,
              y: 0,
              scale: 0.92,
              filter: "blur(16px)",
            },
            visible: {
              opacity: 1,
              x: 0,
              y: 0,
              scale: 1,
              filter: "blur(0px)",
              transition: {
                duration: 1.85,
                ease: [0.22, 1, 0.24, 1],
              },
            },
            exit: {
              opacity: 0,
              x: 0,
              y: -41,
              scale: 0.8,
              filter: "blur(12px)",
              transition: {
                duration: 1.05,
                ease: [0.7, 0, 0.84, 0],
              },
            },
          }}
        >
          <motion.div
            className="absolute rounded-full pointer-events-none"
            style={{
              left: "8%",
              top: "18%",
              width: "34%",
              height: "34%",
              background:
                "radial-gradient(circle at center, hsl(var(--primary) / 0.95) 0%, hsl(var(--primary) / 0.38) 40%, transparent 78%)",
            }}
            variants={{
              hidden: { opacity: 0, scale: 0.45, filter: "blur(4px)" },
              visible: {
                opacity: 0.9,
                scale: 1,
                filter: "blur(8px)",
                transition: { duration: 1.6, ease: [0.2, 1, 0.22, 1] },
              },
              exit: {
                opacity: 0,
                scale: 1,
                filter: "blur(12px)",
                transition: { duration: 0.7, ease: [0.7, 0, 0.84, 0] },
              },
            }}
          />
          <motion.img
            src="/stern.svg"
            alt="NeuralFS Stern"
            className="absolute object-contain drop-shadow-[0_0_42px_hsl(var(--primary)/0.34)]"
            style={{
              left: "0%",
              top: "10.99%",
              width: "86.67%",
              height: "77.93%",
            }}
            variants={{
              hidden: {
                opacity: 0.72,
                scale: 0.9,
                filter: "blur(3px)",
              },
              visible: {
                opacity: 1,
                scale: 1,
                filter: "blur(0px)",
                transition: {
                  duration: 2.0,
                  ease: [0.2, 1, 0.22, 1],
                },
              },
              exit: {
                opacity: 0,
                scale: 1,
                filter: "blur(8px)",
                transition: {
                  duration: 0.95,
                  ease: [0.7, 0, 0.84, 0],
                },
              },
            }}
          />
        </motion.div>

        <motion.div
          className="col-start-1 row-start-2 mt-6 mx-auto w-[13rem] md:w-[15rem] aspect-[804/66]"
          initial={{
            opacity: 0,
            y: 24,
            scale: 0.98,
            filter: "blur(14px)",
          }}
          animate={isExiting ? {
            opacity: 0,
            y: -16,
            scale: 0.99,
            filter: "blur(10px)",
          } : {
            opacity: 1,
            y: 0,
            scale: 1,
            filter: "blur(0px)",
          }}
          transition={{
            duration: isExiting ? 0.85 : 1.45,
            delay: isExiting ? 0.18 : 1.9,
            ease: isExiting ? [0.7, 0, 0.84, 0] : [0.2, 1, 0.22, 1],
          }}
        >
          <motion.img
            src="/neural-wordmark.svg"
            alt="NeuralFS Wordmark"
            className="w-full h-full object-contain drop-shadow-[0_0_14px_hsl(var(--foreground)/0.14)]"
            initial={{ scale: 1.06 }}
            animate={isExiting ? { scale: 1.03, opacity: 0.68 } : { scale: 1, opacity: 1 }}
            transition={{ duration: isExiting ? 0.85 : 1.6, ease: [0.2, 1, 0.22, 1] }}
          />
        </motion.div>

        <motion.div
          className="col-start-1 row-start-3 mt-14"
          variants={{
            hidden: {
              opacity: 0,
              y: 26,
              filter: "blur(14px)",
            },
            visible: {
              opacity: 1,
              y: 0,
              filter: "blur(0px)",
              transition: {
                staggerChildren: 0.28,
                delayChildren: 0.28,
              },
            },
            exit: {
              opacity: 0,
              y: -18,
              filter: "blur(12px)",
              transition: {
                staggerChildren: 0.1,
                staggerDirection: -1,
              },
            },
          }}
        >
          <motion.h1
            className="text-3xl font-bold tracking-tighter mb-3 text-foreground"
            variants={{
              hidden: {
                opacity: 0,
                y: 22,
                scale: 0.98,
                filter: "blur(11px)",
              },
              visible: {
                opacity: 1,
                y: 0,
                scale: 1,
                filter: "blur(0px)",
                transition: {
                  type: "spring",
                  damping: 22,
                  stiffness: 38,
                  mass: 1.35,
                },
              },
              exit: {
                opacity: 0,
                y: -18,
                scale: 0.99,
                filter: "blur(9px)",
                transition: {
                  duration: 0.78,
                  ease: [0.7, 0, 0.84, 0],
                },
              },
            }}
          >
            Willkommen in deinem Workspace
          </motion.h1>
          <motion.p
            className="text-muted-foreground text-[18px]"
            variants={{
              hidden: {
                opacity: 0,
                y: 18,
                filter: "blur(10px)",
              },
              visible: {
                opacity: 1,
                y: 0,
                filter: "blur(0px)",
                transition: {
                  type: "spring",
                  damping: 24,
                  stiffness: 36,
                  mass: 1.3,
                },
              },
              exit: {
                opacity: 0,
                y: -14,
                filter: "blur(8px)",
                transition: {
                  duration: 0.72,
                  ease: [0.7, 0, 0.84, 0],
                },
              },
            }}
          >
            NeuralFS ist bereit.
          </motion.p>
        </motion.div>
      </motion.div>
    </motion.div>
  );
};
