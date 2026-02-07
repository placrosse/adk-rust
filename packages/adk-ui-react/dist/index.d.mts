import React from 'react';

type Component = {
    type: 'text';
    id?: string;
    content: string;
    variant?: TextVariant;
} | {
    type: 'button';
    id?: string;
    label: string;
    action_id: string;
    variant?: ButtonVariant;
    disabled?: boolean;
    icon?: string;
} | {
    type: 'icon';
    id?: string;
    name: string;
    size?: number;
} | {
    type: 'image';
    id?: string;
    src: string;
    alt?: string;
} | {
    type: 'badge';
    id?: string;
    label: string;
    variant?: BadgeVariant;
} | {
    type: 'text_input';
    id?: string;
    name: string;
    label: string;
    input_type?: 'text' | 'email' | 'password' | 'tel' | 'url';
    placeholder?: string;
    required?: boolean;
    default_value?: string;
    min_length?: number;
    max_length?: number;
    error?: string;
} | {
    type: 'number_input';
    id?: string;
    name: string;
    label: string;
    min?: number;
    max?: number;
    step?: number;
    required?: boolean;
    default_value?: number;
    error?: string;
} | {
    type: 'select';
    id?: string;
    name: string;
    label: string;
    options: SelectOption[];
    required?: boolean;
    error?: string;
} | {
    type: 'multi_select';
    id?: string;
    name: string;
    label: string;
    options: SelectOption[];
    required?: boolean;
} | {
    type: 'switch';
    id?: string;
    name: string;
    label: string;
    default_checked?: boolean;
} | {
    type: 'date_input';
    id?: string;
    name: string;
    label: string;
    required?: boolean;
} | {
    type: 'slider';
    id?: string;
    name: string;
    label: string;
    min?: number;
    max?: number;
    step?: number;
    default_value?: number;
} | {
    type: 'textarea';
    id?: string;
    name: string;
    label: string;
    placeholder?: string;
    rows?: number;
    required?: boolean;
    default_value?: string;
    error?: string;
} | {
    type: 'stack';
    id?: string;
    direction: 'horizontal' | 'vertical';
    children: Component[];
    gap?: number;
} | {
    type: 'grid';
    id?: string;
    columns: number;
    children: Component[];
    gap?: number;
} | {
    type: 'card';
    id?: string;
    title?: string;
    description?: string;
    content: Component[];
    footer?: Component[];
} | {
    type: 'container';
    id?: string;
    children: Component[];
    padding?: number;
} | {
    type: 'divider';
    id?: string;
} | {
    type: 'tabs';
    id?: string;
    tabs: Tab[];
} | {
    type: 'table';
    id?: string;
    columns: TableColumn[];
    data: Record<string, unknown>[];
    sortable?: boolean;
    page_size?: number;
    striped?: boolean;
} | {
    type: 'list';
    id?: string;
    items: string[];
    ordered?: boolean;
} | {
    type: 'key_value';
    id?: string;
    pairs: KeyValuePair[];
} | {
    type: 'code_block';
    id?: string;
    code: string;
    language?: string;
} | {
    type: 'chart';
    id?: string;
    title?: string;
    kind: ChartKind;
    data: Record<string, unknown>[];
    x_key: string;
    y_keys: string[];
    x_label?: string;
    y_label?: string;
    show_legend?: boolean;
    colors?: string[];
} | {
    type: 'alert';
    id?: string;
    title: string;
    description?: string;
    variant?: AlertVariant;
} | {
    type: 'progress';
    id?: string;
    value: number;
    label?: string;
} | {
    type: 'toast';
    id?: string;
    message: string;
    variant?: AlertVariant;
    duration?: number;
    dismissible?: boolean;
} | {
    type: 'modal';
    id?: string;
    title: string;
    content: Component[];
    footer?: Component[];
    size?: ModalSize;
    closable?: boolean;
} | {
    type: 'spinner';
    id?: string;
    size?: SpinnerSize;
    label?: string;
} | {
    type: 'skeleton';
    id?: string;
    variant?: SkeletonVariant;
    width?: string;
    height?: string;
};
type TextVariant = 'h1' | 'h2' | 'h3' | 'h4' | 'body' | 'caption' | 'code';
type ButtonVariant = 'primary' | 'secondary' | 'danger' | 'ghost' | 'outline';
type BadgeVariant = 'default' | 'info' | 'success' | 'warning' | 'error' | 'secondary' | 'outline';
type AlertVariant = 'info' | 'success' | 'warning' | 'error';
type ChartKind = 'bar' | 'line' | 'area' | 'pie';
type ModalSize = 'small' | 'medium' | 'large' | 'full';
type SpinnerSize = 'small' | 'medium' | 'large';
type SkeletonVariant = 'text' | 'circle' | 'rectangle';
interface SelectOption {
    label: string;
    value: string;
}
interface TableColumn {
    header: string;
    accessor_key: string;
    sortable?: boolean;
}
interface Tab {
    label: string;
    content: Component[];
}
interface KeyValuePair {
    key: string;
    value: string;
}
interface UiResponse {
    id?: string;
    theme?: 'light' | 'dark' | 'system';
    components: Component[];
}
type UiEvent = {
    action: 'form_submit';
    action_id: string;
    data: Record<string, unknown>;
} | {
    action: 'button_click';
    action_id: string;
} | {
    action: 'input_change';
    name: string;
    value: unknown;
} | {
    action: 'tab_change';
    index: number;
};
declare function uiEventToMessage(event: UiEvent): string;
type UiOperation = 'replace' | 'patch' | 'append' | 'remove';
interface UiUpdate {
    target_id: string;
    operation: UiOperation;
    payload?: Component;
}

