import { CompositeFileSystem } from "../filesystem/composite"
import { InMemoryFileSystem } from "../filesystem/in_memory"
import { PhysicalFileSystem } from "../filesystem/physical"
import { IFileSystemReader } from "../filesystem/reader"
import { IFileSystemWriter } from "../filesystem/writer"
import { LiveServerPresenter } from "../presenter/live_presenter/live_presenter"
import { IPresenter } from "../presenter/presenter"
import { AjvMessageProcessor } from "../processing/ajv_processor"
import { IMessageProcessor } from "../processing/message_processor"
import { QueuedMessageProcessor } from "../processing/queued_processor"
import { IMessageLayer } from "../protocols/message_layer"
import { StreamsMessageLayer } from "../protocols/streams/streams_message_layer"
import { CompositeRendererFactory } from "../renderer/composite_renderer_factory"
import { MarkdownItRendererFactory } from "../renderer/markdown_it_renderer_factory"
import { IRendererFactory } from "../renderer/renderer_factory"
import { IRendererProvider, RendererProvider } from "../renderer/renderer_provider"

export function createDependencies(): Dependencies {
    let messageProcessor = new QueuedMessageProcessor(new AjvMessageProcessor());
    let messageLayer = new StreamsMessageLayer(messageProcessor, process.stdout, process.stdin);
    let fileSystemWriter = new InMemoryFileSystem()
    let fileSystemReader = new CompositeFileSystem([
        new PhysicalFileSystem(),
        fileSystemWriter,
    ])
    let rendererFactory = new CompositeRendererFactory([
        new MarkdownItRendererFactory(fileSystemReader)
    ])
    let rendererProvider = new RendererProvider();
    let presenter = new LiveServerPresenter(8080, fileSystemReader, rendererProvider, "");

    return {
        messageProcessor: messageProcessor,
        messageLayer: messageLayer,
        fileSystemReader: fileSystemReader,
        fileSystemWriter: fileSystemWriter,
        rendererFactory: rendererFactory,
        rendererProvider: rendererProvider,
        presenter: presenter,
    };
}

export type Dependencies = {
    messageProcessor: IMessageProcessor,
    messageLayer: IMessageLayer,
    rendererFactory: IRendererFactory,
    rendererProvider: IRendererProvider,
    fileSystemReader: IFileSystemReader
    fileSystemWriter: IFileSystemWriter
    presenter: IPresenter
}
