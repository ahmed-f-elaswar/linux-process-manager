// Process information from the API
export interface ProcessInfo {
  pid: number;
  ppid: number;
  name: string;
  user: string;
  cpu_usage: number;
  memory_usage: number;
  memory_percent: number;
  state: string;
  command: string;
  start_time: number;
  network_connections: number | null;
  is_container: boolean;
  container_id: string | null;
  gpu_memory: number | null;
}

// Process list response
export interface ProcessListResponse {
  processes: ProcessInfo[];
  total: number;
  filtered: number;
}

// System information
export interface SystemInfo {
  cpu_count: number;
  total_memory: number;
  used_memory: number;
  total_swap: number;
  used_swap: number;
  uptime: number;
  load_average: LoadAverage;
}

export interface LoadAverage {
  one: number;
  five: number;
  fifteen: number;
}

// Kill request/response
export interface KillRequest {
  pid: number;
  signal?: number;
}

export interface KillResponse {
  success: boolean;
  message: string;
}

// History record
export interface HistoryRecord {
  timestamp: string;
  cpu_usage: number;
  memory_usage: number;
}

// Health check
export interface HealthResponse {
  status: string;
  timestamp: string;
}

// Sort configuration
export type SortColumn = 'pid' | 'name' | 'user' | 'cpu' | 'memory' | 'start_time';

export interface SortConfig {
  column: SortColumn;
  ascending: boolean;
}

// Filter configuration
export interface FilterConfig {
  search: string;
  user: string;
  showContainers: boolean;
  showGpu: boolean;
  minCpu: number;
  minMemory: number;
}

// Tree node for process tree view
export interface ProcessTreeNode extends ProcessInfo {
  children: ProcessTreeNode[];
  level: number;
  expanded: boolean;
}

// Available signals
export const SIGNALS = [
  { value: 15, label: 'SIGTERM', description: 'Graceful termination' },
  { value: 9, label: 'SIGKILL', description: 'Forceful termination' },
  { value: 1, label: 'SIGHUP', description: 'Hangup' },
  { value: 2, label: 'SIGINT', description: 'Interrupt' },
  { value: 19, label: 'SIGSTOP', description: 'Stop process' },
  { value: 18, label: 'SIGCONT', description: 'Continue process' },
  { value: 10, label: 'SIGUSR1', description: 'User signal 1' },
  { value: 12, label: 'SIGUSR2', description: 'User signal 2' },
  { value: 3, label: 'SIGQUIT', description: 'Quit' },
] as const;

// Chart data point
export interface ChartDataPoint {
  time: string;
  value: number;
  label?: string;
}
