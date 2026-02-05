import { useParams, useNavigate, useSearchParams } from "react-router-dom";
import { useCallback, useEffect } from "react";
import { NexusLayout } from "@/components/nexus/NexusLayout";

const Nexus = () => {
  const { viewName } = useParams<{ viewName?: string }>();
  const navigate = useNavigate();
  const [searchParams] = useSearchParams();

  // Default to "recent" view if no view is specified
  const currentViewName = viewName || "recent";

  const handleViewChange = useCallback(
    (newViewName: string) => {
      // Preserve any query params (like view mode)
      const params = searchParams.toString();
      const url = `/nexus/${newViewName}${params ? `?${params}` : ""}`;
      navigate(url);
    },
    [navigate, searchParams]
  );

  // Redirect to default view if on base /nexus path
  useEffect(() => {
    if (!viewName) {
      navigate("/nexus/recent", { replace: true });
    }
  }, [viewName, navigate]);

  return (
    <NexusLayout
      currentViewName={currentViewName}
      onViewChange={handleViewChange}
    />
  );
};

export default Nexus;
