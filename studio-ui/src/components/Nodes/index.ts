import { LlmAgentNode } from './LlmAgentNode';
import { SequentialNode } from './SequentialNode';
import { LoopNode } from './LoopNode';
import { ParallelNode } from './ParallelNode';
import { RouterNode } from './RouterNode';

export const nodeTypes = {
  llm: LlmAgentNode,
  sequential: SequentialNode,
  loop: LoopNode,
  parallel: ParallelNode,
  router: RouterNode,
};

export { LlmAgentNode, SequentialNode, LoopNode, ParallelNode, RouterNode };
