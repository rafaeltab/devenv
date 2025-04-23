import { Capabilities, Capability } from "../../capabilities";
import { AcknowledgeSchema } from "../../protocols/v1/acknowledge.schema";
import { InitProgressSchema } from "../../protocols/v1/init_progress.schema";
import { RejectSchema } from "../../protocols/v1/reject.schema";
import { StartSchema } from "../../protocols/v1/start.schema";
import { Dependencies } from "../dependencies";
import { EngineState } from "../state";

export async function handleStart(message: StartSchema, dependencies: Dependencies, state: EngineState): Promise<void> {
    const requestedCapabilities = Capabilities.from(message.capabilities);
    const hasAllCapabilities = state.capabilities.own.hasAll(requestedCapabilities);

    if (!hasAllCapabilities) {
        reject("Server doesn't support requested capabilities", dependencies, state);
        return;
    }

    if (!validateCapabilities(requestedCapabilities, dependencies, state)) {
        return;
    }

    state.capabilities.negotiated = requestedCapabilities;

    const acknowledge = {
        messageType: "acknowledge",
        protocolVersion: state.protocol.version,
        message: "Starting server..."
    } satisfies AcknowledgeSchema;
    dependencies.messageLayer.sendMessage(acknowledge);

    let rendererResult = dependencies.rendererFactory.createRenderer(requestedCapabilities);

    rendererResult.$progress.subscribe((data) => {
        const progress = {
            protocolVersion: state.protocol.version,
            messageType: "init_progress",
            progress: data.progress,
            message: `${data.rendererName}: ${data.message}`
        } satisfies InitProgressSchema;

        dependencies.messageLayer.sendMessage(progress);
    });

    let renderer = await rendererResult.renderer;
    dependencies.rendererProvider.setRenderer(renderer);

    await dependencies.presenter.start();
}

function validateCapabilities(requestedCapabilities: Capabilities, dependencies: Dependencies, state: EngineState): boolean {
    var hasMarkdown = requestedCapabilities.has(Capability.capabilities.render.markdown.self);
    var hasPresenter = requestedCapabilities.has(Capability.capabilities.present.live);

    if (!hasMarkdown) {
        reject("Requested capabilities does not have a valid renderer", dependencies, state);
        return false;
    }

    if (!hasPresenter) {
        reject("Requested capabilities does not have a valid presenter", dependencies, state);
        return false;
    }

    return true;
}

function reject(reason: string, dependencies: Dependencies, state: EngineState) {
    const message = {
        capabilities: state.capabilities.own.toStringList(),
        protocolVersion: state.protocol.version,
        messageType: "reject",
        reason: reason
    } satisfies RejectSchema;

    dependencies.messageLayer.sendMessage(message);
}
