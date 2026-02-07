import { CompleteScreen } from "@/components/onboarding/CompleteScreen";
import { useState, useEffect } from "react";

const OnboardingCompletePlayground = () => {
  const [isExiting, setIsExiting] = useState(false);
  useEffect(() => {
    setTimeout(() => {
      setIsExiting(true);
    }, 7000);
  }, []);

  return <CompleteScreen isExiting={isExiting} />;
};

export default OnboardingCompletePlayground;
