export class OptimizedMap<K, V> extends Map<K, V> {
    private _size: number = 0;

    constructor(entries?: readonly (readonly [K, V])[] | null) {
        super(entries);
        this._size = super.size; // Initialize size based on initial entries
    }

    override set(key: K, value: V): this {
        if (!this.has(key)) {
            this._size++;
        }
        super.set(key, value);
        return this;
    }

    override delete(key: K): boolean {
        if (this.has(key)) {
            this._size--;
            return super.delete(key);
        }
        return false;
    }

    override clear(): void {
        super.clear();
        this._size = 0;
    }

    get internalSize(): number {
        return this._size;
    }

    map<T>(callbackfn: (value: V, key: K, map: this) => T, thisArg?: any): T[] {
        const result: T[] = new Array(this._size); // Pre-allocate array
        let i = 0;
        for (const [key, value] of this) {
            result[i] = callbackfn.call(thisArg, value, key, this);
            i++;
        }
        return result;
    }
}
