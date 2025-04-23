import { Capabilities, Capability } from "../capabilities"

export type EngineState = {
    capabilities: {
        own: Capabilities,
        negotiated?: Capabilities
    },
    protocol: {
        version: string
    }
}

export const initialState = {
    capabilities: {
        own: new Capabilities([Capability.capabilities.versions.v1_0]),
        negotiated: undefined
    },
    protocol: {
        version: "1.0"
    }
} satisfies EngineState

