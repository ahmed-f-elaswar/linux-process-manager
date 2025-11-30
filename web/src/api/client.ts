import type {
  ProcessListResponse,
  SystemInfo,
  ProcessInfo,
  KillRequest,
  KillResponse,
  HealthResponse,
  HistoryRecord,
  SortColumn,
} from '../types';

// API base URL - uses relative path so it works with both dev proxy and production
const API_BASE = '/api';

// Generic fetch wrapper with error handling
async function fetchApi<T>(endpoint: string, options?: RequestInit): Promise<T> {
  const url = `${API_BASE}${endpoint}`;

  try {
    const response = await fetch(url, {
      headers: {
        'Content-Type': 'application/json',
        ...options?.headers,
      },
      ...options,
    });

    if (!response.ok) {
      const error = await response.json().catch(() => ({ error: response.statusText }));
      throw new Error(error.error || `HTTP error ${response.status}`);
    }

    return response.json();
  } catch (error) {
    if (error instanceof TypeError && error.message.includes('fetch')) {
      throw new Error('Cannot connect to API server. Make sure the backend is running.');
    }
    throw error;
  }
}

// Health check
export async function checkHealth(): Promise<HealthResponse> {
  return fetchApi<HealthResponse>('/health');
}

// Get system information
export async function getSystemInfo(): Promise<SystemInfo> {
  return fetchApi<SystemInfo>('/system');
}

// Get process list
export interface GetProcessesParams {
  sortBy?: SortColumn;
  ascending?: boolean;
  user?: string;
  name?: string;
  limit?: number;
}

export async function getProcesses(params?: GetProcessesParams): Promise<ProcessListResponse> {
  const searchParams = new URLSearchParams();

  if (params?.sortBy) searchParams.set('sort_by', params.sortBy);
  if (params?.ascending !== undefined) searchParams.set('ascending', String(params.ascending));
  if (params?.user) searchParams.set('user', params.user);
  if (params?.name) searchParams.set('name', params.name);
  if (params?.limit) searchParams.set('limit', String(params.limit));

  const query = searchParams.toString();
  return fetchApi<ProcessListResponse>(`/processes${query ? `?${query}` : ''}`);
}

// Get single process
export async function getProcess(pid: number): Promise<ProcessInfo> {
  return fetchApi<ProcessInfo>(`/processes/${pid}`);
}

// Kill a process
export async function killProcess(request: KillRequest): Promise<KillResponse> {
  return fetchApi<KillResponse>('/processes/kill', {
    method: 'POST',
    body: JSON.stringify(request),
  });
}

// Get process history
export interface GetHistoryParams {
  pid: number;
  start?: string;
  end?: string;
  limit?: number;
}

export async function getProcessHistory(params: GetHistoryParams): Promise<HistoryRecord[]> {
  const searchParams = new URLSearchParams();

  searchParams.set('pid', String(params.pid));
  if (params.start) searchParams.set('start', params.start);
  if (params.end) searchParams.set('end', params.end);
  if (params.limit) searchParams.set('limit', String(params.limit));

  return fetchApi<HistoryRecord[]>(`/history/processes?${searchParams.toString()}`);
}

// Export all API functions
export const api = {
  checkHealth,
  getSystemInfo,
  getProcesses,
  getProcess,
  killProcess,
  getProcessHistory,
};
