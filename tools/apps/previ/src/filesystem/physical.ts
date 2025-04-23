import { IFileSystemReader } from './reader';
import * as fs from 'fs/promises';

export class PhysicalFileSystem implements IFileSystemReader {
    async readFile(
        filePath: string,
        encoding: BufferEncoding = 'utf8'
    ): Promise<string> {
        try {
            return await fs.readFile(filePath, { encoding });
        } catch (error: any) {
            throw new Error(`Error reading file ${filePath}: ${error.message}`);
        }
    }
}
