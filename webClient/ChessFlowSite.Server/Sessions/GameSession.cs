using ChessFlowSite.Server.Models;
using ChessFlowSite.Server.Services;
using Microsoft.AspNetCore.SignalR;
using System.Diagnostics;
using System.Text.Json;

namespace ChessFlowSite.Server.Sessions
{
    public class GameSession
    {
        private readonly Process _process;
        private readonly StreamWriter _writer;
        private readonly StreamReader _reader;

        private readonly GameManager _gameManager;

        public int GameId { get; set; }
        public Player PlayerWhite { get; set; }
        public Player? PlayerBlack { get; set; } // can be null if it's a bot game

        public bool IsBot => PlayerBlack == null;
        public int? BotId { get; set; } // used for bot games

        private bool DrawWhite { get; set; } = false;
        public bool DrawBlack { get; set; } = false; // used for draw offers

        public string? BotName => BotId switch
        {
            0 => "ChessFlow Engine",
            _ => "Unknown Bot"
        };
        public IHubCallerClients Clients { get; set; }

        public GameSession(int gameId, Player p1, Player p2, IHubCallerClients clients, GameManager gameManager, int? botId = null)
        {
            _gameManager = gameManager;

            GameId = gameId;
            Clients = clients;

            PlayerWhite = p1;
            PlayerBlack = p2;
            BotId = botId;

            string path = "../../GM/target/release/GM";
            string pipeName = $"chessflow-{gameId}";
            _process = new Process
            {
                StartInfo = new ProcessStartInfo
                {
                    FileName = path,
                    Arguments = pipeName,
                    UseShellExecute = false,
                    CreateNoWindow = true,
                    RedirectStandardError = true,
                    RedirectStandardInput = true,
                    RedirectStandardOutput = true,
                    WindowStyle = ProcessWindowStyle.Hidden
                }
            };

            _process.Start();

            _ = Task.Run(async () =>
            {
                var errReader = _process.StandardError;
                while (!errReader.EndOfStream)
                {
                    var line = await errReader.ReadLineAsync();
                    Console.Error.WriteLine($"[Rust Error] {line}");
                }
            });

            _reader = _process.StandardOutput;
            _writer = _process.StandardInput;

            Console.WriteLine("Connected to GM");

            // Send initial game start message
            int initialTimeMS = (PlayerWhite.Format) switch
            {
                "Bullet" => 300000,
                "Blitz" => 600000,
                "Classical" => 3600000,
                _ => throw new ArgumentOutOfRangeException(nameof(PlayerWhite.Format), "Unsupported game format")
            };

            var gameStartModel = new GameStartModel
            {
                game_id = gameId.ToString(),
                white_elo = PlayerWhite.Elo,
                black_elo = PlayerBlack?.Elo ?? 1500,
                initial_time_ms = initialTimeMS,
                increment_ms = 0,
                is_bot_game = IsBot
            };
            SendAsync(gameStartModel);

            _ = Task.Run(ReadLoop);
        }

        public string? GetOpponentId(string connectionId)
        {
            if (PlayerWhite?.ConnectionId == connectionId)
                return PlayerBlack?.ConnectionId;
            if (PlayerBlack?.ConnectionId == connectionId)
                return PlayerWhite?.ConnectionId;
            return null;
        }

        public bool ContainsPlayer(string connectionId) =>
            PlayerWhite?.ConnectionId == connectionId || PlayerBlack?.ConnectionId == connectionId;

        public Task SendMoveToGM(string move, string connectionId)
        {
            var clientColor = PlayerWhite.ConnectionId == connectionId ? "white" : "black";
            Console.WriteLine(String.Format("{0} from {1}", move, clientColor));
            var makeMoveModel = new MakeMoveModel
            {
                san_move = move
            };
            return SendAsync(makeMoveModel);
        }

        public async Task handleResign(string connectionId)
        {
            var clientColor = PlayerWhite.ConnectionId == connectionId ? "white" : "black";
            Console.WriteLine($"Resign from {clientColor}");
            var resignModel = new ResignModel
            {
                type = "resign",
                player_color = clientColor
            };
            await SendAsync(resignModel);
        }

        public async Task OfferDraw(string connectionId) {
            if(IsBot) return; // doesn't make sense to offer a draw in bot games

            if (connectionId == PlayerWhite.ConnectionId)
            {
                DrawWhite = true;
                string opponentId = GetOpponentId(connectionId);
                if (opponentId != null) {
                    await Clients.Client(opponentId).SendAsync("GameEvent", "UpdateDraw", new {});
                }
            }
            else if (connectionId == PlayerBlack?.ConnectionId)
            {
                DrawBlack = true;
                string opponentId = GetOpponentId(connectionId);
                if (opponentId != null)
                {
                    await Clients.Client(opponentId).SendAsync("GameEvent", "UpdateDraw", new { });
                }
            }

            if(DrawWhite && DrawBlack)
            {
                var claimDrawModel = new ClaimDrawModel();
                await SendAsync(claimDrawModel);
            }
        }

