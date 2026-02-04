import { OnboardingContainer } from "@/components/onboarding/OnboardingContainer";

const Index = () => {
  const handleOnboardingComplete = () => {
    console.log("Onboarding complete!");
    // In a real app, this would navigate to the main application
  };

  return <OnboardingContainer onComplete={handleOnboardingComplete} />;
};

export default Index;
