import { describe, it, expect, beforeEach } from 'vitest';
import { InMemoryFileSystem } from './in_memory';

describe('InMemoryFileSystem', () => {
    let fileSystem: InMemoryFileSystem;

    beforeEach(() => {
        fileSystem = new InMemoryFileSystem();
    });

    it('should write and read a file with the same encoding', async () => {
        const path = '/path/to/file.txt';
        const content = 'Hello, world!';
        const encoding = 'utf8';

        await fileSystem.writeFile(path, content, encoding);
        const readContent = await fileSystem.readFile(path, encoding);

        expect(readContent).toBe(content);
    });

    it('should throw an error when reading a file with a different encoding', async () => {
        const path = '/path/to/file.txt';
        const content = 'Hello, world!';
        const writeEncoding = 'utf8';
        const readEncoding = 'ascii';

        await fileSystem.writeFile(path, content, writeEncoding);

        await expect(
            fileSystem.readFile(path, readEncoding)
        ).rejects.toThrowError(
            `Encoding mismatch for file ${path}. Expected ${writeEncoding}, got ${readEncoding}`
        );
    });

    it('should delete a file', async () => {
        const path = '/path/to/file.txt';
        const content = 'Hello, world!';
        const encoding = 'utf8';

        await fileSystem.writeFile(path, content, encoding);
        await fileSystem.deleteFile(path);

        await expect(fileSystem.readFile(path, encoding)).rejects.toThrowError(
            `File not found: ${path}`
        );
    });

    it('should rename a file', async () => {
        const oldPath = '/path/to/old/file.txt';
        const newPath = '/path/to/new/file.txt';
        const content = 'Hello, world!';
        const encoding = 'utf8';

        await fileSystem.writeFile(oldPath, content, encoding);
        await fileSystem.renameFile(oldPath, newPath);

        const readContent = await fileSystem.readFile(newPath, encoding);
        expect(readContent).toBe(content);

        await expect(
            fileSystem.readFile(oldPath, encoding)
        ).rejects.toThrowError(`File not found: ${oldPath}`);
    });

    it('should throw an error when reading a non-existent file', async () => {
        const path = '/path/to/nonexistent/file.txt';
        const encoding = 'utf8';

        await expect(fileSystem.readFile(path, encoding)).rejects.toThrowError(
            `File not found: ${path}`
        );
    });

    it('should throw an error when deleting a non-existent file', async () => {
        const path = '/path/to/nonexistent/file.txt';

        await expect(fileSystem.deleteFile(path)).resolves.toBeUndefined();
    });

    it('should throw an error when renaming a non-existent file', async () => {
        const oldPath = '/path/to/nonexistent/file.txt';
        const newPath = '/path/to/new/file.txt';

        await expect(
            fileSystem.renameFile(oldPath, newPath)
        ).rejects.toThrowError(`File not found: ${oldPath}`);
    });
});
