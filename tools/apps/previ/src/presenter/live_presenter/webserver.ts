// web_server.ts
import express, { Express, Request, Response } from 'express';
import { IFileSystemReader } from '../../filesystem/reader';
import * as path from 'path';
import { Server } from 'http';

export class WebServer {
    private readonly app: Express;
    private readonly port: number;
    private readonly fileSystem: IFileSystemReader;
    // private readonly indexFilePath: string;
    private server: Server | null = null;
    private content: string = ''; // Store the content in memory

    constructor(
        port: number,
        fileSystem: IFileSystemReader,
        // indexFilePath: string,
        initialContent: string
    ) {
        this.port = port;
        this.fileSystem = fileSystem;
        // this.indexFilePath = indexFilePath;
        this.content = initialContent; // Set the initial content
        this.app = express();

        this.app.get('/', this.handleIndexRequest.bind(this));
        this.app.get('/index.html', this.handleIndexRequest.bind(this));
        this.app.get('*', this.handleStaticRequest.bind(this));
    }

    private async handleIndexRequest(
        req: Request,
        res: Response
    ): Promise<void> {
        try {
            // const contentType = this.getContentType(this.indexFilePath);

            res.setHeader('Content-Type', 'index/html');
            res.send(this.content); // Serve the in-memory content
        } catch (error: any) {
            console.error(`Error serving index file: ${error.message}`);
            res.status(500).send('Internal Server Error');
        }
    }

    private async handleStaticRequest(
        req: Request,
        res: Response
    ): Promise<void> {
        const filePath = req.path;

        try {
            const content = await this.fileSystem.readFile(filePath);
            const contentType = this.getContentType(filePath);

            res.setHeader('Content-Type', contentType);
            res.send(content);
        } catch (error: any) {
            console.error(`Error serving file ${filePath}: ${error.message}`);
            res.status(404).send('File not found');
        }
    }

    private getContentType(filePath: string): string {
        const ext = path.extname(filePath).toLowerCase();
        switch (ext) {
            case '.html':
                return 'text/html';
            case '.css':
                return 'text/css';
            case '.js':
                return 'text/javascript';
            case '.png':
                return 'image/png';
            case '.jpg':
            case '.jpeg':
                return 'image/jpeg';
            case '.gif':
                return 'image/gif';
            default:
                return 'text/plain';
        }
    }

    start(): void {
        this.server = this.app.listen(this.port, () => {
            console.log(`Web server listening on port ${this.port}`);
        });
    }

    stop(): Promise<void> {
        return new Promise((resolve, reject) => {
            if (this.server) {
                this.server.close((err) => {
                    if (err) {
                        console.error('Error stopping web server:', err);
                        reject(err);
                    } else {
                        console.log('Web server stopped');
                        this.server = null;
                        resolve();
                    }
                });
            } else {
                resolve();
            }
        });
    }

    setContent(content: string): void {
        this.content = content;
    }
}
