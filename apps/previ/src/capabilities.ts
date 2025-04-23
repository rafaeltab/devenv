export class Capability {
    constructor(public name: string) { }

    static capabilities = {
        versions: {
            v1_0: new Capability("version/1.0")
        },
        present: {
            live: new Capability("present/live")
        },
        render: {
            markdown: {
                self: new Capability('render/markdown'),
                abbreviation: new Capability('render/markdown/abbreviation'),
                alert: new Capability('render/markdown/alert'),
                align: new Capability('render/markdown/align'),
                attribute: new Capability('render/markdown/attribute'),
                container: new Capability('render/markdown/container'),
                demo: new Capability('render/markdown/demo'),
                definitionList: new Capability(
                    'render/markdown/definition-list'
                ),
                image: {
                    caption: new Capability('render/markdown/image/caption'),
                    lazy: new Capability('render/markdown/image/lazy'),
                    theme: new Capability('render/markdown/image/theme'),
                    size: new Capability('render/markdown/image/size'),
                },
                math: {
                    katex: new Capability('render/markdown/math/katex'),
                    tex: new Capability('render/markdown/math/tex'),
                },
                uml: {
                    plantuml: new Capability('render/markdown/uml/plantuml'),
                },
                code: {
                    snippet: new Capability('render/markdown/code/snippet'),
                    highlight: {
                        default: new Capability(
                            'render/markdown/code/highlight/default'
                        ),
                    },
                },
                footnote: new Capability('render/markdown/footnote'),
                icon: {
                    default: new Capability('render/markdown/icon/default'),
                    fontAwesome: new Capability(
                        'render/markdown/icon/fontAwesome'
                    ),
                },
                include: new Capability('render/markdown/include'),
                ins: new Capability('render/markdown/ins'),
                mark: new Capability('render/markdown/mark'),
                ruby: new Capability('render/markdown/ruby'),
                spoiler: new Capability('render/markdown/spoiler'),
                subscript: new Capability('render/markdown/subscript'),
                superscript: new Capability('render/markdown/superscript'),
                tabs: new Capability('render/markdown/tabs'),
                tasklist: new Capability('render/markdown/tasklist'),
            },
        },
    } as const;
}

export class Capabilities {
    public set!: Set<string>;
    public list!: Capability[];
    public length!: number;

    constructor(capabilities: Capability[]) {
        this.set = new Set(capabilities.map((x) => x.name));
        this.list = capabilities;
        this.length = capabilities.length;
    }

    has<T extends Capability | string>(capability: T): boolean {
        let str = typeof capability == 'string' ? capability : capability.name;
        return this.set.has(str);
    }

    hasAll(capabilities: Capabilities): boolean {
        if (capabilities.length > this.length) return false;

        for (let capability of capabilities.list) {
            if (!this.has(capability)) return false;
        }
        return true;
    }

    hasAny(capabilities: Capabilities): boolean {
        for (let capability of capabilities.list) {
            if (this.has(capability)) return true;
        }
        return false;
    }

    toStringList(): string[] {
        return this.list.map(x => x.name);
    }

    static merge(...capabilities: Capabilities[]): Capabilities {
        let combined = new Set(capabilities.flatMap((x) => x.list));
        return new Capabilities([...combined]);
    }

    static from(capabilities: string[]) {
        return new Capabilities(capabilities.map((x) => new Capability(x)));
    }
}
