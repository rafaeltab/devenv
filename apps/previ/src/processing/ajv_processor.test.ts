import { describe, it, expect, beforeEach } from 'vitest';
import { AjvMessageProcessor } from './ajv_processor';
import { IMessageProcessor } from './message_processor';
import * as Types from '../protocols/v1/types';

describe('AjvMessageProcessor', () => {
    let messageProcessor: IMessageProcessor;

    beforeEach(() => {
        messageProcessor = new AjvMessageProcessor();
    });

    it('should successfully process a valid start message', async () => {
        let handlerCalled = false;
        let receivedMessage: unknown = undefined;
        const startMessage: Types.StartSchema = {
            protocolVersion: '1.0',
            messageType: 'start',
            capabilities: ['version/1.0', 'protocol/streams'],
        };

        messageProcessor.registerHandler<Types.StartSchema>(
            '1.0:start',
            async (message) => {
                handlerCalled = true;
                receivedMessage = message;
            }
        );

        await messageProcessor.processMessage(JSON.stringify(startMessage));

        expect(handlerCalled).toBe(true);
        expect(receivedMessage).toEqual(startMessage);
    });

    it('should reject an invalid start message', async () => {
        const invalidStartMessage = {
            protocolVersion: '1.0',
            messageType: 'start',
            capabilities: 123, // Invalid capabilities type
        };

        let errorMessage = '';
        try {
            await messageProcessor.processMessage(
                JSON.stringify(invalidStartMessage)
            );
        } catch (e: any) {
            errorMessage = e.message;
        }

        expect(typeof errorMessage).toBe('string');
        expect(errorMessage).toContain('capabilities');
    });

    it('should call the correct handler for a valid update_filesystem message', async () => {
        let handlerCalled = false;
        const updateFilesystemMessage: Types.UpdateFilesystemSchema = {
            protocolVersion: '1.0',
            messageType: 'update_filesystem',
            updates: [
                {
                    action: 'create',
                    path: '/path/to/new/file.txt',
                    content: {
                        type: 'base64',
                        base64: 'SGVsbG8sIHdvcmxkIQ==',
                        originalEncoding: 'utf8',
                    },
                },
            ],
        };

        messageProcessor.registerHandler<Types.UpdateFilesystemSchema>(
            '1.0:update_filesystem',
            async (message) => {
                handlerCalled = true;
                expect(message).toEqual(updateFilesystemMessage);
            }
        );

        await messageProcessor.processMessage(
            JSON.stringify(updateFilesystemMessage)
        );

        expect(handlerCalled).toBe(true);
    });

    it('should call the correct handler for a valid request_preview message', async () => {
        let handlerCalled = false;
        const requestPreviewMessage: Types.RequestPreviewSchema = {
            protocolVersion: '1.0',
            messageType: 'request_preview',
            path: '/path/to/my/file.md',
            liveUpdate: true,
        };

        messageProcessor.registerHandler<Types.RequestPreviewSchema>(
            '1.0:request_preview',
            async (message) => {
                handlerCalled = true;
                expect(message).toEqual(requestPreviewMessage);
            }
        );

        await messageProcessor.processMessage(
            JSON.stringify(requestPreviewMessage)
        );

        expect(handlerCalled).toBe(true);
    });

    it('should call the correct handler for a valid shutdown message', async () => {
        let handlerCalled = false;
        const shutdownMessage: Types.ShutdownSchema = {
            protocolVersion: '1.0',
            messageType: 'shutdown',
            reason: 'Shutdown requested by Neovim.',
            code: 0,
        };

        messageProcessor.registerHandler<Types.ShutdownSchema>(
            '1.0:shutdown',
            async (message) => {
                handlerCalled = true;
                expect(message).toEqual(shutdownMessage);
            }
        );

        await messageProcessor.processMessage(JSON.stringify(shutdownMessage));

        expect(handlerCalled).toBe(true);
    });
});
