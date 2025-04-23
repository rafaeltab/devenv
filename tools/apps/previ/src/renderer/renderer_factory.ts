import { Observable } from 'rxjs';
import { IRenderer } from './renderer';
import { Capabilities } from '../capabilities';

export interface IRendererFactory {
    createRenderer(capabilities: Capabilities): CreateRendererResult;
    getCapabilities(): Capabilities;
    getSupportedExtensions(): string[];
}

export type CreateRendererProgressStream = Observable<
    RendererInitializationProgress | RendererInitializationComplete
>;

export type CreateRendererResult = {
    renderer: Promise<IRenderer>;
    $progress: CreateRendererProgressStream;
};

export type RendererInitializationComplete = RendererInitializationProgress & {
    progress: 100;
};

export type RendererInitializationProgress = {
    rendererName: string;
    progress: number;
    message: string;
};
