import type { LessonsEntry, ParsedModeDoc, RegisterBand } from './types.js';
/** Parse a mode-doc Markdown file at `path`. Throws on missing required fields. */
export declare function parseModeDoc(path: string): ParsedModeDoc;
export declare function parseModeDocument(content: string, sourcePath: string): ParsedModeDoc;
export declare function summarizeLessons(lessons: LessonsEntry[], maxEntries?: number): string;
export declare function getPassTemplate(doc: ParsedModeDoc, pass_id: string, register: RegisterBand): string;
export declare function getTargetWordsForRegister(doc: ParsedModeDoc, register: RegisterBand): {
    min: number;
    max: number;
};
//# sourceMappingURL=parser.d.ts.map