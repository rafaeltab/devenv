import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import { PhysicalFileSystem } from './physical';
import * as fsPromises from 'fs/promises';
import * as path from 'path';
import { fileURLToPath } from 'url';
import { dirname } from 'path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

describe('PhysicalFileSystem', () => {
    const tempFilePath = path.join(__dirname, 'temp.txt');

    beforeEach(async () => {
        // Ensure the temporary file does not exist before each test
        try {
            await fsPromises.unlink(tempFilePath);
        } catch (error: any) {
            if (error.code !== 'ENOENT') {
                // Ignore "file not found" errors
                throw error;
            }
        }
    });

    afterEach(async () => {
        // Clean up the temporary file after each test
        try {
            await fsPromises.unlink(tempFilePath);
        } catch (error: any) {
            if (error.code !== 'ENOENT') {
                // Ignore "file not found" errors
                throw error;
            }
        }
    });

    it('should read an existing file', async () => {
        const content = 'Hello, world!';
        await fsPromises.writeFile(tempFilePath, content, 'utf8');

        const physicalFileSystem = new PhysicalFileSystem();
        const readContent = await physicalFileSystem.readFile(
            tempFilePath,
            'utf8'
        );

        expect(readContent).toBe(content);
    });

    it('should throw an error when reading a non-existent file', async () => {
        const physicalFileSystem = new PhysicalFileSystem();

        await expect(
            physicalFileSystem.readFile(tempFilePath, 'utf8')
        ).rejects.toThrowError(`Error reading file ${tempFilePath}:`);
    });
});
