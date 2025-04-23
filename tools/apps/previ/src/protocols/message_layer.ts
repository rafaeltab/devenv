import { OutgoingMessages } from './v1/types';

export interface IMessageLayer {
    start(): void;
    stop(): void;
    sendMessage(message: OutgoingMessages): void;
}
