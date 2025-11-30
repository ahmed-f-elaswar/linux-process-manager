import { useState } from 'react';
import {
  ArrowUpDown,
  ArrowUp,
  ArrowDown,
  Search,
  X,
  Box,
  Cpu,
  Skull,
} from 'lucide-react';
import type { ProcessInfo, SortColumn, SortConfig } from '../types';
import { ProcessActions } from './ProcessActions';

interface ProcessTableProps {
  processes: ProcessInfo[];
  isLoading: boolean;
  sortConfig: SortConfig;
  onSort: (column: SortColumn) => void;
  onKill: (pid: number, signal: number) => void;
  isKilling: boolean;
  cpuCores: number;
}

function formatMemory(bytes: number): string {
  if (bytes >= 1024 * 1024 * 1024) {
    return `${(bytes / 1024 / 1024 / 1024).toFixed(1)} GB`;
  }
  if (bytes >= 1024 * 1024) {
    return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
  }
  if (bytes >= 1024) {
    return `${(bytes / 1024).toFixed(1)} KB`;
  }
  return `${bytes} B`;
}

function getCpuColor(cpu: number): string {
  if (cpu >= 80) return 'text-red-600';
  if (cpu >= 50) return 'text-orange-500';
  if (cpu >= 20) return 'text-yellow-600';
  return 'text-gray-700';
}

function getMemoryColor(percent: number): string {
  if (percent >= 50) return 'text-red-600';
  if (percent >= 25) return 'text-orange-500';
  if (percent >= 10) return 'text-yellow-600';
  return 'text-gray-700';
}

function SortIcon({ column, sortConfig }: { column: SortColumn; sortConfig: SortConfig }) {
  if (sortConfig.column !== column) {
    return <ArrowUpDown className="w-4 h-4 text-gray-400" />;
  }
  return sortConfig.ascending ? (
    <ArrowUp className="w-4 h-4 text-blue-600" />
  ) : (
    <ArrowDown className="w-4 h-4 text-blue-600" />
  );
}

export function ProcessTable({
  processes,
  isLoading,
  sortConfig,
  onSort,
  onKill,
  isKilling,
  cpuCores,
}: ProcessTableProps) {
  const [search, setSearch] = useState('');
  const [selectedPid, setSelectedPid] = useState<number | null>(null);

  const filteredProcesses = processes.filter((p) => {
    if (!search) return true;
    const searchLower = search.toLowerCase();
    return (
      p.name.toLowerCase().includes(searchLower) ||
      p.command.toLowerCase().includes(searchLower) ||
      p.user.toLowerCase().includes(searchLower) ||
      p.pid.toString().includes(search)
    );
  });

  const columns: { key: SortColumn; label: string; className?: string; title?: string }[] = [
    { key: 'pid', label: 'PID', className: 'w-20' },
    { key: 'name', label: 'Name' },
    { key: 'user', label: 'User', className: 'w-28' },
    { key: 'cpu', label: 'CPU %', className: 'w-24 text-right', title: 'CPU usage as percentage of total system capacity' },
    { key: 'memory', label: 'Memory', className: 'w-28 text-right' },
  ];

  return (
    <div className="card overflow-hidden">
      {/* Search Bar */}
      <div className="p-4 border-b border-gray-200">
        <div className="relative">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-400" />
          <input
            type="text"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            placeholder="Search processes by name, PID, user, or command..."
            className="input w-full pl-10 pr-10"
          />
          {search && (
            <button
              onClick={() => setSearch('')}
              className="absolute right-3 top-1/2 -translate-y-1/2 text-gray-400 hover:text-gray-600"
            >
              <X className="w-4 h-4" />
            </button>
          )}
        </div>
        <div className="mt-2 text-sm text-gray-500">
          Showing {filteredProcesses.length} of {processes.length} processes
        </div>
      </div>

      {/* Table */}
      <div className="table-container">
        <table className="table">
          <thead>
            <tr>
              {columns.map((col) => (
                <th
                  key={col.key}
                  onClick={() => onSort(col.key)}
                  className={`sortable ${col.className || ''}`}
                  title={col.title}
                >
                  <div className="flex items-center gap-2">
                    {col.label}
                    <SortIcon column={col.key} sortConfig={sortConfig} />
                  </div>
                </th>
              ))}
              <th className="w-32">Status</th>
              <th>Command</th>
              <th className="w-24 text-center">Actions</th>
            </tr>
          </thead>
          <tbody>
            {isLoading ? (
              // Loading skeleton
              Array.from({ length: 10 }).map((_, i) => (
                <tr key={i}>
                  {Array.from({ length: 8 }).map((_, j) => (
                    <td key={j}>
                      <div className="skeleton h-4 w-full" />
                    </td>
                  ))}
                </tr>
              ))
            ) : filteredProcesses.length === 0 ? (
              <tr>
                <td colSpan={8} className="text-center py-8 text-gray-500">
                  {search ? 'No processes match your search' : 'No processes found'}
                </td>
              </tr>
            ) : (
              filteredProcesses.map((process) => (
                <tr
                  key={process.pid}
                  className={selectedPid === process.pid ? 'bg-blue-50' : ''}
                  onClick={() => setSelectedPid(process.pid)}
                >
                  <td className="font-mono text-sm">{process.pid}</td>
                  <td>
                    <div className="flex items-center gap-2">
                      <span className="font-medium text-gray-900">
                        {process.name}
                      </span>
                      {process.is_container && (
                        <span className="badge badge-info">
                          <Box className="w-3 h-3 mr-1" />
                          Container
                        </span>
                      )}
                      {process.gpu_memory && process.gpu_memory > 0 && (
                        <span className="badge badge-warning">
                          <Cpu className="w-3 h-3 mr-1" />
                          GPU
                        </span>
                      )}
                    </div>
                  </td>
                  <td className="text-gray-600">{process.user}</td>
                  <td className={`text-right font-mono ${getCpuColor(process.cpu_usage / cpuCores)}`}>
                    {(process.cpu_usage / cpuCores).toFixed(1)}%
                  </td>
                  <td className={`text-right font-mono ${getMemoryColor(process.memory_percent)}`}>
                    {formatMemory(process.memory_usage)}
                  </td>
                  <td>
                    <StatusBadge state={process.state} />
                  </td>
                  <td className="max-w-xs">
                    <div
                      className="text-gray-500 text-sm truncate"
                      title={process.command}
                    >
                      {process.command || '-'}
                    </div>
                  </td>
                  <td className="text-center">
                    <ProcessActions
                      pid={process.pid}
                      name={process.name}
                      onKill={onKill}
                      isKilling={isKilling}
                    />
                  </td>
                </tr>
              ))
            )}
          </tbody>
        </table>
      </div>
    </div>
  );
}

function StatusBadge({ state }: { state: string }) {
  const stateMap: Record<string, { class: string; label: string }> = {
    R: { class: 'badge-success', label: 'Running' },
    S: { class: 'badge-info', label: 'Sleeping' },
    D: { class: 'badge-warning', label: 'Disk Wait' },
    Z: { class: 'badge-danger', label: 'Zombie' },
    T: { class: 'badge-warning', label: 'Stopped' },
    t: { class: 'badge-warning', label: 'Traced' },
    X: { class: 'badge-danger', label: 'Dead' },
    I: { class: 'badge-info', label: 'Idle' },
  };

  const status = stateMap[state] || { class: 'badge-info', label: state || 'Unknown' };

  return (
    <span className={`badge ${status.class}`}>
      {status.label === 'Zombie' && <Skull className="w-3 h-3 mr-1" />}
      {status.label}
    </span>
  );
}
