import { createEngineWorker } from '/public/engine.js';

const ChessFlowEngine = await createEngineWorker();

type EngineMessage = {
    /** engine message in UCI format*/
    uciMessage: string;
    /** found best move for current position in format `e2e4`*/
    bestMove?: string;
    /** found best move for opponent in format `e7e5` */
    ponder?: string;
    /**  material balance's difference in centipawns(IMPORTANT! the engine gives the cp score in terms of whose turn it is)*/
    positionEvaluation?: string;
    /** count of moves until mate */
    possibleMate?: string;
    /** the best line found */
    pv?: string;
    /** number of halfmoves the engine looks ahead */
    depth?: number;
};

export default class Engine {
    chess_engine_interface: any;
    onMessage: (callback: (messageData: EngineMessage) => void) => void;
    isReady: boolean;

    constructor() {
        this.chess_engine_interface = ChessFlowEngine;
        this.chess_engine_interface.start_wasm_engine();
        this.chess_engine_interface.send_uci_message(`setoption name is_evaluation_mode value true`);
        this.chess_engine_interface.send_uci_message(`setoption name max_depth value 14`);
        this.isReady = false;
        this.onMessage = (callback) => {
            this.chess_engine_interface.register_callback((e) => {
                callback(this.transformSFMessageData(e));
            });
        };
        this.init();
    }

    private transformSFMessageData(e) {
        const uciMessage = e?.data ?? e;

        return {
            uciMessage,
            bestMove: uciMessage.match(/bestmove\s+(\S+)/)?.[1],
            ponder: uciMessage.match(/ponder\s+(\S+)/)?.[1],
            positionEvaluation: uciMessage.match(/cp\s+(\S+)/)?.[1],
            possibleMate: uciMessage.match(/mate\s+(\S+)/)?.[1],
            pv: uciMessage.match(/ pv\s+(.*)/)?.[1],
            depth: Number(uciMessage.match(/ depth\s+(\S+)/)?.[1]) ?? 0,
        };
    }

    init() {
        this.chess_engine_interface.send_uci_message("uci");
        this.chess_engine_interface.send_uci_message("isready");
        this.onMessage(({ uciMessage }) => {
            if (uciMessage === "readyok") {
                this.isReady = true;
            }
        });
    }

    onReady(callback) {
        this.onMessage(({ uciMessage }) => {
            if (uciMessage === "readyok") {
                callback();
            }
        });
    }

    evaluatePositionUntilDepth(fen, depth) {
        this.chess_engine_interface.send_uci_message(`position fen ${fen}`);
        this.chess_engine_interface.send_uci_message(`go depth ${depth}`);
    }

    stop() {
        this.chess_engine_interface.send_uci_message("stop"); // Run when searching takes too long, will return you the bestmove of the deep it has reached
    }

    terminate() {
        this.isReady = false;
        this.chess_engine_interface.send_uci_message("quit"); // Run this before chessboard unmounting.
    }
}