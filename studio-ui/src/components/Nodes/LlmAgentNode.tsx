import { memo } from 'react';
import { Handle, Position } from '@xyflow/react';
import { ThoughtBubble } from '../Overlays/ThoughtBubble';

interface LlmNodeData {
  label: string;
  model?: string;
  tools?: string[];
  isActive?: boolean;
  thought?: string;
}

interface Props {
  data: LlmNodeData;
  selected?: boolean;
}

export const LlmAgentNode = memo(({ data, selected }: Props) => {
  const isActive = data.isActive || false;
  
  return (
    <div className="relative">
      <div 
        className="rounded-lg min-w-[160px] transition-all duration-200"
        style={{ 
          background: '#1e3a5f',
          border: `2px solid ${isActive ? '#4ade80' : '#60a5fa'}`,
          boxShadow: isActive ? '0 0 20px rgba(74, 222, 128, 0.5)' : selected ? '0 0 0 2px #3b82f6' : 'none',
        }}
      >
        <Handle type="target" position={Position.Top} className="!bg-gray-400" />
        
        <div className="px-3 py-2">
          <div className="flex items-center gap-2 font-medium text-white text-sm">
            <span>🤖</span>
            <span>{data.label}</span>
            {isActive && <span className="ml-auto text-green-400 animate-pulse">●</span>}
          </div>
          <div className="mt-2 border-t border-white/20 pt-2">
            <div className="text-xs text-gray-300">{data.model || 'gemini-2.0-flash'}</div>
            {data.tools && data.tools.length > 0 && (
              <div className="flex flex-wrap gap-1 mt-1">
                {data.tools.map(t => (
                  <span key={t} className="px-1.5 py-0.5 bg-black/30 rounded text-[10px] text-gray-300">{t}</span>
                ))}
              </div>
            )}
          </div>
        </div>
        
        <Handle type="source" position={Position.Bottom} className="!bg-gray-400" />
      </div>
      
      {data.thought && <ThoughtBubble text={data.thought} streaming={isActive} />}
    </div>
  );
});

LlmAgentNode.displayName = 'LlmAgentNode';
