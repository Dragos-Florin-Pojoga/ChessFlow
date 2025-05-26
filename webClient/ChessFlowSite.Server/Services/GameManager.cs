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

            db.Games.Add(game);
            await db.SaveChangesAsync();

            var session = new GameSession(game.Id, player, null, clients);
            lock (_lock)
            {
                _activeGames[game.Id] = session;
            }

            await clients.Client(player.ConnectionId).SendAsync("GameStarted", game.Id, "white", game.GuestBlackName);
        }

        public async Task ProcessMoveAsync(int gameId, string connectionId, string move)
        {
            if (_activeGames.TryGetValue(gameId, out var session))
            {
                var opponentId = session.GetOpponentId(connectionId);
                if (opponentId != null)
                {
                    await session.SendMoveToOpponent(move, opponentId);
                }
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

            db.Games.Add(game);
            await db.SaveChangesAsync();

            var session = new GameSession(game.Id, p1, p2, clients);
            lock (_lock)
            {
                _activeGames[game.Id] = session;
            }

            await clients.Client(p1.ConnectionId).SendAsync("GameStarted", game.Id, "white", p2.Username);
            await clients.Client(p2.ConnectionId).SendAsync("GameStarted", game.Id, "black", p1.Username);
        }
    }
}
