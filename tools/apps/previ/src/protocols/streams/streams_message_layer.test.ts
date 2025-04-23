import { describe, it, expect, beforeEach, vi } from 'vitest';
import { StreamsMessageLayer } from './streams_message_layer';
import { IMessageProcessor } from '../../processing/message_processor';
import { Readable, Writable } from 'stream';
import { InitSchema } from '../v1/types';

describe('StreamsMessageLayer', () => {
    let mockMessageProcessor: IMessageProcessor;
    let mockStdout: Writable;
    let mockStdin: Readable;
    let messageLayer: StreamsMessageLayer;

    beforeEach(() => {
        mockMessageProcessor = {
            processMessage: vi.fn().mockResolvedValue(undefined),
            registerHandler: vi.fn(),
        };
        mockStdout = {
            write: vi.fn(),
        } as any;
        mockStdin = new Readable({
            read() {},
        });
        messageLayer = new StreamsMessageLayer(
            mockMessageProcessor,
            mockStdout,
            mockStdin
        );
    });

    it('should start listening for data on stdin when start is called', () => {
        const onSpy = vi.spyOn(mockStdin, 'on');
        messageLayer.start();
        expect(onSpy).toHaveBeenCalledWith('data', expect.any(Function));
        expect(onSpy).toHaveBeenCalledWith('end', expect.any(Function));
        expect(onSpy).toHaveBeenCalledWith('close', expect.any(Function));
        expect(onSpy).toHaveBeenCalledWith('error', expect.any(Function));
    });

    it('should stop listening for data on stdin when stop is called', () => {
        messageLayer.start();
        const removeAllListenersSpy = vi.spyOn(mockStdin, 'removeAllListeners');
        messageLayer.stop();
        expect(removeAllListenersSpy).toHaveBeenCalledWith('data');
        expect(removeAllListenersSpy).toHaveBeenCalledWith('end');
        expect(removeAllListenersSpy).toHaveBeenCalledWith('close');
        expect(removeAllListenersSpy).toHaveBeenCalledWith('error');
    });

    it('should send a message to stdout when sendMessage is called', () => {
        const message: InitSchema = {
            protocolVersion: '1.0',
            messageType: 'init',
            capabilities: [],
        };
        messageLayer.sendMessage(message);
        expect(mockStdout.write).toHaveBeenCalledWith(
            JSON.stringify(message) + '\n'
        );
    });

    it('should process incoming messages from stdin', async () => {
        messageLayer.start();
        const messageData =
            '{"messageType": "init", "protocolVersion": "1.0"}\n';
        mockStdin.push(messageData);
        await new Promise((resolve) => setTimeout(resolve, 0)); // Wait for the event loop
        expect(mockMessageProcessor.processMessage).toHaveBeenCalledWith(
            messageData.trim()
        );
    });

    it('should handle multiple messages in a single chunk of data', async () => {
        messageLayer.start();
        const messageData =
            '{"messageType": "init", "protocolVersion": "1.0"}\n' +
            '{"messageType": "start", "protocolVersion": "1.0"}\n';
        mockStdin.push(messageData);
        await new Promise((resolve) => setTimeout(resolve, 0)); // Wait for the event loop
        expect(mockMessageProcessor.processMessage).toHaveBeenCalledTimes(2);
    });

    it('should handle incomplete messages', async () => {
        messageLayer.start();
        const messageData = '{"messageType": "init", "protocolVersion": "1.0"}';
        mockStdin.push(messageData);
        await new Promise((resolve) => setTimeout(resolve, 0)); // Wait for the event loop
        expect(mockMessageProcessor.processMessage).not.toHaveBeenCalled();
    });

    it('should handle empty messages', async () => {
        messageLayer.start();
        const messageData = '\n';
        mockStdin.push(messageData);
        await new Promise((resolve) => setTimeout(resolve, 0)); // Wait for the event loop
        expect(mockMessageProcessor.processMessage).not.toHaveBeenCalled();
    });

    it('should handle errors when reading from stdin', () => {
        const consoleErrorSpy = vi.spyOn(console, 'error');
        messageLayer.start();
        mockStdin.emit('error', new Error('Simulated error'));
        expect(consoleErrorSpy).toHaveBeenCalledWith(
            'Error reading from stdin:',
            new Error('Simulated error')
        );
    });
});
