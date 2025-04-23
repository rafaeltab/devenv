import { describe, it, expect, beforeEach } from 'vitest';
import { CompositeFileSystem } from './composite';
import { IFileSystemReader } from './reader';

class MockFileSystem implements IFileSystemReader {
    private readonly files: Map<string, string> = new Map();

    constructor(files: { [path: string]: string } = {}) {
        for (const path in files) {
            this.files.set(path, files[path]);
        }
    }

    async readFile(path: string, encoding: string = 'utf8'): Promise<string> {
        const content = this.files.get(path);
        if (content === undefined) {
            throw new Error(`File not found: ${path}`);
        }
        return content;
    }
}

describe('CompositeFileSystem', () => {
    let fileSystem: CompositeFileSystem;
    let fs1: MockFileSystem;
    let fs2: MockFileSystem;
    let fs3: MockFileSystem;

    beforeEach(() => {
        fs1 = new MockFileSystem({
            '/file1.txt': 'Content from fs1',
            '/file2.txt': 'Content from fs1',
        });
        fs2 = new MockFileSystem({
            '/file2.txt': 'Content from fs2',
            '/file3.txt': 'Content from fs2',
        });
        fs3 = new MockFileSystem({
            '/file4.txt': 'Content from fs3',
        });
        fileSystem = new CompositeFileSystem([fs1, fs2, fs3]);
    });

    it('should read a file from the first file system that contains it', async () => {
        const content = await fileSystem.readFile('/file1.txt');
        expect(content).toBe('Content from fs1');
    });

    it('should read a file from the second file system if it is not in the first', async () => {
        const content = await fileSystem.readFile('/file3.txt');
        expect(content).toBe('Content from fs2');
    });

    it('should read a file from the third file system if it is not in the first or second', async () => {
        const content = await fileSystem.readFile('/file4.txt');
        expect(content).toBe('Content from fs3');
    });

    it('should prefer the first file system when a file exists in multiple file systems', async () => {
        const content = await fileSystem.readFile('/file2.txt');
        expect(content).toBe('Content from fs1');
    });

    it('should throw an error if the file is not found in any file system', async () => {
        await expect(
            fileSystem.readFile('/nonexistent.txt')
        ).rejects.toThrowError('File not found: /nonexistent.txt');
    });
});
