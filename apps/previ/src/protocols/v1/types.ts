import * as initSchema from './outgoing/init.schema.json';
import * as acknowledgeSchema from './outgoing/acknowledge.schema.json';
import * as rejectSchema from './outgoing/reject.schema.json';
import * as initProgressSchema from './outgoing/init_progress.schema.json';
import * as previewAcknowledgeSchema from './outgoing/preview_acknowledge.schema.json';
import * as previewRejectSchema from './outgoing/preview_reject.schema.json';
import * as previewCompleteSchema from './outgoing/preview_complete.schema.json';
import * as previewProgressSchema from './outgoing/preview_progress.schema.json';
import * as previewFailSchema from './outgoing/preview_fail.schema.json';
import * as shutdownOutgoingSchema from './outgoing/shutdown.schema.json';
import * as logSchema from './outgoing/log.schema.json';

import * as startSchema from './incoming/start.schema.json';
import * as updateFilesystemSchema from './incoming/update_filesystem.schema.json';
import * as requestPreviewSchema from './incoming/request_preview.schema.json';
import * as shutdownIncomingSchema from './incoming/shutdown.schema.json';

export type * from './acknowledge.schema';
export type * from './init.schema';
export type * from './log.schema';
export type * from './preview_acknowledge.schema';
export type * from './preview_complete.schema';
export type * from './preview_fail.schema';
export type * from './preview_progress.schema';
export type * from './preview_reject.schema';
export type * from './reject.schema';
export type * from './request_preview.schema';
export type * from './shutdown.schema';
export type * from './start.schema';
export type * from './update_filesystem.schema';
export type * from './init_progress.schema';

import type { AcknowledgeSchema } from './acknowledge.schema';
import type { InitSchema } from './init.schema';
import type { LogSchema } from './log.schema';
import type { PreviewAcknowledgeSchema } from './preview_acknowledge.schema';
import type { PreviewCompleteSchema } from './preview_complete.schema';
import type { PreviewFailSchema } from './preview_fail.schema';
import type { PreviewProgressSchema } from './preview_progress.schema';
import type { PreviewRejectSchema } from './preview_reject.schema';
import type { RejectSchema } from './reject.schema';
import type { RequestPreviewSchema } from './request_preview.schema';
import type { ShutdownSchema } from './shutdown.schema';
import type { StartSchema } from './start.schema';
import type { UpdateFilesystemSchema } from './update_filesystem.schema';
import type { InitProgressSchema } from './init_progress.schema';

export {
    initSchema,
    acknowledgeSchema,
    rejectSchema,
    previewAcknowledgeSchema,
    previewRejectSchema,
    previewCompleteSchema,
    previewProgressSchema,
    previewFailSchema,
    shutdownOutgoingSchema,
    logSchema,
    startSchema,
    updateFilesystemSchema,
    requestPreviewSchema,
    shutdownIncomingSchema,
    initProgressSchema,
};

export type IncomingMessages =
    | StartSchema
    | UpdateFilesystemSchema
    | RequestPreviewSchema
    | ShutdownSchema;
export type OutgoingMessages =
    | InitSchema
    | AcknowledgeSchema
    | RejectSchema
    | PreviewAcknowledgeSchema
    | PreviewRejectSchema
    | PreviewCompleteSchema
    | PreviewProgressSchema
    | PreviewFailSchema
    | ShutdownSchema
    | LogSchema
    | InitProgressSchema;
