import { useState } from 'react';
import { Skull, MoreVertical, X } from 'lucide-react';
import { SIGNALS } from '../types';

interface ProcessActionsProps {
  pid: number;
  name: string;
  onKill: (pid: number, signal: number) => void;
  isKilling: boolean;
}

export function ProcessActions({ pid, name, onKill, isKilling }: ProcessActionsProps) {
  const [showMenu, setShowMenu] = useState(false);
  const [showConfirm, setShowConfirm] = useState(false);
  const [selectedSignal, setSelectedSignal] = useState(15); // Default SIGTERM

  const handleKill = () => {
    setShowConfirm(false);
    setShowMenu(false);
    onKill(pid, selectedSignal);
  };

  return (
    <div className="relative">
      <button
        onClick={(e) => {
          e.stopPropagation();
          setShowMenu(!showMenu);
        }}
        className="btn btn-ghost btn-icon"
      >
        <MoreVertical className="w-4 h-4 text-gray-500" />
      </button>

      {showMenu && (
        <>
          {/* Backdrop */}
          <div
            className="fixed inset-0 z-40"
            onClick={() => setShowMenu(false)}
          />

          {/* Menu */}
          <div className="absolute right-0 mt-1 w-56 dropdown-content z-50">
            <div className="py-1">
              <div className="px-3 py-2 text-xs font-semibold text-gray-500 uppercase">
                Send Signal
              </div>
              {SIGNALS.map((signal) => (
                <button
                  key={signal.value}
                  onClick={() => {
                    setSelectedSignal(signal.value);
                    setShowConfirm(true);
                    setShowMenu(false);
                  }}
                  className="dropdown-item w-full justify-between"
                >
                  <span className="text-gray-700">{signal.label}</span>
                  <span className="text-xs text-gray-500">
                    {signal.description}
                  </span>
                </button>
              ))}
            </div>
          </div>
        </>
      )}

      {/* Confirmation Modal */}
      {showConfirm && (
        <>
          <div className="dialog-overlay" onClick={() => setShowConfirm(false)} />
          <div className="dialog-content">
            <div className="flex items-center justify-between mb-4">
              <h3 className="dialog-title">Confirm Action</h3>
              <button
                onClick={() => setShowConfirm(false)}
                className="btn btn-ghost btn-icon"
              >
                <X className="w-5 h-5" />
              </button>
            </div>

            <div className="mb-6">
              <div className="alert alert-error mb-4">
                <Skull className="w-8 h-8 flex-shrink-0" />
                <div>
                  <p className="font-medium">
                    Send <strong>{SIGNALS.find((s) => s.value === selectedSignal)?.label}</strong> to process?
                  </p>
                  <p className="text-sm mt-1">
                    PID: {pid} - {name}
                  </p>
                </div>
              </div>

              <p className="text-sm text-gray-500">
                {SIGNALS.find((s) => s.value === selectedSignal)?.description}
              </p>
            </div>

            <div className="flex gap-3 justify-end">
              <button
                onClick={() => setShowConfirm(false)}
                className="btn btn-secondary"
              >
                Cancel
              </button>
              <button
                onClick={handleKill}
                disabled={isKilling}
                className="btn btn-destructive flex items-center gap-2"
              >
                <Skull className="w-4 h-4" />
                {isKilling ? 'Sending...' : 'Send Signal'}
              </button>
            </div>
          </div>
        </>
      )}
    </div>
  );
}
