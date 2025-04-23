import { describe, it, expect, beforeEach, vi, MockedFunction } from 'vitest';
import { CompositeRendererFactory } from './composite_renderer_factory';
import {
    CreateRendererResult,
    IRendererFactory,
    RendererInitializationProgress,
} from './renderer_factory';
import { of, Subject } from 'rxjs';
import { Capabilities, Capability } from '../capabilities';

type MockRendererFactory = {
    createRenderer: MockedFunction<
        (capabilities: string[]) => CreateRendererResult
    >;
    getCapabilities: MockedFunction<() => Capabilities>;
    getSupportedExtensions: MockedFunction<() => string[]>;
} & IRendererFactory;

describe('CompositeRendererFactory', () => {
    let mockRendererFactory1: MockRendererFactory;
    let mockRendererFactory2: MockRendererFactory;
    let compositeRendererFactory: CompositeRendererFactory;

    beforeEach(() => {
        mockRendererFactory1 = {
            createRenderer: vi.fn().mockReturnValue({
                renderer: Promise.resolve({ render: vi.fn() }),
                $progress: of({
                    rendererName: 'Renderer1',
                    progress: 1,
                    message: 'Initialized',
                }),
            }),
            getCapabilities: vi
                .fn()
                .mockReturnValue(
                    Capabilities.from(['capability1', 'capability2'])
                ),
            getSupportedExtensions: vi.fn().mockReturnValue(['ext1', 'ext2']),
        };
        mockRendererFactory2 = {
            createRenderer: vi.fn().mockReturnValue({
                renderer: Promise.resolve({ render: vi.fn() }),
                $progress: of({
                    rendererName: 'Renderer2',
                    progress: 1,
                    message: 'Initialized',
                }),
            }),
            getCapabilities: vi
                .fn()
                .mockReturnValue(
                    Capabilities.from(['capability3', 'capability4'])
                ),
            getSupportedExtensions: vi.fn().mockReturnValue(['ext3', 'ext4']),
        };
    });

    it('should create a composite renderer with the appropriate renderers based on capabilities', async () => {
        compositeRendererFactory = new CompositeRendererFactory([
            mockRendererFactory1,
            mockRendererFactory2,
        ]);
        const capabilities = new Capabilities([
            new Capability('capability1'),
            new Capability('capability3'),
        ]);
        const { renderer } =
            compositeRendererFactory.createRenderer(capabilities);
        await renderer;

        expect(mockRendererFactory1.createRenderer).toHaveBeenCalledWith(
            capabilities
        );
        expect(mockRendererFactory2.createRenderer).toHaveBeenCalledWith(
            capabilities
        );

        const compositeRenderer = await renderer;
        expect(compositeRenderer).toBeDefined();
    });

    it('should not create a renderer for a factory if none of its capabilities are present', async () => {
        compositeRendererFactory = new CompositeRendererFactory([
            mockRendererFactory1,
            mockRendererFactory2,
        ]);
        const capabilities = new Capabilities([
            new Capability('capability5'),
            new Capability('capability6'),
        ]);
        const { renderer } =
            compositeRendererFactory.createRenderer(capabilities);

        expect(mockRendererFactory1.createRenderer).not.toHaveBeenCalled();
        expect(mockRendererFactory2.createRenderer).not.toHaveBeenCalled();

        const compositeRenderer = await renderer;
        expect(compositeRenderer).toBeDefined();
    });

    it('should return a combined list of capabilities from all factories', () => {
        compositeRendererFactory = new CompositeRendererFactory([
            mockRendererFactory1,
            mockRendererFactory2,
        ]);
        const capabilities = compositeRendererFactory.getCapabilities();

        expect(capabilities.list.map((x) => x.name)).to.have.members([
            'capability1',
            'capability2',
            'capability3',
            'capability4',
        ]);
    });

    it('should return a combined list of supported extensions from all factories', () => {
        compositeRendererFactory = new CompositeRendererFactory([
            mockRendererFactory1,
            mockRendererFactory2,
        ]);
        const extensions = compositeRendererFactory.getSupportedExtensions();

        expect(extensions).toEqual(['ext1', 'ext2', 'ext3', 'ext4']);
    });

    it('should combine progress streams from all factories', async () => {
        const progress1 = new Subject<RendererInitializationProgress>();
        const progress2 = new Subject<RendererInitializationProgress>();

        mockRendererFactory1.createRenderer.mockReturnValue({
            renderer: Promise.resolve({ render: vi.fn() }),
            $progress: progress1.asObservable(),
        });
        mockRendererFactory2.createRenderer.mockReturnValue({
            renderer: Promise.resolve({ render: vi.fn() }),
            $progress: progress2.asObservable(),
        });

        compositeRendererFactory = new CompositeRendererFactory([
            mockRendererFactory1,
            mockRendererFactory2,
        ]);
        const capabilities = new Capabilities([
            new Capability('capability1'),
            new Capability('capability3'),
        ]);
        const { $progress } =
            compositeRendererFactory.createRenderer(capabilities);

        const progressValues: RendererInitializationProgress[] = [];
        $progress.subscribe((value) => progressValues.push(value));

        progress1.next({
            rendererName: 'Renderer1',
            progress: 0.5,
            message: 'Loading...',
        });
        progress2.next({
            rendererName: 'Renderer2',
            progress: 0.75,
            message: 'Configuring...',
        });
        progress1.complete();
        progress2.complete();

        expect(progressValues).toEqual([
            { rendererName: 'Renderer1', progress: 0.5, message: 'Loading...' },
            {
                rendererName: 'Renderer2',
                progress: 0.75,
                message: 'Configuring...',
            },
        ]);
    });
});
