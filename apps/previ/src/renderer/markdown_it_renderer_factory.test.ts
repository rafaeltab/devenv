import { describe, it, expect, beforeEach, vi, MockedFunction } from 'vitest';
import { MarkdownItRendererFactory } from './markdown_it_renderer_factory';
import { Capabilities, Capability } from '../capabilities';
import { IFileSystemReader } from '../filesystem/reader';
import { MarkdownItRenderer } from './markdown_it_renderer';
import { JSDOM } from 'jsdom';
import {
    RendererInitializationComplete,
    RendererInitializationProgress,
} from './renderer_factory';

describe('MarkdownItRendererFactory', () => {
    let mockFileSystem: {
        readFile: MockedFunction<() => string>;
    } & IFileSystemReader;

    beforeEach(() => {
        mockFileSystem = {
            readFile: vi.fn().mockResolvedValue('# Hello, world!'),
        };
    });

    it('should create a MarkdownItRenderer with the specified plugins based on capabilities', async () => {
        const factory = new MarkdownItRendererFactory(mockFileSystem);
        const capabilities = new Capabilities([
            Capability.capabilities.render.markdown.alert,
        ]);
        const { renderer, $progress } = factory.createRenderer(capabilities);
        const markdownItRenderer = (await renderer) as MarkdownItRenderer;

        expect(markdownItRenderer).toBeInstanceOf(MarkdownItRenderer);
    });

    it('should emit progress updates during plugin initialization', async () => {
        const factory = new MarkdownItRendererFactory(mockFileSystem);
        const capabilities = new Capabilities([
            Capability.capabilities.render.markdown.alert,
        ]);
        const { renderer, $progress } = factory.createRenderer(capabilities);

        const progressValues: (
            | RendererInitializationProgress
            | RendererInitializationComplete
        )[] = [];
        $progress.subscribe((value) => progressValues.push(value));

        // Wait for the renderer to be created
        await renderer;

        expect(progressValues.length).toBeGreaterThan(0);
        expect(
            progressValues.some((p) => p.message.includes('@mdit/plugin-alert'))
        ).toBe(true);
    });

    it('should return the correct capabilities', () => {
        const factory = new MarkdownItRendererFactory(mockFileSystem);
        const capabilities = factory.getCapabilities();

        expect(capabilities.list).toContain(
            Capability.capabilities.render.markdown.self
        );
        expect(capabilities.list).toContain(
            Capability.capabilities.render.markdown.alert
        );
    });

    it('should return the correct supported extensions', () => {
        const factory = new MarkdownItRendererFactory(mockFileSystem);
        const extensions = factory.getSupportedExtensions();

        expect(extensions).toEqual(['md']);
    });

    it('should render markdown correctly with the alert plugin', async () => {
        const factory = new MarkdownItRendererFactory(mockFileSystem);
        const capabilities = new Capabilities([
            Capability.capabilities.render.markdown.alert,
        ]);
        const { renderer } = factory.createRenderer(capabilities);
        const markdownItRenderer = (await renderer) as MarkdownItRenderer;
        let file = `
> [!warning]
> This is a warning alert.
        `;

        mockFileSystem.readFile.mockResolvedValue(file);
        const html = await markdownItRenderer.render('/path/to/file.md');

        const dom = new JSDOM(html);
        const alertDiv = dom.window.document.querySelector('.markdown-alert');
        expect(alertDiv).not.toBeNull();
        expect(alertDiv?.classList.toString()).to.include('warning');
        expect(alertDiv?.textContent).toContain('This is a warning alert.');
    });
});
