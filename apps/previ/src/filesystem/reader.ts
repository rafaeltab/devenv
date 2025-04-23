export interface IFileSystemReader {
    readFile(path: string, encoding?: BufferEncoding): Promise<string>;
}
