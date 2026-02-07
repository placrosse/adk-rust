const { describe, it } = require('node:test');
const assert = require('node:assert/strict');

const {
    ProtocolClient,
    UnifiedRenderStore,
    buildOutboundEvent,
    createProtocolClient,
} = require('../dist/index.js');

describe('Protocol client', () => {
    it('applies ag_ui payload through unified store', () => {
        const client = createProtocolClient({ protocol: 'ag_ui' });
        const payload = {
            protocol: 'ag_ui',
            events: [
                { type: 'RUN_STARTED', threadId: 'thread-main', runId: 'run-main' },
                {
                    type: 'CUSTOM',
                    name: 'adk.ui.surface',
                    value: {
                        format: 'adk-ui-surface-v1',
                        surface: {
                            surfaceId: 'main',
                            catalogId: 'catalog',
                            components: [{ id: 'root', component: 'Column', children: [] }],
                        },
                    },
                },
                { type: 'RUN_FINISHED', threadId: 'thread-main', runId: 'run-main' },
            ],
        };

        const parsed = client.applyPayload(payload);
        assert.equal(parsed.length, 2);
        const surface = client.getStore().getA2uiStore().getSurface('main');
        assert.ok(surface);
        assert.equal(surface.components.get('root').component, 'Column');
    });

    it('keeps legacy adk_ui payloads for renderer compatibility', () => {
        const store = new UnifiedRenderStore();
        const client = new ProtocolClient({ store });

        const payload = {
            id: 'legacy-1',
            components: [
                {
                    type: 'text',
                    content: 'hello',
                },
            ],
        };

        const parsed = client.applyPayload(payload);
        assert.deepEqual(parsed, []);
        const legacy = client.getStore().getLegacyUiResponse();
        assert.ok(legacy);
        assert.equal(legacy.components[0].type, 'text');
    });

    it('builds outbound events using current protocol mapping', () => {
        const client = createProtocolClient({ protocol: 'mcp_apps' });
        const outbound = client.buildOutboundEvent({
            action: 'button_click',
            action_id: 'confirm',
        });

        assert.equal(outbound.protocol, 'mcp_apps');
        assert.equal(outbound.method, 'ui.event');
        assert.equal(outbound.params.event.action, 'button_click');
    });

    it('buildOutboundEvent supports explicit ag_ui options', () => {
        const outbound = buildOutboundEvent(
            'ag_ui',
            {
                action: 'tab_change',
                index: 1,
            },
            { surfaceId: 'page-1', threadId: 'thread-x', runId: 'run-x' },
        );

        assert.equal(outbound.protocol, 'ag_ui');
        assert.equal(outbound.event.threadId, 'thread-x');
        assert.equal(outbound.event.runId, 'run-x');
        assert.equal(outbound.event.value.surfaceId, 'page-1');
        assert.equal(outbound.event.value.event.action, 'tab_change');
    });
});
