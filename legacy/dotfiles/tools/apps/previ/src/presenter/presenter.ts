export interface IPresenter {
    start(): Promise<void>;
    stop(): Promise<void>;
    present(content: string): Promise<void>;
}
