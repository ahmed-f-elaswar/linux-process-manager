import { useState, useCallback } from 'react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { Header } from './components/Header';
import { SystemStats } from './components/SystemStats';
import { ProcessTable } from './components/ProcessTable';
import { ConnectionStatus } from './components/ConnectionStatus';
import { useProcesses, useSystemInfo, useKillProcess, useHealthCheck } from './hooks/useProcesses';
import type { SortColumn, SortConfig } from './types';
import './index.css';

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      retry: 1,
      refetchOnWindowFocus: false,
    },
  },
});

function Dashboard() {
  const [autoRefresh, setAutoRefresh] = useState(true);
  const [refreshInterval, setRefreshInterval] = useState(2000);
  const [sortConfig, setSortConfig] = useState<SortConfig>({
    column: 'cpu',
    ascending: false,
  });
  const [lastUpdate, setLastUpdate] = useState<Date | null>(null);

  // Queries
  const {
    data: processData,
    isLoading: isLoadingProcesses,
    refetch: refetchProcesses,
    isError: isProcessError,
    error: processError,
  } = useProcesses(
    { sortBy: sortConfig.column, ascending: sortConfig.ascending },
    autoRefresh ? refreshInterval : 0
  );

  const {
    data: systemInfo,
    isLoading: isLoadingSystem,
    refetch: refetchSystem,
    isError: isSystemError,
  } = useSystemInfo(autoRefresh ? refreshInterval : 0);

  const { data: health } = useHealthCheck();

  const killProcess = useKillProcess();

  // Handlers
  const handleRefresh = useCallback(() => {
    refetchProcesses();
    refetchSystem();
    setLastUpdate(new Date());
  }, [refetchProcesses, refetchSystem]);

  const handleSort = useCallback((column: SortColumn) => {
    setSortConfig((prev) => ({
      column,
      ascending: prev.column === column ? !prev.ascending : false,
    }));
  }, []);

  const handleKill = useCallback(
    (pid: number, signal: number) => {
      killProcess.mutate(
        { pid, signal },
        {
          onSuccess: (response) => {
            if (response.success) {
              // Show success notification could be added here
              console.log(response.message);
            } else {
              console.error(response.message);
            }
          },
          onError: (error) => {
            console.error('Failed to send signal:', error);
          },
        }
      );
    },
    [killProcess]
  );

  const isConnected = !!health;
  const isError = isProcessError || isSystemError;
  const errorMessage = processError instanceof Error ? processError.message : undefined;

  return (
    <div className="min-h-screen bg-gray-50">
      <Header
        onRefresh={handleRefresh}
        isRefreshing={isLoadingProcesses}
        autoRefresh={autoRefresh}
        setAutoRefresh={setAutoRefresh}
        refreshInterval={refreshInterval}
        setRefreshInterval={setRefreshInterval}
      />

      <main className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-6 space-y-6">
        {/* Connection Status */}
        {isError && (
          <ConnectionStatus
            isConnected={isConnected}
            isError={isError}
            error={errorMessage}
            lastUpdate={lastUpdate}
          />
        )}

        {/* System Stats */}
        <section>
          <h2 className="text-lg font-semibold mb-4 text-gray-800">
            System Overview
          </h2>
          <SystemStats systemInfo={systemInfo} isLoading={isLoadingSystem} />
        </section>

        {/* Process Table */}
        <section>
          <h2 className="text-lg font-semibold mb-4 text-gray-800">
            Processes
          </h2>
          <ProcessTable
            processes={processData?.processes || []}
            isLoading={isLoadingProcesses}
            sortConfig={sortConfig}
            onSort={handleSort}
            onKill={handleKill}
            isKilling={killProcess.isPending}
            cpuCores={systemInfo?.cpu_count || 1}
          />
        </section>

        {/* Footer Status */}
        <footer className="text-center py-4 border-t border-gray-200">
          <ConnectionStatus
            isConnected={isConnected}
            isError={false}
            lastUpdate={lastUpdate}
          />
        </footer>
      </main>
    </div>
  );
}

function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <Dashboard />
    </QueryClientProvider>
  );
}

export default App;
