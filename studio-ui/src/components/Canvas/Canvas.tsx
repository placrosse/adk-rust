import { useCallback, useEffect, useState, DragEvent } from 'react';
import {
  ReactFlow,
  Background,
  Controls,
  Node,
  Edge,
  useNodesState,
  useEdgesState,
  Connection,
} from '@xyflow/react';
import '@xyflow/react/dist/style.css';
import { useStore } from '../../store';
import { TestConsole } from '../Console/TestConsole';

const AGENT_TYPES = [
  { type: 'llm', label: 'LLM Agent', enabled: true },
  { type: 'sequential', label: 'Sequential Agent', enabled: true },
  { type: 'tool', label: 'Tool Agent', enabled: false },
];

type FlowPhase = 'idle' | 'input' | 'output';

export function Canvas() {
  const { currentProject, closeProject, saveProject, selectNode, selectedNodeId, updateAgent, addAgent, addEdge: addProjectEdge, removeEdge: removeProjectEdge } = useStore();
  const [showConsole, setShowConsole] = useState(true);
  const [flowPhase, setFlowPhase] = useState<FlowPhase>('idle');

  const [nodes, setNodes, onNodesChange] = useNodesState<Node>([]);
  const [edges, setEdges, onEdgesChange] = useEdgesState<Edge>([]);

  useEffect(() => {
    if (!currentProject) return;
    
    const agentIds = Object.keys(currentProject.agents);
    // Filter out sub-agents (those that belong to a sequential)
    const allSubAgents = new Set(
      agentIds.flatMap(id => currentProject.agents[id].sub_agents || [])
    );
    const topLevelAgents = agentIds.filter(id => !allSubAgents.has(id));
    
    const newNodes: Node[] = [
      { id: 'START', position: { x: 200, y: 50 }, data: { label: '▶ START' }, type: 'input', style: { background: '#1a472a', border: '2px solid #4ade80', borderRadius: 8, padding: 10, color: '#fff' } },
      { id: 'END', position: { x: 200, y: 150 + topLevelAgents.length * 150 }, data: { label: '⏹ END' }, type: 'output', style: { background: '#4a1a1a', border: '2px solid #f87171', borderRadius: 8, padding: 10, color: '#fff' } },
    ];
    
    topLevelAgents.forEach((id, i) => {
      const agent = currentProject.agents[id];
      if (agent.type === 'sequential') {
        // Render sequential as container with sub-agents listed
        const subAgentLabels = (agent.sub_agents || []).map((subId, idx) => (
          <div key={subId} className="text-xs bg-gray-700 rounded px-2 py-1 mt-1">
            {idx + 1}. {subId}
          </div>
        ));
        newNodes.push({
          id,
          position: { x: 200, y: 150 + i * 150 },
          data: { 
            label: (
              <div className="text-center">
                <div className="font-semibold">⛓ {id}</div>
                <div className="text-xs text-gray-400 mb-1">Sequential Agent</div>
                <div className="border-t border-gray-600 pt-1 mt-1">
                  {subAgentLabels}
                </div>
              </div>
            )
          },
          style: { background: '#1e3a5f', border: '2px solid #60a5fa', borderRadius: 8, padding: 12, color: '#fff', minWidth: 150 },
        });
      } else {
        newNodes.push({
          id,
          position: { x: 200, y: 150 + i * 150 },
          data: { label: <div className="text-center"><div>🤖 {id}</div><div className="text-xs text-gray-400">LLM Agent</div></div> },
          style: { background: '#16213e', border: '2px solid #e94560', borderRadius: 8, padding: 12, color: '#fff', minWidth: 120 },
        });
      }
    });
    setNodes(newNodes);
  }, [currentProject, setNodes]);

  // Update edges based on flow phase
  useEffect(() => {
    if (!currentProject) return;
    
    const newEdges: Edge[] = currentProject.workflow.edges.map((e, i) => {
      const isStartEdge = e.from === 'START';
      const isEndEdge = e.to === 'END';
      const animated = (flowPhase === 'input' && isStartEdge) || (flowPhase === 'output' && isEndEdge);
      
      return {
        id: `e${i}`,
        source: e.from,
        target: e.to,
        type: 'smoothstep',
        animated,
        style: { stroke: animated ? '#4ade80' : '#e94560', strokeWidth: 2 },
        interactionWidth: 20,
      };
    });
    setEdges(newEdges);
  }, [currentProject, flowPhase, setEdges]);

  const createAgent = useCallback((agentType: string = 'llm') => {
    if (!currentProject) return;
    const agentCount = Object.keys(currentProject.agents).length;
    const id = agentType === 'sequential' ? `seq_${agentCount + 1}` : `agent_${agentCount + 1}`;
    
    if (agentType === 'sequential') {
      // Create sequential container with 2 default sub-agents
      const sub1 = `${id}_agent_1`;
      const sub2 = `${id}_agent_2`;
      addAgent(sub1, {
        type: 'llm',
        model: 'gemini-2.0-flash',
        instruction: 'You are agent 1.',
        tools: [],
        sub_agents: [],
        position: { x: 0, y: 0 },
      });
      addAgent(sub2, {
        type: 'llm',
        model: 'gemini-2.0-flash',
        instruction: 'You are agent 2.',
        tools: [],
        sub_agents: [],
        position: { x: 0, y: 0 },
      });
      addAgent(id, {
        type: 'sequential',
        instruction: '',
        tools: [],
        sub_agents: [sub1, sub2],
        position: { x: 200, y: 150 + agentCount * 180 },
      });
    } else {
      addAgent(id, {
        type: 'llm',
        model: 'gemini-2.0-flash',
        instruction: 'You are a helpful assistant.',
        tools: [],
        sub_agents: [],
        position: { x: 200, y: 150 + agentCount * 120 },
      });
    }
    addProjectEdge('START', id);
    addProjectEdge(id, 'END');
    selectNode(id);
  }, [currentProject, addAgent, addProjectEdge, selectNode]);

  const onDragStart = (e: DragEvent, nodeType: string) => {
    e.dataTransfer.setData('application/reactflow', nodeType);
    e.dataTransfer.effectAllowed = 'move';
  };

  const onDragOver = useCallback((e: DragEvent) => {
    e.preventDefault();
    e.dataTransfer.dropEffect = 'move';
  }, []);

  const onDrop = useCallback((e: DragEvent) => {
    e.preventDefault();
    const type = e.dataTransfer.getData('application/reactflow');
    if (!type) return;
    createAgent(type);
  }, [createAgent]);

  const onConnect = useCallback((params: Connection) => {
    if (params.source && params.target) {
      addProjectEdge(params.source, params.target);
    }
  }, [addProjectEdge]);

  const onEdgesDelete = useCallback((edgesToDelete: Edge[]) => {
    edgesToDelete.forEach((edge) => {
      removeProjectEdge(edge.source, edge.target);
    });
  }, [removeProjectEdge]);

  const onEdgeDoubleClick = useCallback((_: React.MouseEvent, edge: Edge) => {
    removeProjectEdge(edge.source, edge.target);
  }, [removeProjectEdge]);

  const onNodeClick = useCallback((_: React.MouseEvent, node: Node) => {
    if (node.id !== 'START' && node.id !== 'END') {
      selectNode(node.id);
    } else {
      selectNode(null);
    }
  }, [selectNode]);

  const onPaneClick = useCallback(() => selectNode(null), [selectNode]);

  if (!currentProject) return null;

  const selectedAgent = selectedNodeId ? currentProject.agents[selectedNodeId] : null;
  const hasAgents = Object.keys(currentProject.agents).length > 0;

  return (
    <div className="flex flex-col h-full">
      <div className="flex flex-1 overflow-hidden">
        {/* Palette */}
        <div className="w-48 bg-studio-panel border-r border-gray-700 p-4 flex flex-col">
          <h3 className="font-semibold mb-4">Components</h3>
          <div className="space-y-2 flex-1">
            {AGENT_TYPES.map(({ type, label, enabled }) => (
              <div
                key={type}
                draggable={enabled}
                onDragStart={(e) => enabled && onDragStart(e, type)}
                onClick={() => enabled && createAgent(type)}
                className={`p-2 bg-studio-accent rounded text-sm ${
                  enabled ? 'cursor-grab hover:bg-studio-highlight' : 'opacity-50 cursor-not-allowed'
                }`}
              >
                {enabled ? '⊕ ' : ''}{label}
              </div>
            ))}
          </div>
          <div className="space-y-2">
            <button onClick={() => setShowConsole(!showConsole)} className="w-full px-3 py-2 bg-gray-700 rounded text-sm">
              {showConsole ? 'Hide Console' : 'Show Console'}
            </button>
            <button onClick={closeProject} className="w-full px-3 py-2 bg-gray-700 rounded text-sm">Back</button>
          </div>
        </div>

        {/* Canvas */}
        <div className="flex-1">
          <ReactFlow
            nodes={nodes}
            edges={edges}
            onNodesChange={onNodesChange}
            onEdgesChange={onEdgesChange}
            onEdgesDelete={onEdgesDelete}
            onEdgeDoubleClick={onEdgeDoubleClick}
            onConnect={onConnect}
            onNodeClick={onNodeClick}
            onPaneClick={onPaneClick}
            onDrop={onDrop}
            onDragOver={onDragOver}
            deleteKeyCode={['Backspace', 'Delete']}
            fitView
            fitViewOptions={{ padding: 0.3, maxZoom: 1 }}
            minZoom={0.1}
            maxZoom={2}
          >
            <Background color="#333" gap={20} />
            <Controls />
          </ReactFlow>
        </div>

        {/* Properties */}
        {selectedAgent && (
          <div className="w-72 bg-studio-panel border-l border-gray-700 p-4 overflow-y-auto">
            <div className="flex justify-between items-center mb-4">
              <h3 className="font-semibold">{selectedNodeId}</h3>
              <div className="flex gap-2">
                <button onClick={saveProject} className="px-2 py-1 bg-studio-highlight rounded text-xs">Save</button>
                <button onClick={() => selectNode(null)} className="px-2 py-1 bg-gray-600 rounded text-xs">Close</button>
              </div>
            </div>
            
            {selectedAgent.type === 'sequential' ? (
              /* Sequential Agent Properties */
              <div>
                <label className="block text-sm text-gray-400 mb-2">Sub-Agents (in order)</label>
                {(selectedAgent.sub_agents || []).map((subId, idx) => {
                  const subAgent = currentProject.agents[subId];
                  if (!subAgent) return null;
                  return (
                    <div key={subId} className="mb-4 p-2 bg-gray-800 rounded">
                      <div className="text-sm font-medium mb-2">{idx + 1}. {subId}</div>
                      <label className="block text-xs text-gray-400 mb-1">Model</label>
                      <input
                        className="w-full px-2 py-1 bg-studio-bg border border-gray-600 rounded text-xs mb-2"
                        value={subAgent.model || ''}
                        onChange={(e) => updateAgent(subId, { model: e.target.value })}
                      />
                      <label className="block text-xs text-gray-400 mb-1">Instruction</label>
                      <textarea
                        className="w-full px-2 py-1 bg-studio-bg border border-gray-600 rounded text-xs h-20"
                        value={subAgent.instruction}
                        onChange={(e) => updateAgent(subId, { instruction: e.target.value })}
                      />
                    </div>
                  );
                })}
              </div>
            ) : (
              /* LLM Agent Properties */
              <div>
                <label className="block text-sm text-gray-400 mb-1">Model</label>
                <input
                  className="w-full px-2 py-1 bg-studio-bg border border-gray-600 rounded text-sm mb-3"
                  value={selectedAgent.model || ''}
                  onChange={(e) => updateAgent(selectedNodeId!, { model: e.target.value })}
                />
                <label className="block text-sm text-gray-400 mb-1">Instruction</label>
                <textarea
                  className="w-full px-2 py-1 bg-studio-bg border border-gray-600 rounded text-sm h-32"
                  value={selectedAgent.instruction}
                  onChange={(e) => updateAgent(selectedNodeId!, { instruction: e.target.value })}
                />
              </div>
            )}
          </div>
        )}
      </div>

      {/* Test Console */}
      {showConsole && hasAgents && (
        <div className="h-64">
          <TestConsole onFlowPhase={setFlowPhase} />
        </div>
      )}
      {showConsole && !hasAgents && (
        <div className="h-32 bg-studio-panel border-t border-gray-700 flex items-center justify-center text-gray-500">
          Drag "LLM Agent" onto the canvas to get started
        </div>
      )}
    </div>
  );
}
