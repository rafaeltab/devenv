import { IRenderer } from './renderer';
import * as path from 'path';

export class CompositeRenderer implements IRenderer {
    private readonly renderers: Record<string, IRenderer>;

    constructor(renderers: Record<string, IRenderer>) {
        this.renderers = renderers;
    }

    async render(filePath: string): Promise<string> {
        const ext = path.extname(filePath).slice(1); // Remove the leading dot
        const renderer = this.renderers[ext];

        if (!renderer) {
            throw new Error(`No renderer found for file type: ${ext}`);
        }

        return renderer.render(filePath);
    }
}
