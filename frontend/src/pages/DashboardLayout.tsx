import { Outlet } from "react-router-dom";
import DashboardSidebar from "@/components/DashboardSidebar";
import { useRealtimeEvents } from "@/hooks/useRealtimeEvents";
import { OnboardingTour } from "@/components/OnboardingTour";

const DashboardLayout = () => {
  // Mount SSE connection once for the entire dashboard — updates inbox + broadcasts in real time
  useRealtimeEvents();

  return (
    <div className="flex min-h-screen bg-background">
      <DashboardSidebar />
      <main className="flex-1 overflow-y-auto p-8">
        <Outlet />
      </main>
      <OnboardingTour />
    </div>
  );
};

export default DashboardLayout;
