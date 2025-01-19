/* tslint:disable */
/* eslint-disable */
export class Tetris {
  free(): void;
  constructor(canvas_id: string);
  render(): void;
  move_down(): void;
  move_left(): void;
  move_right(): void;
  start(): void;
  rotate_clockwise(): void;
  rotate_counterclockwise(): void;
  game_over: boolean;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_tetris_free: (a: number, b: number) => void;
  readonly __wbg_get_tetris_game_over: (a: number) => number;
  readonly __wbg_set_tetris_game_over: (a: number, b: number) => void;
  readonly tetris_new: (a: number, b: number) => [number, number, number];
  readonly tetris_render: (a: number) => void;
  readonly tetris_move_down: (a: number) => void;
  readonly tetris_move_left: (a: number) => void;
  readonly tetris_move_right: (a: number) => void;
  readonly tetris_start: (a: number) => void;
  readonly tetris_rotate_clockwise: (a: number) => void;
  readonly tetris_rotate_counterclockwise: (a: number) => void;
  readonly __wbindgen_exn_store: (a: number) => void;
  readonly __externref_table_alloc: () => number;
  readonly __wbindgen_export_2: WebAssembly.Table;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __externref_table_dealloc: (a: number) => void;
  readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;
/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
*
* @returns {InitOutput}
*/
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
