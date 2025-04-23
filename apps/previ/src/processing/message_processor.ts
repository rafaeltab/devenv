import { IncomingMessages } from '../protocols/v1/types';

export type MessageHandler<T> = (message: T) => Promise<void>;

export interface IMessageProcessor {
    processMessage(messageData: unknown): Promise<void>;
    registerHandler<T extends IncomingMessages>(
        key: string,
        handler: MessageHandler<T>
    ): void;
}
