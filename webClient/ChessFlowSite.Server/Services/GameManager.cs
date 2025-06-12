using ChessFlowSite.Server.Hubs;
using ChessFlowSite.Server.Models;
using ChessFlowSite.Server.Sessions;
using Microsoft.AspNetCore.SignalR;
using Microsoft.EntityFrameworkCore;
using ReactApp1.Server.Data;
using System.Drawing;

namespace ChessFlowSite.Server.Services
{
    public class GameManager
    {
        private readonly object _lock = new();

        private readonly List<Player> _waitingPlayers = new();
        private readonly Dictionary<int, GameSession> _activeGames = new();
        private readonly IDbContextFactory<ApplicationDbContext> _dbFactory;

        public GameManager(IDbContextFactory<ApplicationDbContext> dbFactory)
        {
            _dbFactory = dbFactory;
        }

        public async Task AddToQueueAsync(Player player, IHubCallerClients clients) {
            lock (_lock) {
                var eligiblePlayers = _waitingPlayers.Where(p => Math.Abs(p.Elo - player.Elo) <= 100 && p.Format == player.Format && p.ConnectionId != player.ConnectionId).ToList();
                var random = new Random();
                var matchedPlayer = eligiblePlayers.Count > 0
                ? eligiblePlayers[random.Next(eligiblePlayers.Count)]
                : null;

                if (matchedPlayer != null)
                {
                    Console.WriteLine($"Matched {player.Username} with {matchedPlayer.Username}");
                    _waitingPlayers.Remove(matchedPlayer);
                    StartGame(player, matchedPlayer, clients);
                }
                else {
                    _waitingPlayers.Add(player);
                }
            }
        }

        public async Task StartBotGame(Player player, int botId, IHubCallerClients clients) {
            await using var db = _dbFactory.CreateDbContext();

            Game game = new Game();
            if (!player.isGuest)
            {
                game.PlayerWhiteId = player.UserId;
            }
            else
            {
                game.GuestWhiteName = player.Username;
            }
            game.GuestBlackName = 
                botId switch
                {
                    0 => "ChessFlow Engine",
                    _ => "Unknown Bot"
                };
            game.BotId = botId.ToString();
            game.IsBotGame = true;
            game.IsRated = true; //temp maybe
            game.Format = player.Format;
            game.EloWhite = player.Elo;
            game.EloBlack = botId switch
            {
                0 => 1500, // ChessFlow Engine default elo
                _ => 1200 // Default elo for unknown bots
            };

            var botElo = botId switch
            {
                0 => 1500, // ChessFlow Engine default elo
                _ => 1200 // Default elo for unknown bots
            };

            db.Games.Add(game);
            await db.SaveChangesAsync();

            // get the game with players included (if they are not guests)
            game = await db.Games
            .Include(g => g.PlayerWhite)
            .Include(g => g.PlayerBlack)
            .FirstOrDefaultAsync(g => g.Id == game.Id);


            var session = new GameSession(game.Id, player, null, clients, this);
            lock (_lock)
            {
                _activeGames[game.Id] = session;
            }

            await clients.Client(player.ConnectionId).SendAsync("GameStarted", game.Id, new GameDataModel(game, "w", player.Elo, botElo));
        }

        public async Task ProcessMoveAsync(int gameId, string connectionId, string move)
        {
            if (_activeGames.TryGetValue(gameId, out var session))
            {
                await session.SendMoveToGM(move, connectionId);
            }
        }

        public async Task HandleDisconnectAsync(string connectionId)
        {
            lock (_lock)
            {
                // Remove from active games
                var session = _activeGames.Values.FirstOrDefault(g => g.ContainsPlayer(connectionId));
                if (session != null)
                {
                    _activeGames.Remove(session.GameId);

                    session.handleResign(connectionId);
                }

                // Remove from queue
                _waitingPlayers.RemoveAll(p => p.ConnectionId == connectionId);
            }
        }

        public async Task OfferDrawAsync(string connectionId)
        {
            lock (_lock)
            {
                var session = _activeGames.Values.FirstOrDefault(g => g.ContainsPlayer(connectionId));
                if (session != null)
                {
                    session.OfferDraw(connectionId);
                }
            }
        }

        private async Task StartGame(Player p1, Player p2, IHubCallerClients clients) {
            await using var db = _dbFactory.CreateDbContext();

            Game game = new Game();
            if (!p1.isGuest)
            {
                game.PlayerWhiteId = p1.UserId;
            }
            else {
                game.GuestWhiteName = p1.Username;
            }
            if (!p2.isGuest)
            {
                game.PlayerBlackId = p2.UserId;
            }
            else
            {
                game.GuestBlackName = p2.Username;
            }
            game.IsBotGame = false;
            game.IsRated = true; //temp maybe
            game.Format = p1.Format;
            game.EloWhite = p1.Elo;
            game.EloBlack = p2.Elo;

            db.Games.Add(game);
            await db.SaveChangesAsync();

            // get the game with players included (if they are not guests)
            game = await db.Games
            .Include(g => g.PlayerWhite)
            .Include(g => g.PlayerBlack)
            .FirstOrDefaultAsync(g => g.Id == game.Id);

            var session = new GameSession(game.Id, p1, p2, clients, this);
            lock (_lock)
            {
                _activeGames[game.Id] = session;
            }

            await clients.Client(p1.ConnectionId).SendAsync("GameStarted", game.Id, new GameDataModel(game, "w", p1.Elo, p2.Elo));
            await clients.Client(p2.ConnectionId).SendAsync("GameStarted", game.Id, new GameDataModel(game, "b", p2.Elo, p1.Elo));
        }

