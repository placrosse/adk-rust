import { useState, useRef, useEffect } from 'react';
import { useStore } from '../../store';
import { useSSE } from '../../hooks/useSSE';

interface Message {
  role: 'user' | 'assistant';
  content: string;
}

type FlowPhase = 'idle' | 'input' | 'output';

interface Props {
  onFlowPhase?: (phase: FlowPhase) => void;
}

export function TestConsole({ onFlowPhase }: Props) {
  const { currentProject } = useStore();
  const [messages, setMessages] = useState<Message[]>([]);
  const [input, setInput] = useState('');
  const { send, cancel, isStreaming, streamingText, currentAgent } = useSSE(currentProject?.id ?? null);
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const sendingRef = useRef(false);

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages, streamingText]);

  // Update flow phase based on streaming state
  useEffect(() => {
    if (streamingText) {
      onFlowPhase?.('output');
    } else if (!isStreaming) {
      onFlowPhase?.('idle');
    }
  }, [streamingText, isStreaming, onFlowPhase]);

  const sendMessage = () => {
    if (!input.trim() || !currentProject || isStreaming || sendingRef.current) return;
    sendingRef.current = true;
    const userMsg = input.trim();
    setInput('');
    setMessages((m) => [...m, { role: 'user', content: userMsg }]);
    onFlowPhase?.('input');
    
    send(
      userMsg,
      (text) => {
        if (text) {
          setMessages((m) => [...m, { role: 'assistant', content: text }]);
        }
        onFlowPhase?.('idle');
        sendingRef.current = false;
      },
      (error) => {
        setMessages((m) => [...m, { role: 'assistant', content: `Error: ${error}` }]);
        onFlowPhase?.('idle');
        sendingRef.current = false;
      }
    );
  };

  const clearChat = async () => {
    if (currentProject) {
      await fetch(`/api/projects/${currentProject.id}/session`, { method: 'DELETE' });
    }
    setMessages([]);
  };

  const handleCancel = () => {
    cancel();
    onFlowPhase?.('idle');
  };

  return (
    <div className="flex flex-col h-full bg-studio-panel border-t border-gray-700">
      <div className="p-2 border-b border-gray-700 text-sm font-semibold flex justify-between">
        <span>💬 Test Console</span>
        <div className="flex gap-2">
          {messages.length > 0 && !isStreaming && (
            <button onClick={clearChat} className="text-gray-400 text-xs hover:text-white">Clear</button>
          )}
          {isStreaming && (
            <button onClick={handleCancel} className="text-red-400 text-xs">Stop</button>
          )}
        </div>
      </div>
      <div className="flex-1 overflow-y-auto p-3 space-y-3">
        {messages.length === 0 && !streamingText && (
          <div className="text-gray-500 text-sm">Send a message to test your agent...</div>
        )}
        {messages.map((m, i) => (
          <div key={i} className={`text-sm ${m.role === 'user' ? 'text-blue-400' : 'text-gray-200'}`}>
            <span className="font-semibold">{m.role === 'user' ? 'You: ' : 'Agent: '}</span>
            <span className="whitespace-pre-wrap">{m.content}</span>
          </div>
        ))}
        {streamingText && (
          <div className="text-sm text-gray-200">
            <span className="font-semibold">{currentAgent || 'Agent'}: </span>
            <span className="whitespace-pre-wrap">{streamingText}</span>
            <span className="animate-pulse">▌</span>
          </div>
        )}
        <div ref={messagesEndRef} />
      </div>
      <div className="p-2 border-t border-gray-700 flex gap-2">
        <input
          type="text"
          value={input}
          onChange={(e) => setInput(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === 'Enter' && !e.repeat) {
              e.preventDefault();
              sendMessage();
            }
          }}
          placeholder="Type a message..."
          className="flex-1 px-3 py-2 bg-studio-bg border border-gray-600 rounded text-sm"
          disabled={isStreaming}
        />
        <button
          onClick={sendMessage}
          disabled={isStreaming || !input.trim()}
          className="px-4 py-2 bg-studio-highlight rounded text-sm disabled:opacity-50"
        >
          Send
        </button>
      </div>
    </div>
  );
}
