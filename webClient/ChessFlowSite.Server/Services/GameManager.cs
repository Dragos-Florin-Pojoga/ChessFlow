using ChessFlowSite.Server.Models;
using ChessFlowSite.Server.Sessions;
using Microsoft.AspNetCore.SignalR;
using Microsoft.EntityFrameworkCore;
using ReactApp1.Server.Data;

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


            var session = new GameSession(game.Id, player, null, clients);
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

                    var opponentId = session.GetOpponentId(connectionId);
                    if (opponentId != null)
                    {
                        session.Clients.Client(opponentId).SendAsync("OpponentDisconnected");
                    }
                }

                // Remove from queue
                _waitingPlayers.RemoveAll(p => p.ConnectionId == connectionId);
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

            var session = new GameSession(game.Id, p1, p2, clients);
            lock (_lock)
            {
                _activeGames[game.Id] = session;
            }

            await clients.Client(p1.ConnectionId).SendAsync("GameStarted", game.Id, new GameDataModel(game, "w", p1.Elo, p2.Elo));
            await clients.Client(p2.ConnectionId).SendAsync("GameStarted", game.Id, new GameDataModel(game, "b", p2.Elo, p1.Elo));
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
            Timer = Format == "Bullet" ? 300 :
                    Format == "Blitz" ? 600 :
                    Format == "Classical" ? 3600 : 0; // Default timer values for different formats
            OpponentTimer = Timer; // Assuming both players start with the same timer
            Fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"; // Initial FEN for a new game, temporary hardcoded string for now
        }
    }


}