interface RendererProps {
    component: Component;
    onAction?: (event: UiEvent) => void;
    /** Theme for this component: 'dark' wraps in dark mode styling */
    theme?: 'light' | 'dark' | 'system';
}
declare const Renderer: React.FC<RendererProps>;
interface StreamingRendererProps extends RendererProps {
    updates?: UiUpdate | UiUpdate[];
}
declare const StreamingRenderer: React.FC<StreamingRendererProps>;

type DataBinding = {
    path: string;
};
type FunctionCall = {
    call: string;
    args?: unknown[];
    returnType?: string;
};
interface ResolveContext {
    dataModel: Record<string, unknown>;
    scope?: Record<string, unknown>;
    functions?: FunctionRegistry;
}
type FunctionRegistry = Record<string, (args: unknown[], ctx: ResolveContext) => unknown>;
declare function isDataBinding(value: unknown): value is DataBinding;
declare function isFunctionCall(value: unknown): value is FunctionCall;
declare function resolvePath(dataModel: Record<string, unknown>, path: string, scope?: Record<string, unknown>): unknown;
declare function resolveDynamicValue(value: unknown, dataModel: Record<string, unknown>, scope?: Record<string, unknown>, functions?: FunctionRegistry): unknown;
declare function resolveDynamicString(value: unknown, dataModel: Record<string, unknown>, scope?: Record<string, unknown>, functions?: FunctionRegistry): string;

type A2uiActionEventDefinition = {
    name: string;
    context?: Record<string, unknown>;
};
type A2uiActionDefinition = {
    event?: A2uiActionEventDefinition;
    functionCall?: FunctionCall;
};
type A2uiActionEventPayload = {
    action: {
        name: string;
        surfaceId: string;
        sourceComponentId: string;
        timestamp: string;
        context: Record<string, unknown>;
    };
};
interface ActionEventOptions {
    dataModel: Record<string, unknown>;
    scope?: Record<string, unknown>;
    functions?: FunctionRegistry;
    timestamp?: Date;
}
declare function buildActionEvent(action: A2uiActionDefinition | undefined, surfaceId: string, sourceComponentId: string, options: ActionEventOptions): A2uiActionEventPayload | null;

type A2uiComponent = Record<string, unknown> & {
    id: string;
    component: string;
};
interface SurfaceState {
    components: Map<string, A2uiComponent>;
    dataModel: Record<string, unknown>;
}
declare class A2uiStore {
    private surfaces;
    getSurface(surfaceId: string): SurfaceState | undefined;
    ensureSurface(surfaceId: string): SurfaceState;
    applyUpdateComponents(surfaceId: string, components: A2uiComponent[]): void;
    removeSurface(surfaceId: string): void;
    applyUpdateDataModel(surfaceId: string, path: string | undefined, value: unknown): void;
}

