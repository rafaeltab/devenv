export class SpyablePromise<T> implements Promise<T> {
    private _fulfilled = false;
    private _rejected = false;
    [Symbol.toStringTag]!: string;

    public get fulfilled() {
        return this._fulfilled;
    }
    public get rejected() {
        return this._rejected;
    }

    constructor(private promise: Promise<T>) {
        this[Symbol.toStringTag] = promise[Symbol.toStringTag];
        this.promise.then(
            () => {
                this._fulfilled = true;
            },
            () => {
                this._rejected = true;
            }
        );
        this.promise.catch(() => {
            this._rejected = true;
        });
    }

    then<TResult1 = T, TResult2 = never>(
        onfulfilled?:
            | ((value: T) => TResult1 | PromiseLike<TResult1>)
            | null
            | undefined,
        onrejected?:
            | ((reason: any) => TResult2 | PromiseLike<TResult2>)
            | null
            | undefined
    ): Promise<TResult1 | TResult2> {
        return this.promise.then(onfulfilled, onrejected);
    }
    catch<TResult = never>(
        onrejected?:
            | ((reason: any) => TResult | PromiseLike<TResult>)
            | null
            | undefined
    ): Promise<T | TResult> {
        return this.promise.catch(onrejected);
    }
    finally(onfinally?: (() => void) | null | undefined): Promise<T> {
        return this.promise.finally(onfinally);
    }
}
