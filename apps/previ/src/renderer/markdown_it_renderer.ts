import { IRenderer } from './renderer';
import MarkdownIt from 'markdown-it';
import { IFileSystemReader } from '../filesystem/reader';

export class MarkdownItRenderer implements IRenderer {
    constructor(
        private md: MarkdownIt,
        private fileSystem: IFileSystemReader
    ) {}

    async render(filePath: string): Promise<string> {
        try {
            const content = await this.fileSystem.readFile(filePath, 'utf8');
            return this.md.render(content);
        } catch (error: any) {
            throw new Error(
                `Error rendering file ${filePath}: ${error.message}`
            );
        }
    }
}
