import { IFileSystemReader } from './reader';
import { IFileSystemWriter } from './writer';

interface StoredFile {
    content: string;
    encoding: string;
}

export class InMemoryFileSystem
    implements IFileSystemReader, IFileSystemWriter
{
    private readonly files: Map<string, StoredFile> = new Map();

    async readFile(path: string, encoding: string = 'utf8'): Promise<string> {
        const storedFile = this.files.get(path);
        if (storedFile === undefined) {
            throw new Error(`File not found: ${path}`);
        }
        if (storedFile.encoding !== encoding) {
            throw new Error(
                `Encoding mismatch for file ${path}. Expected ${storedFile.encoding}, got ${encoding}`
            );
        }
        return storedFile.content;
    }

    async writeFile(
        path: string,
        content: string,
        encoding: string = 'utf8'
    ): Promise<void> {
        this.files.set(path, { content, encoding });
    }

    async deleteFile(path: string): Promise<void> {
        this.files.delete(path);
    }

    async renameFile(oldPath: string, newPath: string): Promise<void> {
        const storedFile = this.files.get(oldPath);
        if (storedFile === undefined) {
            throw new Error(`File not found: ${oldPath}`);
        }
        this.files.set(newPath, storedFile);
        this.files.delete(oldPath);
    }
}
