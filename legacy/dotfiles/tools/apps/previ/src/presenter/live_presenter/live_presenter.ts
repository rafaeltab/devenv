import { IPresenter } from '../presenter';
import { WebServer } from './webserver';
import { IFileSystemReader } from '../../filesystem/reader';
import { IRendererProvider } from '../../renderer/renderer_provider';

export class LiveServerPresenter implements IPresenter {
    private readonly webServer: WebServer;
    private readonly rendererProvider: IRendererProvider;

    constructor(
        port: number,
        fileSystem: IFileSystemReader,
        rendererProvider: IRendererProvider,
        initialContent: string
    ) {
        this.rendererProvider = rendererProvider;
        this.webServer = new WebServer(port, fileSystem, initialContent);
    }

    async start(): Promise<void> {
        this.webServer.start();
    }

    async stop(): Promise<void> {
        await this.webServer.stop();
    }

    async present(filePath: string): Promise<void> {
        const renderer = this.rendererProvider.getRenderer();
        const content = await renderer.render(filePath);
        this.webServer.setContent(content);
    }
}
