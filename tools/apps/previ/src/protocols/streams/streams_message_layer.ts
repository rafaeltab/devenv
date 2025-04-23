import { IMessageProcessor } from '../../processing/message_processor';
import { Readable, Writable } from 'stream';
import { OutgoingMessages } from '../v1/types';
import { IMessageLayer } from '../message_layer';

export class StreamsMessageLayer implements IMessageLayer {
    private readonly messageProcessor: IMessageProcessor;
    private readonly stdout: Writable;
    private readonly stdin: Readable;

    private buffer: string = '';
    private isRunning: boolean = false;

    constructor(
        messageProcessor: IMessageProcessor,
        stdout?: Writable,
        stdin?: Readable
    ) {
        this.messageProcessor = messageProcessor;
        this.stdout = stdout ?? process.stdout;
        this.stdin = stdin ?? process.stdin;
    }

    public start(): void {
        if (this.isRunning) {
            return;
        }

        this.isRunning = true;
        this.stdin.on('data', (chunk: Buffer) => {
            this.buffer += chunk.toString();
            this.processBuffer();
        });

        this.stdin.on('end', () => {
            this.stop();
        });

        this.stdin.on('close', () => {
            this.stop();
        });

        this.stdin.on('error', (err) => {
            console.error('Error reading from stdin:', err);
            this.stop();
        });
    }

    public stop(): void {
        if (!this.isRunning) {
            return;
        }

        this.isRunning = false;
        this.stdin.removeAllListeners('data');
        this.stdin.removeAllListeners('end');
        this.stdin.removeAllListeners('close');
        this.stdin.removeAllListeners('error');
    }

    public sendMessage(message: OutgoingMessages): void {
        this.stdout.write(JSON.stringify(message) + '\n');
    }

    private processBuffer(): void {
        while (true) {
            const newlineIndex = this.buffer.indexOf('\n');
            if (newlineIndex === -1) {
                break;
            }

            const message = this.buffer.slice(0, newlineIndex);
            this.buffer = this.buffer.slice(newlineIndex + 1);

            if (message.trim().length == 0) continue;
            this.messageProcessor.processMessage(message).catch((err) => {
                console.error('Error processing message:', err);
            });
        }
    }
}