        public async Task SendBoardToClient(string connectionId)
        {
            var clientColor = PlayerWhite.ConnectionId == connectionId ? "white" : "black";
            var requestBoardModel = new RequestBoardModel(); 
            await SendAsync(requestBoardModel);
        }

        private async Task SendAsync(object message)
        {
            var json = JsonSerializer.Serialize(message);
            Console.WriteLine($"Sending to GM:\n{json}");
            await _writer.WriteLineAsync(json);
        }

        private async Task ReadLoop()
        {
            while (true)
            {
                var line = await _reader.ReadLineAsync();
                if (line == null) break;
                Console.WriteLine($"Received from GM:\n{line}");
                using JsonDocument doc = JsonDocument.Parse(line);
                var root = doc.RootElement;
                string type = root.GetProperty("type").GetString();
                Console.WriteLine(type);
                if (type == "moveResult")
                {
                    string fen = root.GetProperty("fen").GetString();
                    string turn = root.GetProperty("turn").GetString();
                    int whiteTime = root.GetProperty("white_ms").GetInt32();
                    int blackTime = root.GetProperty("black_ms").GetInt32();
                    string lastMove = root.GetProperty("last_move").GetString();
                    bool valid = root.GetProperty("is_valid").GetBoolean();
                    string? moveHistory = null;
                    if (root.TryGetProperty("move_history", out var moveHistoryElement) && moveHistoryElement.ValueKind == JsonValueKind.String)
                    {
                        moveHistory = moveHistoryElement.GetString();
                    }
                    var payload = new
                    {
                        fen = fen,
                        turn = turn,
                        whiteTime = whiteTime,
                        blackTime = blackTime,
                        valid = valid,
                        lastMove = lastMove,
                        moveHistory = moveHistory
                    };
                    Console.WriteLine(payload);
                    await Clients.Clients(PlayerWhite.ConnectionId).SendAsync("GameEvent", "MoveResult", payload);
                    if (PlayerBlack != null)
                    {
                        await Clients.Clients(PlayerBlack!.ConnectionId).SendAsync("GameEvent", "MoveResult", payload);
                    }
                }
                else if (type == "gameOver") {
                    string result = root.GetProperty("reason").GetString();
                    string? winner = root.GetProperty("winner").GetString();
                    int deltaEloWhite = root.GetProperty("white_elo_change").GetInt32();
                    int deltaEloBlack = root.GetProperty("black_elo_change").GetInt32();
                    string finalFen = root.GetProperty("fen").GetString();
                    int moveCount = root.GetProperty("move_count").GetInt32();
                    moveCount = (moveCount + 1) / 2;
                    string PGN = AddPgnMoveNumbers(root.GetProperty("pgn").GetString());



                    var payload = new
                    {
                        result = result,
                        winner = winner,
                        deltaEloWhite = deltaEloWhite,
                        deltaEloBlack = deltaEloBlack
                    };
                    Console.WriteLine(payload);
                    await _gameManager.EndGame(GameId, PlayerWhite, PlayerBlack, deltaEloWhite, deltaEloBlack, winner, result, moveCount, finalFen, PGN);
                    await Clients.Clients(PlayerWhite.ConnectionId).SendAsync("GameEvent", "GameOver", payload);
                    if (PlayerBlack != null)
                    {
                        await Clients.Clients(PlayerBlack!.ConnectionId).SendAsync("GameEvent", "GameOver", payload);
                    }
                }
            }
        }

        private static string AddPgnMoveNumbers(string moveHistory)
        {
            if (string.IsNullOrWhiteSpace(moveHistory)) return string.Empty;

            var moves = moveHistory.Split(' ', StringSplitOptions.RemoveEmptyEntries);
            var numberedMoves = new List<string>();

            for (int i = 0; i < moves.Length; i += 2)
            {
                int moveNumber = (i / 2) + 1;
                if (i + 1 < moves.Length)
                {
                    // White and Black move
                    numberedMoves.Add($"{moveNumber}.{moves[i]} {moves[i + 1]}");
                }
                else
                {
                    // Only White move (game ended on White's turn)
                    numberedMoves.Add($"{moveNumber}.{moves[i]}");
                }
            }

            return string.Join(" ", numberedMoves);
        }
    }
    public class GameStartModel {
        public string type { get; set; } = "startGame";
        public string game_id { get; set; }
        public int white_elo { get; set; }
        public int black_elo { get; set; }
        public int initial_time_ms { get; set; }
        public int increment_ms { get; set; } = 0;
        public bool is_bot_game { get; set; }
    }

    public class MakeMoveModel {
        public string type { get; set; } = "makeMove";
        public string san_move { get; set; }
    }

    public class ResignModel {
        public string type { get; set; } = "resign";
        public string player_color { get; set; }
    }

    public class RequestBoardModel {
        public string type { get; set; } = "requestBoard";
    }

    public class ClaimDrawModel {
        public string type { get; set; } = "claimDraw";
    }
}
