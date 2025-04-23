import { IRenderer } from './renderer';

export interface IRendererProvider {
    getRenderer(): IRenderer;
    setRenderer(renderer: IRenderer): void;
}

export class RendererProvider implements IRendererProvider {
    private renderer: IRenderer | null = null;

    getRenderer(): IRenderer {
        if (!this.renderer) {
            throw new Error('Renderer has not been set.');
        }
        return this.renderer;
    }

    setRenderer(renderer: IRenderer): void {
        this.renderer = renderer;
    }
}
