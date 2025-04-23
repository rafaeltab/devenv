import { describe, it, expect, beforeEach, vi, MockedFunction } from 'vitest';
import { IMessageProcessor, MessageHandler } from './message_processor';
import { QueuedMessageProcessor } from './queued_processor';
import { StartSchema } from '../protocols/v1/types';
import { Completer } from '../utils/completer';
import { SpyablePromise } from '../utils/spyable_promise';

describe('QueuedMessageProcessor', () => {
    let mockMessageProcessor: {
        processMessage: MockedFunction<(messageData: string) => Promise<void>>;
        registerHandler: MockedFunction<
            (key: string, handler: MessageHandler<unknown>) => void
        >;
    } & IMessageProcessor;
    let queuedMessageProcessor: QueuedMessageProcessor;

    beforeEach(() => {
        mockMessageProcessor = {
            processMessage: vi.fn().mockResolvedValue(undefined),
            registerHandler: vi.fn(),
        } satisfies IMessageProcessor;
        queuedMessageProcessor = new QueuedMessageProcessor(
            mockMessageProcessor
        );
    });

    it('should process messages in the order they are received', async () => {
        const message1: StartSchema = {
            protocolVersion: '1.0',
            messageType: 'start',
            capabilities: ['version/1.0', 'protocol/streams'],
        };
        const message2: StartSchema = {
            protocolVersion: '1.0',
            messageType: 'start',
            capabilities: ['version/1.0', 'protocol/http'],
        };
        const message3: StartSchema = {
            protocolVersion: '1.0',
            messageType: 'start',
            capabilities: ['version/1.0', 'render/markdown'],
        };

        await queuedMessageProcessor.processMessage(JSON.stringify(message1));
        await queuedMessageProcessor.processMessage(JSON.stringify(message2));
        await queuedMessageProcessor.processMessage(JSON.stringify(message3));

        // Wait for all messages to be processed
        await new Promise((resolve) => setTimeout(resolve, 0));

        expect(mockMessageProcessor.processMessage).toHaveBeenCalledTimes(3);
        expect(mockMessageProcessor.processMessage).toHaveBeenNthCalledWith(
            1,
            JSON.stringify(message1)
        );
        expect(mockMessageProcessor.processMessage).toHaveBeenNthCalledWith(
            2,
            JSON.stringify(message2)
        );
        expect(mockMessageProcessor.processMessage).toHaveBeenNthCalledWith(
            3,
            JSON.stringify(message3)
        );
    });

    it('should handle errors in message processing without blocking the queue', async () => {
        const message1: StartSchema = {
            protocolVersion: '1.0',
            messageType: 'start',
            capabilities: ['version/1.0', 'protocol/streams'],
        };
        const message2: StartSchema = {
            protocolVersion: '1.0',
            messageType: 'start',
            capabilities: ['version/1.0', 'protocol/http'],
        };
        const message3: StartSchema = {
            protocolVersion: '1.0',
            messageType: 'start',
            capabilities: ['version/1.0', 'render/markdown'],
        };

        mockMessageProcessor.processMessage.mockImplementationOnce(() => {
            throw new Error('Simulated error');
        });

        let errorMessage = '';
        try {
            await queuedMessageProcessor.processMessage(
                JSON.stringify(message1)
            );
        } catch (error) {
            errorMessage = error.message;
        }
        await queuedMessageProcessor.processMessage(JSON.stringify(message2));
        await queuedMessageProcessor.processMessage(JSON.stringify(message3));

        // Wait for all messages to be processed
        await new Promise((resolve) => setTimeout(resolve, 0));

        expect(mockMessageProcessor.processMessage).toHaveBeenCalledTimes(3);
        expect(mockMessageProcessor.processMessage).toHaveBeenNthCalledWith(
            2,
            JSON.stringify(message2)
        );
        expect(mockMessageProcessor.processMessage).toHaveBeenNthCalledWith(
            3,
            JSON.stringify(message3)
        );
        expect(errorMessage).to.contain('Simulated error');
    });

    it('should register handlers with the underlying message processor', () => {
        const key = '1.0:start';
        const handler = vi.fn();

        queuedMessageProcessor.registerHandler(key, handler);

        expect(mockMessageProcessor.registerHandler).toHaveBeenCalledWith(
            key,
            handler
        );
    });

    it('should only start processing a new message when the previous handler has finished', async () => {
        const message: StartSchema = {
            protocolVersion: '1.0',
            messageType: 'start',
            capabilities: ['version/1.0', 'protocol/streams'],
        };

        let firstCompleter = new Completer();
        let secondCompleter = new Completer();
        let thirdCompleter = new Completer();

        let firstStarted = false;
        let firstFinished = false;
        let secondStarted = false;
        let secondFinished = false;
        let thirdStarted = false;
        let thirdFinished = false;

        mockMessageProcessor.processMessage.mockImplementation(
            async (message) => {
                // Simulate a long-running handler
                if (!firstStarted) {
                    firstStarted = true;
                    await firstCompleter.promise;
                    firstFinished = true;
                } else if (!secondStarted) {
                    secondStarted = true;
                    await secondCompleter.promise;
                    secondFinished = true;
                } else {
                    thirdStarted = true;
                    await thirdCompleter.promise;
                    thirdFinished = true;
                }
            }
        );

        let firstPromise = new SpyablePromise(
            queuedMessageProcessor.processMessage(JSON.stringify(message))
        );
        let secondPromise = new SpyablePromise(
            queuedMessageProcessor.processMessage(JSON.stringify(message))
        );
        let thirdPromise = new SpyablePromise(
            queuedMessageProcessor.processMessage(JSON.stringify(message))
        );

        await new Promise((resolve) => setTimeout(resolve, 0));

        expect(firstPromise.fulfilled, 'First promise').toBe(false);
        expect(secondPromise.fulfilled, 'Second promise').toBe(false);
        expect(thirdPromise.fulfilled, 'Second promise').toBe(false);
        expect(firstStarted, 'First should have started').toBe(true);
        expect(firstFinished, 'First should not have finished').toBe(false);
        expect(secondStarted, 'Second should not have started').toBe(false);
        expect(secondFinished, 'Second should not have finished').toBe(false);
        expect(thirdStarted, 'Third should not have started').toBe(false);
        expect(thirdFinished, 'Third should not have finished').toBe(false);

        firstCompleter.resolve(null);
        await new Promise((resolve) => setTimeout(resolve, 0));

        expect(firstPromise.fulfilled, 'First promise').toBe(true);
        expect(secondPromise.fulfilled, 'Second promise').toBe(false);
        expect(thirdPromise.fulfilled, 'Third promise').toBe(false);
        expect(firstStarted, 'First should have started').toBe(true);
        expect(firstFinished, 'First should have finished').toBe(true);
        expect(secondStarted, 'Second should have started').toBe(true);
        expect(secondFinished, 'Second should not have finished').toBe(false);
        expect(thirdStarted, 'Third should not have started').toBe(false);
        expect(thirdFinished, 'Third should not have finished').toBe(false);

        secondCompleter.resolve(null);
        await new Promise((resolve) => setTimeout(resolve, 0));

        expect(firstPromise.fulfilled, 'First promise').toBe(true);
        expect(secondPromise.fulfilled, 'Second promise').toBe(true);
        expect(thirdPromise.fulfilled, 'Third promise').toBe(false);
        expect(firstStarted, 'First should have started').toBe(true);
        expect(firstFinished, 'First should have finished').toBe(true);
        expect(secondStarted, 'Second should have started').toBe(true);
        expect(secondFinished, 'Second should have finished').toBe(true);
        expect(thirdStarted, 'Third should have started').toBe(true);
        expect(thirdFinished, 'Third should not have finished').toBe(false);

        thirdCompleter.resolve(null);
        await new Promise((resolve) => setTimeout(resolve, 0));

        expect(firstPromise.fulfilled, 'First promise').toBe(true);
        expect(secondPromise.fulfilled, 'Second promise').toBe(true);
        expect(thirdPromise.fulfilled, 'Third promise').toBe(true);
        expect(firstStarted, 'First should have started').toBe(true);
        expect(firstFinished, 'First should have finished').toBe(true);
        expect(secondStarted, 'Second should have started').toBe(true);
        expect(secondFinished, 'Second should have finished').toBe(true);
        expect(thirdStarted, 'Third should have started').toBe(true);
        expect(thirdFinished, 'Third should have finished').toBe(true);
    });
});
