import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { api, type GetProcessesParams } from '../api/client';
import type { KillRequest } from '../types';

// Hook for fetching processes
export function useProcesses(params?: GetProcessesParams, refetchInterval = 2000) {
  return useQuery({
    queryKey: ['processes', params],
    queryFn: () => api.getProcesses(params),
    refetchInterval,
    staleTime: 1000,
  });
}

// Hook for fetching system info
export function useSystemInfo(refetchInterval = 2000) {
  return useQuery({
    queryKey: ['systemInfo'],
    queryFn: api.getSystemInfo,
    refetchInterval,
    staleTime: 1000,
  });
}

// Hook for fetching a single process
export function useProcess(pid: number) {
  return useQuery({
    queryKey: ['process', pid],
    queryFn: () => api.getProcess(pid),
    enabled: pid > 0,
  });
}

// Hook for killing a process
export function useKillProcess() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (request: KillRequest) => api.killProcess(request),
    onSuccess: () => {
      // Invalidate processes query to refetch the list
      queryClient.invalidateQueries({ queryKey: ['processes'] });
    },
  });
}

// Hook for process history
export function useProcessHistory(pid: number, enabled = true) {
  return useQuery({
    queryKey: ['processHistory', pid],
    queryFn: () => api.getProcessHistory({ pid, limit: 100 }),
    enabled: enabled && pid > 0,
    staleTime: 5000,
  });
}

// Hook for health check
export function useHealthCheck() {
  return useQuery({
    queryKey: ['health'],
    queryFn: api.checkHealth,
    refetchInterval: 30000,
    retry: false,
  });
}
