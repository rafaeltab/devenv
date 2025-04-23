import { ErrorObject } from 'ajv';

export class ValidationException extends Error {
    constructor(
        errors: ErrorObject<string, Record<string, unknown>, unknown>[]
    ) {
        super();
        this.name = `ValidationException`;
        this.message = `ValidationException: ${JSON.stringify(errors)}`;
    }
}
