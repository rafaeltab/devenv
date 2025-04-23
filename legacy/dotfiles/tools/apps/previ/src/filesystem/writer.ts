export interface IFileSystemWriter {
    writeFile(
        path: string,
        content: string,
        encoding?: BufferEncoding
    ): Promise<void>;
    deleteFile(path: string): Promise<void>;
    renameFile(oldPath: string, newPath: string): Promise<void>;
}
