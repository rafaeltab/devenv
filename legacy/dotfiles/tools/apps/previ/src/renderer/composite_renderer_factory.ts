import { merge } from 'rxjs';
import { IRenderer } from './renderer';
import {
    CreateRendererResult,
    IRendererFactory,
    CreateRendererProgressStream,
} from './renderer_factory';
import { OptimizedMap } from '../utils/optimized_map';
import { CompositeRenderer } from './composite_renderer';
import { Capabilities } from '../capabilities';

export class CompositeRendererFactory implements IRendererFactory {
    private factories: IRendererFactory[];

    constructor(factories: IRendererFactory[]) {
        this.factories = factories;
    }

    createRenderer(capabilities: Capabilities): CreateRendererResult {
        let rendererMap = new OptimizedMap<CreateRendererResult, string[]>();
        let rendererCount = 0;

        for (let factory of this.factories) {
            let factoryCapabilities = factory.getCapabilities();
            let isNeeded = factoryCapabilities.hasAny(capabilities);

            if (!isNeeded) continue;

            rendererMap.set(
                factory.createRenderer(capabilities),
                factory.getSupportedExtensions()
            );
            rendererCount++;
        }

        let resultProgresses: CreateRendererProgressStream[] = rendererMap.map(
            (_, key) => key.$progress
        );

        return {
            $progress: merge(...resultProgresses),
            renderer: (async () => {
                let resultMap: Record<string, IRenderer> = {};
                for (let renderer of rendererMap) {
                    for (let fileType of renderer[1]) {
                        resultMap[fileType] = await renderer[0].renderer;
                    }
                }

                return new CompositeRenderer(resultMap);
            })(),
        };
    }

    getCapabilities(): Capabilities {
        return Capabilities.merge(
            ...this.factories.map((x) => x.getCapabilities())
        );
    }

    getSupportedExtensions(): string[] {
        return [
            ...new Set(
                this.factories.flatMap((x) => x.getSupportedExtensions())
            ),
        ];
    }
}
