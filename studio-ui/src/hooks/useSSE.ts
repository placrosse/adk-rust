import { useCallback, useRef, useState } from 'react';

interface ToolCall {
  name: string;
  args: unknown;
}

export interface TraceEvent {
  type: 'user' | 'agent_start' | 'agent_end' | 'model' | 'tool_call' | 'tool_result' | 'done' | 'error';
  timestamp: number;
  data: string;
  agent?: string;
}

export function useSSE(projectId: string | null, binaryPath?: string | null) {
  const [isStreaming, setIsStreaming] = useState(false);
  const [streamingText, setStreamingText] = useState('');
  const [currentAgent, setCurrentAgent] = useState('');
  const [toolCalls, setToolCalls] = useState<ToolCall[]>([]);
  const [events, setEvents] = useState<TraceEvent[]>([]);
  const esRef = useRef<EventSource | null>(null);
  const textRef = useRef('');
  const agentRef = useRef('');

  const addEvent = (type: TraceEvent['type'], data: string, agent?: string) => {
    setEvents(prev => [...prev, { type, timestamp: Date.now(), data, agent: agent || agentRef.current }]);
  };

  const send = useCallback(
    (input: string, onComplete: (text: string) => void, onError?: (msg: string) => void) => {
      if (!projectId) return;

      textRef.current = '';
      agentRef.current = '';
      setStreamingText('');
      setCurrentAgent('');
      setToolCalls([]);
      setEvents([{ type: 'user', timestamp: Date.now(), data: input }]);
      setIsStreaming(true);

      const params = new URLSearchParams({ input });
      if (binaryPath) {
        params.set('binary_path', binaryPath);
      }
      const es = new EventSource(`/api/projects/${projectId}/stream?${params}`);
      esRef.current = es;
      let ended = false;

      es.addEventListener('agent', (e) => {
        if (textRef.current) {
          textRef.current += '\n\n';
          setStreamingText(textRef.current);
        }
        agentRef.current = e.data;
        setCurrentAgent(e.data);
        addEvent('agent_start', 'runtime', e.data);
      });

      es.addEventListener('chunk', (e) => {
        textRef.current = e.data;  // Replace, not append (binary sends full response)
        setStreamingText(textRef.current);
      });

      es.addEventListener('trace', (e) => {
        try {
          const trace = JSON.parse(e.data);
          if (trace.type === 'node_start') {
            agentRef.current = trace.node;
            setCurrentAgent(trace.node);
            addEvent('agent_start', `Step ${trace.step}`, trace.node);
          } else if (trace.type === 'node_end') {
            addEvent('agent_end', `${trace.duration_ms}ms`, trace.node);
          } else if (trace.type === 'state') {
            // State update - show what changed
            const state = trace.state || {};
            if (state.response) {
              addEvent('model', state.response.slice(0, 100) + (state.response.length > 100 ? '...' : ''), agentRef.current);
            }
          } else if (trace.type === 'done') {
            addEvent('done', `${trace.total_steps} steps`);
          }
        } catch {}
      });

      es.addEventListener('tool_call', (e) => {
        try {
          const data = JSON.parse(e.data);
          setToolCalls(prev => [...prev, { name: data.name, args: data.args }]);
          textRef.current += `\n🔧 Calling ${data.name}...\n`;
          setStreamingText(textRef.current);
          addEvent('tool_call', `${data.name}(${JSON.stringify(data.args)})`);
        } catch {}
      });

      es.addEventListener('tool_result', (e) => {
        try {
          const data = JSON.parse(e.data);
          const resultStr = typeof data.result === 'string' ? data.result : JSON.stringify(data.result).slice(0, 200);
          textRef.current += `✓ ${data.name}: ${resultStr}\n`;
          setStreamingText(textRef.current);
          addEvent('tool_result', `${data.name} → ${resultStr}`);
        } catch {}
      });

      es.addEventListener('end', () => {
        ended = true;
        const finalText = textRef.current;
        setStreamingText('');
        setCurrentAgent('');
        setIsStreaming(false);
        es.close();
        onComplete(finalText);
      });

      es.addEventListener('error', (e) => {
        if (!ended) {
          const msg = (e as MessageEvent).data || 'Connection error';
          setStreamingText('');
          setCurrentAgent('');
          setIsStreaming(false);
          es.close();
          addEvent('error', msg);
          onError?.(msg);
        }
      });
    },
    [projectId, binaryPath]
  );

  const cancel = useCallback(() => {
    esRef.current?.close();
    setStreamingText('');
    setCurrentAgent('');
    setIsStreaming(false);
  }, []);

  const clearEvents = useCallback(() => setEvents([]), []);

  return { send, cancel, isStreaming, streamingText, currentAgent, toolCalls, events, clearEvents };
}
