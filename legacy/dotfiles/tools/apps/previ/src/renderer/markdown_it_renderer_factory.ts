import {
    CreateRendererResult,
    IRendererFactory,
    RendererInitializationComplete,
    RendererInitializationProgress,
} from './renderer_factory';
import MarkdownIt from 'markdown-it';
import { MarkdownItRenderer } from './markdown_it_renderer';
import { Capabilities, Capability } from '../capabilities';
import { ReplaySubject, Subject } from 'rxjs';
import { IFileSystemReader } from '../filesystem/reader';

export class MarkdownItRendererFactory implements IRendererFactory {
    constructor(private fileSystem: IFileSystemReader) {}

    createRenderer(capabilities: Capabilities): CreateRendererResult {
        let pluginList = plugins.filter((x) =>
            capabilities.hasAll(x.getCapabilities())
        );
        let subject = new ReplaySubject<
            RendererInitializationProgress | RendererInitializationComplete
        >();
        let markdown = new MarkdownIt();

        let res = pluginList.map(async (x, i) => {
            let progressStart = (i / pluginList.length) * 100;
            let progressEnd = ((i + 1) / pluginList.length) * 100;
            await x.initialize(
                markdown,
                capabilities,
                progressStart,
                progressEnd,
                subject
            );
        });

        return {
            renderer: Promise.all(res).then(
                () => new MarkdownItRenderer(markdown, this.fileSystem)
            ),
            $progress: subject.asObservable(),
        };
    }

    getCapabilities(): Capabilities {
        return new Capabilities([
            ...new Set(plugins.flatMap((x) => x.getCapabilities().list)),
            Capability.capabilities.render.markdown.self,
        ]);
    }

    getSupportedExtensions(): string[] {
        return ['md'];
    }
}

