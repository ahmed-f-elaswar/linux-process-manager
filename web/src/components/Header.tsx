import { RefreshCw, Activity, Settings, Monitor } from 'lucide-react';
import { useState } from 'react';

interface HeaderProps {
  onRefresh: () => void;
  isRefreshing: boolean;
  autoRefresh: boolean;
  setAutoRefresh: (value: boolean) => void;
  refreshInterval: number;
  setRefreshInterval: (value: number) => void;
}

export function Header({
  onRefresh,
  isRefreshing,
  autoRefresh,
  setAutoRefresh,
  refreshInterval,
  setRefreshInterval,
}: HeaderProps) {
  const [showSettings, setShowSettings] = useState(false);

  return (
    <header className="bg-white border-b border-gray-200 sticky top-0 z-40">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div className="flex items-center justify-between h-16">
          {/* Logo */}
          <div className="flex items-center gap-3">
            <div className="flex items-center justify-center w-10 h-10 bg-blue-600 rounded-lg">
              <Monitor className="w-6 h-6 text-white" />
            </div>
            <div>
              <h1 className="text-lg font-semibold text-gray-900">
                Process Manager
              </h1>
              <p className="text-xs text-gray-500">
                Real-time monitoring
              </p>
            </div>
          </div>

          {/* Controls */}
          <div className="flex items-center gap-3">
            {/* Auto Refresh */}
            <div className="flex items-center gap-2">
              <Activity
                className={`w-4 h-4 ${autoRefresh ? 'text-green-500' : 'text-gray-400'}`}
              />
              <button
                onClick={() => setAutoRefresh(!autoRefresh)}
                className={`btn btn-sm ${autoRefresh ? 'btn-primary' : 'btn-outline'}`}
              >
                Auto: {autoRefresh ? 'ON' : 'OFF'}
              </button>
            </div>

            {/* Refresh */}
            <button
              onClick={onRefresh}
              disabled={isRefreshing}
              className="btn btn-secondary flex items-center gap-2"
            >
              <RefreshCw className={`w-4 h-4 ${isRefreshing ? 'animate-spin' : ''}`} />
              Refresh
            </button>

            {/* Settings */}
            <div className="relative">
              <button
                onClick={() => setShowSettings(!showSettings)}
                className="btn btn-ghost btn-icon"
              >
                <Settings className="w-5 h-5 text-gray-500" />
              </button>

              {showSettings && (
                <>
                  <div
                    className="fixed inset-0 z-40"
                    onClick={() => setShowSettings(false)}
                  />
                  <div className="absolute right-0 mt-2 w-64 dropdown-content z-50">
                    <div className="p-3">
                      <h3 className="font-medium text-sm text-gray-900 mb-3">Settings</h3>
                      <label className="block text-sm text-gray-600 mb-1">
                        Refresh Interval
                      </label>
                      <select
                        value={refreshInterval}
                        onChange={(e) => setRefreshInterval(Number(e.target.value))}
                        className="input w-full"
                      >
                        <option value={1000}>1 second</option>
                        <option value={2000}>2 seconds</option>
                        <option value={5000}>5 seconds</option>
                        <option value={10000}>10 seconds</option>
                        <option value={30000}>30 seconds</option>
                      </select>
                    </div>
                  </div>
                </>
              )}
            </div>
          </div>
        </div>
      </div>
    </header>
  );
}
