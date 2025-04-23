import { IncomingMessages } from '../protocols/v1/types';
import { Completer } from '../utils/completer';
import { IMessageProcessor } from './message_processor';

export class QueuedMessageProcessor implements IMessageProcessor {
    private nextMessageProcessor: IMessageProcessor;
    private messageQueue: [string, Completer<void>][] = [];
    private isProcessing: boolean = false;

    constructor(messageProcessor: IMessageProcessor) {
        this.nextMessageProcessor = messageProcessor;
    }

    public registerHandler<T extends IncomingMessages>(
        key: string,
        handler: (message: T) => Promise<void>
    ): void {
        this.nextMessageProcessor.registerHandler(key, handler);
    }

    public async processMessage(messageData: string): Promise<void> {
        let completer = new Completer<void>();
        this.messageQueue.push([messageData, completer]);
        if (!this.isProcessing) {
            void this.processQueue();
        }

        await completer.promise;
    }

    private async processQueue(): Promise<void> {
        this.isProcessing = true;
        while (this.messageQueue.length > 0) {
            const messageData = this.messageQueue.shift();
            if (!messageData) {
                throw Error('Unexpected error');
            }

            try {
                await this.nextMessageProcessor.processMessage(messageData[0]);
                messageData[1].resolve(undefined);
            } catch (error) {
                messageData[1].reject(error);
            }
        }
        this.isProcessing = false;
    }
}
