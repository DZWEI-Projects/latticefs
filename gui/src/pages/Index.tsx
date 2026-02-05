import { useState, useEffect } from "react";
import { useNavigate } from "react-router-dom";
import { OnboardingContainer } from "@/components/onboarding/OnboardingContainer";
import { checkInitialized } from "@/lib/lfs";

const ONBOARDING_COMPLETE_KEY = "lfs-onboarding-complete";

const Index = () => {
  const navigate = useNavigate();
  const [showOnboarding, setShowOnboarding] = useState<boolean | null>(null);

  useEffect(() => {
    const checkShouldSkip = async () => {
      // Fast path: check localStorage first
      if (localStorage.getItem(ONBOARDING_COMPLETE_KEY) === "true") {
        navigate("/nexus", { replace: true });
        return;
      }
      
      // Slow path: check if LFS is already initialized on disk
      const initialized = await checkInitialized();
      if (initialized) {
        localStorage.setItem(ONBOARDING_COMPLETE_KEY, "true");
        navigate("/nexus", { replace: true });
        return;
      }
      
      setShowOnboarding(true);
    };
    
    checkShouldSkip();
  }, [navigate]);

  const handleOnboardingComplete = () => {
    console.log("Onboarding complete!");
    // Navigation is handled by OnboardingContainer
  };

  // Show nothing while checking (or a loading spinner)
  if (showOnboarding === null) return null;
  
  return <OnboardingContainer onComplete={handleOnboardingComplete} />;
};

export default Index;
