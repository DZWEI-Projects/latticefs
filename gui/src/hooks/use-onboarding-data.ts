import { useCallback, useEffect, useState } from "react";
import {
  assignProject,
  fetchOnboardingData,
  fetchSettings,
  updateSettings,
  type OnboardingSettings,
} from "@/lib/latticeApi";
import { mapOnboardingData, type OnboardingState } from "@/data/onboardingData";

type UseOnboardingData = {
  data: OnboardingState | null;
  isLoading: boolean;
  error: string | null;
  refresh: () => Promise<void>;
  assignToProject: (objectId: string, projectId: string) => Promise<void>;
  settings: OnboardingSettings | null;
  updateSecuritySettings: (next: OnboardingSettings) => Promise<void>;
};

export const useOnboardingData = (): UseOnboardingData => {
  const [data, setData] = useState<OnboardingState | null>(null);
  const [settings, setSettings] = useState<OnboardingSettings | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    setIsLoading(true);
    setError(null);
    try {
      const [onboarding, storedSettings] = await Promise.all([
        fetchOnboardingData(),
        fetchSettings(),
      ]);
      setData(mapOnboardingData(onboarding));
      setSettings(storedSettings);
    } catch (err) {
      if (err instanceof Error) {
        setError(err.message);
      } else {
        setError("Onboarding-Daten konnten nicht geladen werden.");
      }
    } finally {
      setIsLoading(false);
    }
  }, []);

  const assignToProject = useCallback(async (objectId: string, projectId: string) => {
    setError(null);
    try {
      const updated = await assignProject(objectId, projectId);
      setData(mapOnboardingData(updated));
    } catch (err) {
      if (err instanceof Error) {
        setError(err.message);
      } else {
        setError("Projekt konnte nicht zugewiesen werden.");
      }
    }
  }, []);

  const updateSecuritySettings = useCallback(async (next: OnboardingSettings) => {
    setError(null);
    try {
      const updated = await updateSettings(next);
      setSettings(updated);
    } catch (err) {
      if (err instanceof Error) {
        setError(err.message);
      } else {
        setError("Einstellungen konnten nicht gespeichert werden.");
      }
    }
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  return { data, isLoading, error, refresh, assignToProject, settings, updateSecuritySettings };
};
