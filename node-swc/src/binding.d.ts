/* tslint:disable */
/* eslint-disable */

/* auto-generated by NAPI-RS */

export interface TransformOutput {
  code: string
  map?: string
}
export function bundle(confItems: Buffer, signal?: AbortSignal | undefined | null): Promise<{ [index: string]: { code: string, map?: string } }>
export function minify(code: Buffer, opts: Buffer, signal?: AbortSignal | undefined | null): Promise<TransformOutput>
export function minifySync(code: Buffer, opts: Buffer): TransformOutput
export function parse(src: string, options: Buffer, filename?: string | undefined | null, signal?: AbortSignal | undefined | null): Promise<string>
export function parseSync(src: string, opts: Buffer, filename?: string | undefined | null): string
export function parseFileSync(path: string, opts: Buffer): string
export function parseFile(path: string, options: Buffer, signal?: AbortSignal | undefined | null): Promise<string>
export function print(programJson: string, options: Buffer, signal?: AbortSignal | undefined | null): Promise<TransformOutput>
export function printSync(program: string, options: Buffer): TransformOutput
export function transform(src: string, isModule: boolean, options: Buffer, signal?: AbortSignal | undefined | null): Promise<TransformOutput>
export function transformSync(s: string, isModule: boolean, opts: Buffer): TransformOutput
export function transformFile(src: string, isModule: boolean, options: Buffer, signal?: AbortSignal | undefined | null): Promise<TransformOutput>
export function transformFileSync(s: string, isModule: boolean, opts: Buffer): TransformOutput
export function getTargetTriple(): string
export function initCustomTraceSubscriber(traceOutFilePath?: string | undefined | null): void
/** Hack for `Type Generation` */
export interface TransformOutput {
  code: string
  map?: string
}
export type JsCompiler = Compiler
export class Compiler {
  constructor()
}
