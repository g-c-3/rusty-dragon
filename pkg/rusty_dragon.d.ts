/* tslint:disable */
/* eslint-disable */

export class Engine {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Search to `depth` plies and return the best move in UCI notation,
     * or an empty string if there are no legal moves.
     */
    best_move(depth: number): string;
    /**
     * Current position as FEN.
     */
    fen(): string;
    /**
     * Whether the side to move is currently in check.
     */
    is_check(): boolean;
    /**
     * Whether the current position is a draw by the 50-move rule.
     */
    is_fifty_move_draw(): boolean;
    /**
     * Whether the current position has occurred three times.
     */
    is_repetition(): boolean;
    /**
     * All legal moves from the current position, as a space-separated
     * string of UCI move strings (e.g. "e2e4 g1f3 e7e8q").
     */
    legal_moves(): string;
    /**
     * Apply a move given in UCI notation (e.g. "e2e4", "e7e8q").
     * Returns true if the move was legal and was applied.
     */
    make_move(uci: string): boolean;
    /**
     * Create a new engine at the standard starting position.
     */
    constructor();
    /**
     * perft(depth) node count, returned as a string (perft(6)+ exceeds
     * what JS can represent exactly as a f64-backed Number).
     */
    perft(depth: number): string;
    /**
     * Reset to the standard starting position.
     */
    reset(): void;
    /**
     * Load a position from FEN.
     */
    set_fen(fen: string): void;
    /**
     * 1 if White to move, -1 if Black to move.
     */
    side_to_move(): number;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_engine_free: (a: number, b: number) => void;
    readonly engine_best_move: (a: number, b: number, c: number) => void;
    readonly engine_fen: (a: number, b: number) => void;
    readonly engine_is_check: (a: number) => number;
    readonly engine_is_fifty_move_draw: (a: number) => number;
    readonly engine_is_repetition: (a: number) => number;
    readonly engine_legal_moves: (a: number, b: number) => void;
    readonly engine_make_move: (a: number, b: number, c: number) => number;
    readonly engine_new: () => number;
    readonly engine_perft: (a: number, b: number, c: number) => void;
    readonly engine_reset: (a: number) => void;
    readonly engine_set_fen: (a: number, b: number, c: number) => void;
    readonly engine_side_to_move: (a: number) => number;
    readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
    readonly __wbindgen_export: (a: number, b: number, c: number) => void;
    readonly __wbindgen_export2: (a: number, b: number) => number;
    readonly __wbindgen_export3: (a: number, b: number, c: number, d: number) => number;
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