interface A2uiSurfaceRendererProps {
    store: A2uiStore;
    surfaceId: string;
    rootId?: string;
    onAction?: (payload: A2uiActionEventPayload) => void;
    theme?: 'light' | 'dark' | 'system';
    functions?: FunctionRegistry;
}
declare const A2uiSurfaceRenderer: React.FC<A2uiSurfaceRendererProps>;

type CreateSurfaceMessage = {
    createSurface: {
        surfaceId: string;
        catalogId: string;
        theme?: Record<string, unknown>;
        sendDataModel?: boolean;
    };
};
type UpdateComponentsMessage = {
    updateComponents: {
        surfaceId: string;
        components: A2uiComponent[];
    };
};
type UpdateDataModelMessage = {
    updateDataModel: {
        surfaceId: string;
        path?: string;
        value?: unknown;
    };
};
type DeleteSurfaceMessage = {
    deleteSurface: {
        surfaceId: string;
    };
};
type A2uiMessage = CreateSurfaceMessage | UpdateComponentsMessage | UpdateDataModelMessage | DeleteSurfaceMessage;
interface ParsedA2uiMessage {
    message: A2uiMessage;
    raw: string;
}
declare function parseJsonl(payload: string): ParsedA2uiMessage[];
declare function applyParsedMessages(store: A2uiStore, parsed: ParsedA2uiMessage[]): void;

declare function applyUiUpdates(component: Component, updates: UiUpdate[]): Component | null;
declare function applyUiUpdate(component: Component, update: UiUpdate): Component | null;

declare function parseProtocolPayload(payload: unknown): ParsedA2uiMessage[];
declare function applyProtocolPayload(store: A2uiStore, payload: unknown): ParsedA2uiMessage[];

declare class UnifiedRenderStore {
    private readonly a2uiStore;
    private legacyUiResponse;
    constructor(a2uiStore?: A2uiStore);
    getA2uiStore(): A2uiStore;
    getLegacyUiResponse(): UiResponse | null;
    clearLegacyUiResponse(): void;
    applyPayload(payload: unknown): ParsedA2uiMessage[];
}

type UiProtocol = 'adk_ui' | 'a2ui' | 'ag_ui' | 'mcp_apps';
interface ProtocolClientOptions {
    protocol?: UiProtocol;
    store?: UnifiedRenderStore;
}
interface OutboundEventOptions {
    surfaceId?: string;
    threadId?: string;
    runId?: string;
}
declare function buildOutboundEvent(protocol: UiProtocol, event: UiEvent, options?: OutboundEventOptions): Record<string, unknown>;
declare class ProtocolClient {
    private protocol;
    private readonly store;
    constructor(options?: ProtocolClientOptions);
    getProtocol(): UiProtocol;
    setProtocol(protocol: UiProtocol): void;
    getStore(): UnifiedRenderStore;
    applyPayload(payload: unknown): ParsedA2uiMessage[];
    buildOutboundEvent(event: UiEvent, options?: OutboundEventOptions): Record<string, unknown>;
}
declare function createProtocolClient(options?: ProtocolClientOptions): ProtocolClient;

export { type A2uiActionDefinition, type A2uiActionEventDefinition, type A2uiActionEventPayload, type A2uiMessage, A2uiStore, A2uiSurfaceRenderer, type ActionEventOptions, type Component, type CreateSurfaceMessage, type DataBinding, type DeleteSurfaceMessage, type FunctionCall, type FunctionRegistry, type OutboundEventOptions, type ParsedA2uiMessage, ProtocolClient, type ProtocolClientOptions, Renderer, type ResolveContext, StreamingRenderer, type TableColumn, type UiEvent, type UiProtocol, type UiResponse, UnifiedRenderStore, type UpdateComponentsMessage, type UpdateDataModelMessage, applyParsedMessages, applyProtocolPayload, applyUiUpdate, applyUiUpdates, buildActionEvent, buildOutboundEvent, createProtocolClient, isDataBinding, isFunctionCall, parseJsonl, parseProtocolPayload, resolveDynamicString, resolveDynamicValue, resolvePath, uiEventToMessage };
