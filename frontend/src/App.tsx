import { Toaster } from "@/components/ui/toaster";
import { Toaster as Sonner } from "@/components/ui/sonner";
import { TooltipProvider } from "@/components/ui/tooltip";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { BrowserRouter, Routes, Route, Navigate } from "react-router-dom";
import { AuthProvider, useAuth } from "@/providers/AuthProvider";
import { ThemeProvider } from "@/providers/ThemeProvider";
import Index from "./pages/Index";
import NotFound from "./pages/NotFound";
import DashboardLayout from "./pages/DashboardLayout";
import InboxPage from "./pages/InboxPage";
import UsersPage from "./pages/UsersPage";
import BroadcastPage from "./pages/BroadcastPage";
import SettingsPage from "./pages/SettingsPage";
import LoginPage from "./pages/LoginPage";
import BlockedUsersPage from "./pages/BlockedUsersPage";

const queryClient = new QueryClient();

/** Redirects unauthenticated users to the landing page. Shows loading spinner while auth loads. */
function ProtectedRoute({ children }: { children: React.ReactNode }) {
  const { isAuthenticated, isLoading } = useAuth();
  
  if (isLoading) {
    return (
      <div className="flex min-h-screen items-center justify-center bg-background">
        <div className="text-center">
          <div className="inline-block h-8 w-8 animate-spin rounded-full border-4 border-solid border-primary border-r-transparent"></div>
          <p className="mt-4 text-sm text-muted-foreground font-mono">Loading...</p>
        </div>
      </div>
    );
  }
  
  if (!isAuthenticated) return <Navigate to="/" replace />;
  return <>{children}</>;
}

/** Redirects already-authenticated users away from the landing/login pages. */
function PublicOnlyRoute({ children }: { children: React.ReactNode }) {
  const { isAuthenticated, isLoading } = useAuth();
  
  if (isLoading) {
    return (
      <div className="flex min-h-screen items-center justify-center bg-background">
        <div className="text-center">
          <div className="inline-block h-8 w-8 animate-spin rounded-full border-4 border-solid border-primary border-r-transparent"></div>
          <p className="mt-4 text-sm text-muted-foreground font-mono">Loading...</p>
        </div>
      </div>
    );
  }
  
  if (isAuthenticated) return <Navigate to="/dashboard/inbox" replace />;
  return <>{children}</>;
}

const App = () => (
  <QueryClientProvider client={queryClient}>
    <ThemeProvider>
      <AuthProvider>
        <TooltipProvider>
          <Toaster />
          <Sonner />
          <BrowserRouter>
            <Routes>
              <Route path="/" element={<PublicOnlyRoute><Index /></PublicOnlyRoute>} />
              <Route path="/login" element={<PublicOnlyRoute><LoginPage /></PublicOnlyRoute>} />
              <Route path="/dashboard" element={<ProtectedRoute><DashboardLayout /></ProtectedRoute>}>
                <Route index element={<Navigate to="/dashboard/inbox" replace />} />
                <Route path="inbox" element={<InboxPage />} />
                <Route path="users" element={<UsersPage />} />
                <Route path="broadcasts" element={<BroadcastPage />} />
                <Route path="settings" element={<SettingsPage />} />
                <Route path="blocked" element={<BlockedUsersPage />} />
              </Route>
              <Route path="*" element={<NotFound />} />
            </Routes>
          </BrowserRouter>
        </TooltipProvider>
      </AuthProvider>
    </ThemeProvider>
  </QueryClientProvider>
);

export default App;
