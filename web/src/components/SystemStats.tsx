import { Cpu, HardDrive, Activity, Clock, Server } from 'lucide-react';
import type { SystemInfo } from '../types';

interface SystemStatsProps {
  systemInfo: SystemInfo | undefined;
  isLoading: boolean;
}

function formatBytes(bytes: number): string {
  const gb = bytes / 1024 / 1024 / 1024;
  if (gb >= 1) {
    return `${gb.toFixed(1)} GB`;
  }
  const mb = bytes / 1024 / 1024;
  return `${mb.toFixed(0)} MB`;
}

function formatUptime(seconds: number): string {
  const days = Math.floor(seconds / 86400);
  const hours = Math.floor((seconds % 86400) / 3600);
  const mins = Math.floor((seconds % 3600) / 60);

  if (days > 0) {
    return `${days}d ${hours}h ${mins}m`;
  }
  if (hours > 0) {
    return `${hours}h ${mins}m`;
  }
  return `${mins}m`;
}

function StatCard({
  icon: Icon,
  label,
  value,
  subValue,
  color,
  isLoading,
}: {
  icon: typeof Cpu;
  label: string;
  value: string;
  subValue?: string;
  color: string;
  isLoading: boolean;
}) {
  return (
    <div className="stat-card">
      <div className="flex items-center gap-3 mb-2">
        <div className={`p-2 rounded-lg ${color}`}>
          <Icon className="w-5 h-5 text-white" />
        </div>
        <span className="stat-label">{label}</span>
      </div>
      {isLoading ? (
        <div className="skeleton h-8 w-24" />
      ) : (
        <>
          <div className="stat-value">{value}</div>
          {subValue && <div className="stat-description">{subValue}</div>}
        </>
      )}
    </div>
  );
}

function ProgressBar({ value, max, color }: { value: number; max: number; color: string }) {
  const percentage = max > 0 ? (value / max) * 100 : 0;

  return (
    <div className="progress">
      <div
        className={`progress-bar ${color}`}
        style={{ width: `${Math.min(percentage, 100)}%` }}
      />
    </div>
  );
}

export function SystemStats({ systemInfo, isLoading }: SystemStatsProps) {
  const memoryPercent = systemInfo
    ? ((systemInfo.used_memory / systemInfo.total_memory) * 100).toFixed(1)
    : '0';

  const swapPercent = systemInfo && systemInfo.total_swap > 0
    ? ((systemInfo.used_swap / systemInfo.total_swap) * 100).toFixed(1)
    : '0';

  return (
    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 xl:grid-cols-5 gap-4">
      {/* CPU Cores */}
      <StatCard
        icon={Cpu}
        label="CPU Cores"
        value={systemInfo?.cpu_count.toString() || '-'}
        color="bg-blue-600"
        isLoading={isLoading}
      />

      {/* Load Average */}
      <StatCard
        icon={Activity}
        label="Load Average"
        value={
          systemInfo
            ? `${systemInfo.load_average.one.toFixed(2)}`
            : '-'
        }
        subValue={
          systemInfo
            ? `5m: ${systemInfo.load_average.five.toFixed(2)} / 15m: ${systemInfo.load_average.fifteen.toFixed(2)}`
            : undefined
        }
        color="bg-purple-600"
        isLoading={isLoading}
      />

      {/* Memory Usage */}
      <div className="stat-card">
        <div className="flex items-center gap-3 mb-2">
          <div className="p-2 rounded-lg bg-green-600">
            <HardDrive className="w-5 h-5 text-white" />
          </div>
          <span className="stat-label">Memory</span>
        </div>
        {isLoading ? (
          <div className="skeleton h-8 w-24" />
        ) : (
          <>
            <div className="stat-value">{memoryPercent}%</div>
            <div className="stat-description">
              {systemInfo ? `${formatBytes(systemInfo.used_memory)} / ${formatBytes(systemInfo.total_memory)}` : '-'}
            </div>
            <ProgressBar
              value={systemInfo?.used_memory || 0}
              max={systemInfo?.total_memory || 1}
              color="progress-bar-green"
            />
          </>
        )}
      </div>

      {/* Swap Usage */}
      <div className="stat-card">
        <div className="flex items-center gap-3 mb-2">
          <div className="p-2 rounded-lg bg-orange-500">
            <Server className="w-5 h-5 text-white" />
          </div>
          <span className="stat-label">Swap</span>
        </div>
        {isLoading ? (
          <div className="skeleton h-8 w-24" />
        ) : (
          <>
            <div className="stat-value">{swapPercent}%</div>
            <div className="stat-description">
              {systemInfo && systemInfo.total_swap > 0
                ? `${formatBytes(systemInfo.used_swap)} / ${formatBytes(systemInfo.total_swap)}`
                : 'No swap'}
            </div>
            {systemInfo && systemInfo.total_swap > 0 && (
              <ProgressBar
                value={systemInfo.used_swap}
                max={systemInfo.total_swap}
                color="progress-bar-yellow"
              />
            )}
          </>
        )}
      </div>

      {/* Uptime */}
      <StatCard
        icon={Clock}
        label="Uptime"
        value={systemInfo ? formatUptime(systemInfo.uptime) : '-'}
        color="bg-cyan-600"
        isLoading={isLoading}
      />
    </div>
  );
}
