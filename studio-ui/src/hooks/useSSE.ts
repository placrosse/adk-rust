import { useCallback, useRef, useState } from 'react';

export function useSSE(projectId: string | null) {
  const [isStreaming, setIsStreaming] = useState(false);
  const [streamingText, setStreamingText] = useState('');
  const esRef = useRef<EventSource | null>(null);
  const textRef = useRef('');
  const queueRef = useRef<string[]>([]);
  const typingRef = useRef(false);

  const processQueue = (setFn: (t: string) => void) => {
    if (typingRef.current || queueRef.current.length === 0) return;
    
    typingRef.current = true;
    const chunk = queueRef.current.shift()!;
    let i = 0;
    
    const typeChar = () => {
      if (i < chunk.length) {
        textRef.current += chunk[i];
        setFn(textRef.current);
        i++;
        setTimeout(typeChar, 10);
      } else {
        typingRef.current = false;
        processQueue(setFn); // Process next chunk
      }
    };
    typeChar();
  };

  const send = useCallback(
    (input: string, onComplete: (text: string) => void, onError?: (msg: string) => void) => {
      if (!projectId) return;

      textRef.current = '';
      queueRef.current = [];
      typingRef.current = false;
      setStreamingText('');
      setIsStreaming(true);

      const params = new URLSearchParams({ input });
      const es = new EventSource(`/api/projects/${projectId}/stream?${params}`);
      esRef.current = es;
      let ended = false;

      es.addEventListener('chunk', (e) => {
        queueRef.current.push(e.data);
        processQueue(setStreamingText);
      });

      es.addEventListener('end', () => {
        ended = true;
        // Wait for typing queue to finish
        const waitForQueue = () => {
          if (queueRef.current.length > 0 || typingRef.current) {
            setTimeout(waitForQueue, 50);
          } else {
            const finalText = textRef.current;
            setStreamingText('');
            setIsStreaming(false);
            es.close();
            onComplete(finalText);
          }
        };
        waitForQueue();
      });

      es.addEventListener('error', (e) => {
        if (!ended) {
          const msg = (e as MessageEvent).data || 'Connection error';
          setStreamingText('');
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
    queueRef.current = [];
    typingRef.current = false;
    setStreamingText('');
    setIsStreaming(false);
  }, []);

  return { send, cancel, isStreaming, streamingText };
}
