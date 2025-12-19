interface CanvasToolbarProps {
  onAutoLayout: () => void;
  onFitView: () => void;
}

export function CanvasToolbar({ onAutoLayout, onFitView }: CanvasToolbarProps) {
  return (
    <div className="absolute top-2 left-2 z-10 flex gap-2">
      <button
        onClick={onAutoLayout}
        className="px-3 py-1.5 bg-gray-800 hover:bg-gray-700 border border-gray-600 rounded text-sm flex items-center gap-2 text-gray-200"
        title="Auto Layout (Ctrl+L)"
      >
        <span>⊞</span> Layout
      </button>
      <button
        onClick={onFitView}
        className="px-3 py-1.5 bg-gray-800 hover:bg-gray-700 border border-gray-600 rounded text-sm flex items-center gap-2 text-gray-200"
        title="Fit to View (Ctrl+0)"
      >
        <span>⊡</span> Fit
      </button>
    </div>
  );
}
