import { useCallback, useRef, useState } from 'react';

export function useSSE(projectId: string | null) {
  const [isStreaming, setIsStreaming] = useState(false);
  const [streamingText, setStreamingText] = useState('');
  const [currentAgent, setCurrentAgent] = useState('');
  const esRef = useRef<EventSource | null>(null);
  const textRef = useRef('');

  const send = useCallback(
    (input: string, onComplete: (text: string) => void, onError?: (msg: string) => void) => {
      if (!projectId) return;

      textRef.current = '';
      setStreamingText('');
      setCurrentAgent('');
      setIsStreaming(true);

      const params = new URLSearchParams({ input });
      const es = new EventSource(`/api/projects/${projectId}/stream?${params}`);
      esRef.current = es;
      let ended = false;

      es.addEventListener('agent', (e) => {
        // New agent started, add separator
        if (textRef.current) {
          textRef.current += '\n\n';
          setStreamingText(textRef.current);
        }
        setCurrentAgent(e.data);
      });

      es.addEventListener('chunk', (e) => {
        textRef.current += e.data;
        setStreamingText(textRef.current);
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
          onError?.(msg);
        }
      });
    },
    [projectId]
  );

  const cancel = useCallback(() => {
    esRef.current?.close();
    setStreamingText('');
    setCurrentAgent('');
    setIsStreaming(false);
  }, []);

  return { send, cancel, isStreaming, streamingText, currentAgent };
}