        public async Task EndGame(int GameId, Player playerWhite, Player? playerBlack, int deltaEloWhite, int deltaEloBlack, string? winner, string result, int moveCount, string finalFen, string PGN) {
            lock (_lock) {
                if (_waitingPlayers.Any(p => p.ConnectionId == playerWhite.ConnectionId))
                {
                    _waitingPlayers.Remove(playerWhite);
                }
                if (playerBlack != null && _waitingPlayers.Any(p => p.ConnectionId == playerBlack.ConnectionId))
                {
                    _waitingPlayers.Remove(playerBlack);
                }
                GameHub.RemoveConnection(playerWhite.ConnectionId);
                if (playerBlack != null)
                {
                    GameHub.RemoveConnection(playerBlack.ConnectionId);
                }
            }
            // update database

            await using var db = _dbFactory.CreateDbContext();
            var game = await db.Games
                .Include(g => g.PlayerWhite)
                .Include(g => g.PlayerBlack)
                .FirstOrDefaultAsync(g => g.Id == GameId);
            if (game != null) {
                game.DeltaEloWhite = deltaEloWhite;
                game.DeltaEloBlack = deltaEloBlack;
                string realResult = "";
                if (winner != null) {
                    realResult = winner == "white" ? "White" : "Black";
                }
                else if (result == "agreedDraw")
                {
                    realResult = "Draw";
                }
                else if(result == "stalemate")
                {
                    realResult = "Stalemate"; 
                }
                else
                {
                    throw new ArgumentException("Invalid result type");
                }

                game.Result = realResult;
                game.EndTime = DateTime.UtcNow;

                game.FinalFEN = finalFen;
                game.PGN = PGN;
                game.MoveCount = moveCount;

                await updateELoPlayer(playerWhite, deltaEloWhite);
                await updateELoPlayer(playerBlack, deltaEloBlack);

                await db.SaveChangesAsync();
            }

        }

        public async Task RequestBoardAsync(string connectionId)
        {
            lock (_lock)
            {
                var session = _activeGames.Values.FirstOrDefault(g => g.ContainsPlayer(connectionId));
                if (session != null)
                {
                    session.SendBoardToClient(connectionId);
                }
            }
        }

        private async Task updateELoPlayer(Player player, int deltaELo) {
            if(player != null && !player.isGuest)
            {
                await using var db = _dbFactory.CreateDbContext();
                var user = await db.ApplicationUsers.FirstOrDefaultAsync(u => u.Id == player.UserId);
                if (user != null)
                {
                    user.Elo = player.Elo + deltaELo;
                    await db.SaveChangesAsync();
                }
            }
        }
    }

    public class GameDataModel {
        public string Side { get; set; } // "white" or "black"
        public string Name { get; set; }
        public string OpponentName { get; set; }
        public int Elo { get; set; }
        public int OpponentElo { get; set; }

        public bool IsGuest { get; set; }
        public bool IsOpponentGuest { get; set; }
        public bool IsBotGame { get; set; }
        public string Format { get; set; } // "Blitz", "Bullet", "Classical"

        public int Timer { get; set; }
        public int OpponentTimer { get; set; }
        public string Fen { get; set; }

        public GameDataModel(Game game, string side, int elo, int opponentElo) {

            Side = side;
            Name = side == "w" ? game.GuestWhiteName ?? game.PlayerWhite?.Name : game.GuestBlackName ?? game.PlayerBlack?.Name;
            OpponentName = side == "w" ? game.GuestBlackName ?? game.PlayerBlack?.Name : game.GuestWhiteName ?? game.PlayerWhite?.Name;
            Elo = elo;
            OpponentElo = opponentElo;
            IsGuest = side == "w" ? game.GuestWhiteName != null : game.GuestBlackName != null;
            IsOpponentGuest = side == "w" ? game.GuestBlackName != null : game.GuestWhiteName != null;
            IsBotGame = game.IsBotGame;
            Format = game.Format ?? "Classical"; // Default to Classical if not set
            Timer = Format == "Bullet" ? 300000 :
                    Format == "Blitz" ? 600000 :
                    Format == "Classical" ? 3600000 : 0; // Default timer values for different formats
            OpponentTimer = Timer; // Assuming both players start with the same timer
            Fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"; // Initial FEN for a new game, temporary hardcoded string for now
        }
    }


}
