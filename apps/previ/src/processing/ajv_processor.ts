import Ajv, { ValidateFunction } from 'ajv';
import { IMessageProcessor, MessageHandler } from './message_processor';
import {
    IncomingMessages,
    requestPreviewSchema,
    shutdownIncomingSchema,
    startSchema,
    updateFilesystemSchema,
} from '../protocols/v1/types';
import { ValidationException } from '../exceptions/validation_exception';

// Define a generic type for all incoming messages
interface IncomingMessage {
    protocolVersion: string;
    messageType: string;
    [key: string]: any; // Allow other properties
}

export class AjvMessageProcessor implements IMessageProcessor {
    private ajv: Ajv;
    private schemas: { [key: string]: any }; // Store schemas by version and type
    private validators: { [key: string]: ValidateFunction<any> }; // Store compiled validators
    private messageHandlers: { [key: string]: MessageHandler<any> }; // Store message handlers

    constructor() {
        this.ajv = new Ajv();
        this.schemas = {
            '1.0:start': startSchema,
            '1.0:update_filesystem': updateFilesystemSchema,
            '1.0:request_preview': requestPreviewSchema,
            '1.0:shutdown': shutdownIncomingSchema,
        };
        this.validators = {};
        this.messageHandlers = {};

        this.compileValidators();
    }

    private compileValidators(): void {
        for (let key of Object.keys(this.schemas)) {
            this.compileValidator(key);
        }
    }

    private compileValidator(key: string): void {
        if (!this.schemas[key]) {
            throw new Error(`Schema not found for key: ${key}`);
        }

        this.validators[key] = this.ajv.compile<unknown>(this.schemas[key]);
    }

    public registerHandler<T extends IncomingMessages>(
        key: string,
        handler: MessageHandler<T>
    ): void {
        this.messageHandlers[key] = handler;
    }

    public async processMessage(messageData: string): Promise<void> {
        const message: IncomingMessage = JSON.parse(messageData);

        const { protocolVersion, messageType } = message;
        const schemaKey = `${protocolVersion}:${messageType}`;

        if (!this.validators[schemaKey]) {
            throw new Error(`No validator found for schema: ${schemaKey}`);
        }

        const validate = this.validators[schemaKey];
        const valid = validate(message);

        if (!valid) {
            throw new ValidationException(validate.errors!);
        }

        const handler = this.messageHandlers[schemaKey];
        if (!handler) {
            throw new Error(
                `No handler found for message type: ${messageType}`
            );
        }

        await handler(message);
    }
}
