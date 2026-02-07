import type { ParsedA2uiMessage } from './a2ui/parser';
import type { UiEvent } from './types';
import { UnifiedRenderStore } from './store';

export type UiProtocol = 'adk_ui' | 'a2ui' | 'ag_ui' | 'mcp_apps';

export interface ProtocolClientOptions {
    protocol?: UiProtocol;
    store?: UnifiedRenderStore;
}

export interface OutboundEventOptions {
    surfaceId?: string;
    threadId?: string;
    runId?: string;
}

const DEFAULT_SURFACE_ID = 'main';

export function buildOutboundEvent(
    protocol: UiProtocol,
    event: UiEvent,
    options: OutboundEventOptions = {},
): Record<string, unknown> {
    const surfaceId = options.surfaceId ?? DEFAULT_SURFACE_ID;

    switch (protocol) {
        case 'ag_ui':
            return {
                protocol: 'ag_ui',
                event: {
                    type: 'CUSTOM',
                    name: 'adk.ui.event',
                    threadId: options.threadId ?? `thread-${surfaceId}`,
                    runId: options.runId ?? `run-${surfaceId}`,
                    value: {
                        surfaceId,
                        event,
                    },
                },
            };
        case 'mcp_apps':
            return {
                protocol: 'mcp_apps',
                method: 'ui.event',
                params: {
                    surfaceId,
                    event,
                },
            };
        case 'a2ui':
        case 'adk_ui':
        default:
            return {
                protocol,
                event: {
                    surfaceId,
                    ...event,
                },
            };
    }
}

export class ProtocolClient {
    private protocol: UiProtocol;
    private readonly store: UnifiedRenderStore;

    constructor(options: ProtocolClientOptions = {}) {
        this.protocol = options.protocol ?? 'adk_ui';
        this.store = options.store ?? new UnifiedRenderStore();
    }

    getProtocol(): UiProtocol {
        return this.protocol;
    }

    setProtocol(protocol: UiProtocol) {
        this.protocol = protocol;
    }

    getStore(): UnifiedRenderStore {
        return this.store;
    }

    applyPayload(payload: unknown): ParsedA2uiMessage[] {
        return this.store.applyPayload(payload);
    }

    buildOutboundEvent(event: UiEvent, options: OutboundEventOptions = {}): Record<string, unknown> {
        return buildOutboundEvent(this.protocol, event, options);
    }
}

export function createProtocolClient(options: ProtocolClientOptions = {}): ProtocolClient {
    return new ProtocolClient(options);
}
