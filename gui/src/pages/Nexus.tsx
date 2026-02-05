import { useParams, useNavigate, useSearchParams } from "react-router-dom";
import { useCallback, useEffect } from "react";
import { NexusLayout } from "@/components/nexus/NexusLayout";

const Nexus = () => {
  const { viewId } = useParams<{ viewId?: string }>();
  const navigate = useNavigate();
  const [searchParams] = useSearchParams();

  // Default to "recent" view if no view is specified
  const currentViewId = viewId || "recent";

  const handleViewChange = useCallback(
    (newViewId: string) => {
      // Preserve any query params (like view mode)
      const params = searchParams.toString();
      const url = `/nexus/${newViewId}${params ? `?${params}` : ""}`;
      navigate(url);
    },
    [navigate, searchParams]
  );

  // Redirect to default view if on base /nexus path
  useEffect(() => {
    if (!viewId) {
      navigate("/nexus/recent", { replace: true });
    }
  }, [viewId, navigate]);

  return (
    <NexusLayout
      currentViewId={currentViewId}
      onViewChange={handleViewChange}
    />
  );
};

export default Nexus;
