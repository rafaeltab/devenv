import { InitSchema } from '../protocols/v1/init.schema';
import { EngineState, initialState } from './state';
import { Dependencies, createDependencies } from "./dependencies";

export class Engine {
    private state: EngineState = initialState;
    private dependencies!: Dependencies;

    constructor() {
        this.dependencies = createDependencies();
    }

    public start() {
        this.discoverCapabilities();
        this.dependencies.messageLayer.start();
        this.dependencies.messageLayer.sendMessage({
            messageType: "init",
            capabilities: this.state.capabilities.own.toStringList(),
            protocolVersion: "1.0",
        } satisfies InitSchema);
    }

    private discoverCapabilities() {
        this.state.capabilities.own = this.dependencies.rendererFactory.getCapabilities();
    }
}

