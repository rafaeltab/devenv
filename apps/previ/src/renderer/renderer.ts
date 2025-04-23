export interface IRenderer {
    render(filePath: string): Promise<string>;
}
