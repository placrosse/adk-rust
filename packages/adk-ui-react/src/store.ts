import { A2uiStore } from './a2ui/store';
import type { ParsedA2uiMessage } from './a2ui/parser';
import { applyProtocolPayload } from './protocols';
import type { UiResponse } from './types';

function isRecord(value: unknown): value is Record<string, unknown> {
    return typeof value === 'object' && value !== null;
}

function isLegacyComponentArray(value: unknown): value is UiResponse['components'] {
    return Array.isArray(value) && value.every((entry) => isRecord(entry) && typeof entry.type === 'string');
}

function getUiTheme(value: unknown): UiResponse['theme'] | undefined {
    if (value === 'light' || value === 'dark' || value === 'system') {
        return value;
    }
    return undefined;
}

function extractLegacyUiResponse(payload: unknown): UiResponse | null {
    if (!isRecord(payload)) {
        return null;
    }

    if (isLegacyComponentArray(payload.components)) {
        return {
            id: typeof payload.id === 'string' ? payload.id : undefined,
            theme: getUiTheme(payload.theme),
            components: payload.components,
        };
    }

    if (
        isRecord(payload.payload)
        && isLegacyComponentArray(payload.payload.components)
    ) {
        return {
            id: typeof payload.payload.id === 'string' ? payload.payload.id : undefined,
            theme: getUiTheme(payload.payload.theme),
            components: payload.payload.components,
        };
    }

    return null;
}

export class UnifiedRenderStore {
    private readonly a2uiStore: A2uiStore;
    private legacyUiResponse: UiResponse | null = null;

    constructor(a2uiStore: A2uiStore = new A2uiStore()) {
        this.a2uiStore = a2uiStore;
    }

    getA2uiStore(): A2uiStore {
        return this.a2uiStore;
    }

    getLegacyUiResponse(): UiResponse | null {
        return this.legacyUiResponse;
    }

    clearLegacyUiResponse() {
        this.legacyUiResponse = null;
    }

    applyPayload(payload: unknown): ParsedA2uiMessage[] {
        const parsed = applyProtocolPayload(this.a2uiStore, payload);
        if (parsed.length > 0) {
            return parsed;
        }

        const legacy = extractLegacyUiResponse(payload);
        if (legacy) {
            this.legacyUiResponse = legacy;
        }
        return [];
    }
}
