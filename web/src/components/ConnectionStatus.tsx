import { WifiOff, Wifi, AlertCircle } from 'lucide-react';

interface ConnectionStatusProps {
  isConnected: boolean;
  isError: boolean;
  error?: string;
  lastUpdate: Date | null;
}

export function ConnectionStatus({
  isConnected,
  isError,
  error,
  lastUpdate,
}: ConnectionStatusProps) {
  if (isError) {
    return (
      <div className="alert alert-error">
        <AlertCircle className="w-6 h-6 flex-shrink-0" />
        <div>
          <p className="font-medium">Connection Error</p>
          <p className="text-sm mt-1">
            {error || 'Cannot connect to API server. Make sure the backend is running.'}
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="flex items-center gap-2 text-sm text-gray-500">
      {isConnected ? (
        <>
          <Wifi className="w-4 h-4 text-green-500" />
          <span>Connected</span>
        </>
      ) : (
        <>
          <WifiOff className="w-4 h-4 text-yellow-500" />
          <span>Connecting...</span>
        </>
      )}
      {lastUpdate && (
        <span className="ml-2">
          Last update: {lastUpdate.toLocaleTimeString()}
        </span>
      )}
    </div>
  );
}
