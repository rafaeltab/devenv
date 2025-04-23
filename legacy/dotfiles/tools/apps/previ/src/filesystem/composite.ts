import { IFileSystemReader } from './reader';

export class CompositeFileSystem implements IFileSystemReader {
    private readonly fileSystems: IFileSystemReader[];

    constructor(fileSystems: IFileSystemReader[]) {
        this.fileSystems = fileSystems;
    }

    async readFile(
        path: string,
        encoding: BufferEncoding = 'utf8'
    ): Promise<string> {
        for (const fs of this.fileSystems) {
            try {
                return await fs.readFile(path, encoding);
            } catch (error) {
                // Ignore the error and try the next file system
            }
        }
        throw new Error(`File not found: ${path}`);
    }
}