const plugins = [
    makeConfiguration(
        Capability.capabilities.render.markdown.abbreviation,
        '@mdit/plugin-abbr',
        async (md) => {
            const { abbr } = await import('@mdit/plugin-abbr');

            md.use(abbr);
        }
    ),
    makeConfiguration(
        Capability.capabilities.render.markdown.alert,
        '@mdit/plugin-alert',
        async (md) => {
            const { alert } = await import('@mdit/plugin-alert');

            md.use(alert);
        }
    ),
    makeConfiguration(
        Capability.capabilities.render.markdown.align,
        '@mdit/plugin-align',
        async (md) => {
            const { align } = await import('@mdit/plugin-align');

            md.use(align);
        }
    ),
    makeConfiguration(
        Capability.capabilities.render.markdown.attribute,
        '@mdit/plugin-attrs',
        async (md) => {
            const { attrs } = await import('@mdit/plugin-attrs');

            md.use(attrs);
        }
    ),
    makeConfiguration(
        Capability.capabilities.render.markdown.container,
        '@mdit/plugin-container',
        async (md) => {
            const { container } = await import('@mdit/plugin-container');

            md.use(container);
        }
    ),
    makeConfiguration(
        Capability.capabilities.render.markdown.demo,
        '@mdit/plugin-demo',
        async (md) => {
            const { demo } = await import('@mdit/plugin-demo');

            md.use(demo);
        }
    ),
    makeConfiguration(
        Capability.capabilities.render.markdown.definitionList,
        '@mdit/plugin-dl',
        async (md) => {
            const { dl } = await import('@mdit/plugin-dl');

            md.use(dl);
        }
    ),
    makeConfiguration(
        Capability.capabilities.render.markdown.image.caption,
        '@mdit/plugin-figure',
        async (md) => {
            const { figure } = await import('@mdit/plugin-figure');

            md.use(figure);
        }
    ),
    makeConfiguration(
        Capability.capabilities.render.markdown.footnote,
        '@mdit/plugin-footnote',
        async (md) => {
            const { footnote } = await import('@mdit/plugin-footnote');

            md.use(footnote);
        }
    ),
    makeConfiguration(
        Capability.capabilities.render.markdown.icon.default,
        '@mdit/plugin-icon',
        async (md) => {
            const { icon } = await import('@mdit/plugin-icon');

            md.use(icon);
        }
    ),
    makeConfiguration(
        Capability.capabilities.render.markdown.image.lazy,
        '@mdit/plugin-img-lazyload',
        async (md) => {
            const { imgLazyload } = await import('@mdit/plugin-img-lazyload');

            md.use(imgLazyload);
        }
    ),
    makeConfiguration(
        Capability.capabilities.render.markdown.image.theme,
        '@mdit/plugin-img-mark',
        async (md) => {
            const { imgMark } = await import('@mdit/plugin-img-mark');

            md.use(imgMark);
        }
    ),
    makeConfiguration(
        Capability.capabilities.render.markdown.image.size,
        '@mdit/plugin-img-size',
        async (md) => {
            const { imgSize } = await import('@mdit/plugin-img-size');

            md.use(imgSize);
        }
    ),
    makeConfiguration(
        Capability.capabilities.render.markdown.include,
        '@mdit/plugin-include',
        async (md) => {
            const { include } = await import('@mdit/plugin-include');

            md.use(include);
        }
    ),
    makeConfiguration(
        Capability.capabilities.render.markdown.ins,
        '@mdit/plugin-ins',
        async (md) => {
            const { ins } = await import('@mdit/plugin-ins');

            md.use(ins);
        }
    ),
    makeConfiguration(
        Capability.capabilities.render.markdown.math.katex,
        '@mdit/plugin-katex',
        async (md) => {
            const { katex } = await import('@mdit/plugin-katex');

            md.use(katex);
        }
    ),
    makeConfiguration(
        Capability.capabilities.render.markdown.mark,
        '@mdit/plugin-mark',
        async (md) => {
            const { mark } = await import('@mdit/plugin-mark');

            md.use(mark);
        }
    ),
    makeConfiguration(
        Capability.capabilities.render.markdown.uml.plantuml,
        '@mdit/plugin-plantuml',
        async (md) => {
            const { plantuml } = await import('@mdit/plugin-plantuml');

            md.use(plantuml);
        }
    ),
    makeConfiguration(
        Capability.capabilities.render.markdown.ruby,
        '@mdit/plugin-ruby',
        async (md) => {
            const { ruby } = await import('@mdit/plugin-ruby');

            md.use(ruby);
        }
    ),
    makeConfiguration(
        Capability.capabilities.render.markdown.code.snippet,
        '@mdit/plugin-snippet',
        async (md) => {
            const { snippet } = await import('@mdit/plugin-snippet');

            md.use(snippet);
        }
    ),
    makeConfiguration(
        Capability.capabilities.render.markdown.spoiler,
        '@mdit/plugin-spoiler',
        async (md) => {
            const { spoiler } = await import('@mdit/plugin-spoiler');

            md.use(spoiler);
        }
    ),
    makeConfiguration(
        Capability.capabilities.render.markdown.subscript,
        '@mdit/plugin-sub',
        async (md) => {
            const { sub } = await import('@mdit/plugin-sub');

            md.use(sub);
        }
    ),
    makeConfiguration(
        Capability.capabilities.render.markdown.superscript,
        '@mdit/plugin-sup',
        async (md) => {
            const { sup } = await import('@mdit/plugin-sup');

            md.use(sup);
        }
    ),
    makeConfiguration(
        Capability.capabilities.render.markdown.tabs,
        '@mdit/plugin-tab',
        async (md) => {
            const { tab } = await import('@mdit/plugin-tab');

            md.use(tab);
        }
    ),
    makeConfiguration(
        Capability.capabilities.render.markdown.tasklist,
        '@mdit/plugin-tasklist',
        async (md) => {
            const { tasklist } = await import('@mdit/plugin-tasklist');

            md.use(tasklist);
        }
    ),
    makeConfiguration(
        Capability.capabilities.render.markdown.math.tex,
        '@mdit/plugin-tex',
        async (md) => {
            const { tex } = await import('@mdit/plugin-tex');

            md.use(tex);
        }
    ),
];

interface PluginConfiguration {
    getCapabilities(): Capabilities;
    initialize(
        markdown: MarkdownIt,
        capabilities: Capabilities,
        progressStart: number,
        progressEnd: number,
        subject: Subject<
            RendererInitializationProgress | RendererInitializationComplete
        >
    ): Promise<void>;
}

function makeConfiguration(
    capability: Capability,
    pluginName: string,
    config: (markdown: MarkdownIt, a: Capabilities) => Promise<void>
): PluginConfiguration {
    return {
        getCapabilities() {
            return new Capabilities([capability]);
        },
        async initialize(
            markdown: MarkdownIt,
            capabilities: Capabilities,
            progressStart: number,
            progressEnd: number,
            subject: Subject<
                RendererInitializationProgress | RendererInitializationComplete
            >
        ) {
            subject.next({
                rendererName: 'markdown-it',
                message: `Initializing ${pluginName}`,
                progress: progressStart,
            });

            await config(markdown, capabilities);

            subject.next({
                rendererName: 'markdown-it',
                message: `Initialized ${pluginName}`,
                progress: progressEnd,
            });
        },
    };
}
